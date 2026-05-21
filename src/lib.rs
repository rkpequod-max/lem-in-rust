use wasm_bindgen::prelude::*;
use std::collections::{HashMap, HashSet};
use std::cell::RefCell;
use std::rc::Rc;

// ============================================================================
// STRUCTURES DE DONNÉES
// ============================================================================

#[derive(Clone, Debug)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

pub type RoomRef = Rc<RefCell<Room>>;

pub struct Room {
    pub name: String,
    pub coord: Coord,
    pub ant: i32,          // ID de la fourmi (0 si vide)
    pub start: bool,
    pub end: bool,
    pub links: Vec<RoomRef>,
    pub f_to: Option<RoomRef>,  // Salle suivante dans le flux
    pub f_fm: Option<RoomRef>,  // Salle précédente dans le flux
}

impl Room {
    pub fn new(name: String, coord: Coord, is_start: bool, is_end: bool) -> Self {
        Room {
            name,
            coord,
            ant: 0,
            start: is_start,
            end: is_end,
            links: Vec::new(),
            f_to: None,
            f_fm: None,
        }
    }
}

pub struct Farm {
    pub ants: i32,
    pub rooms: Vec<RoomRef>,
}

impl Farm {
    pub fn new() -> Self {
        Farm {
            ants: 0,
            rooms: Vec::new(),
        }
    }
}

// ============================================================================
// PARSING ET VALIDATION
// ============================================================================

#[wasm_bindgen]
pub fn parse_and_solve(input: &str) -> String {
    console_error_panic_hook::set_once();
    
    let mut farm = Farm::new();
    let lines: Vec<&str> = input.lines().collect();
    
    let mut section = 0; // 0: ants, 1: rooms, 2: links
    let mut io: Option<i32> = None; // None, Some(0)=start, Some(1)=end
    let mut start_count = 0;
    let mut end_count = 0;
    
    let mut room_names: HashSet<String> = HashSet::new();
    let mut room_coords: HashSet<(i32, i32)> = HashSet::new();
    let mut room_map: HashMap<String, RoomRef> = HashMap::new();
    
    let mut output = String::new();
    
    for line in lines {
        let line = line.trim();
        
        if line.is_empty() {
            continue;
        }
        
        // Gestion des commandes spéciales
        if line == "##start" {
            io = Some(0);
            continue;
        } else if line == "##end" {
            io = Some(1);
            continue;
        } else if line.starts_with('#') {
            continue; // Commentaire
        }
        
        // Section 0: Nombre de fourmis
        if section == 0 {
            if let Ok(ants) = line.parse::<i32>() {
                farm.ants = ants;
                section = 1;
                continue;
            } else {
                section = 1;
            }
        }
        
        // Section 1: Salles
        if section == 1 {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 3 {
                let name = parts[0].to_string();
                if name.starts_with('L') || name.starts_with('#') {
                    section = 2;
                    continue;
                }
                
                if let (Ok(x), Ok(y)) = (parts[1].parse::<i32>(), parts[2].parse::<i32>()) {
                    if room_names.contains(&name) {
                        return format!("ERROR:Room already exists: {}", name);
                    }
                    if room_coords.contains(&(x, y)) {
                        return format!("ERROR:Coordinates not available: ({}, {})", x, y);
                    }
                    
                    let is_start = io == Some(0);
                    let is_end = io == Some(1);
                    
                    if is_start { start_count += 1; }
                    if is_end { end_count += 1; }
                    
                    let room = Rc::new(RefCell::new(Room::new(name.clone(), Coord { x, y }, is_start, is_end)));
                    room_names.insert(name);
                    room_coords.insert((x, y));
                    room_map.insert(room.borrow().name.clone(), room.clone());
                    farm.rooms.push(room);
                    
                    io = None;
                    continue;
                }
            }
            section = 2;
        }
        
        // Section 2: Liens
        if section == 2 {
            if let Some(dash_pos) = line.find('-') {
                let parts: Vec<&str> = line.split('-').collect();
                if parts.len() == 2 && parts[0] != parts[1] {
                    if let (Some(room1), Some(room2)) = (room_map.get(parts[0]), room_map.get(parts[1])) {
                        // Lien bidirectionnel
                        {
                            let mut r1 = room1.borrow_mut();
                            if !r1.links.iter().any(|r| Rc::ptr_eq(r, room2)) {
                                r1.links.push(room2.clone());
                            }
                        }
                        {
                            let mut r2 = room2.borrow_mut();
                            if !r2.links.iter().any(|r| Rc::ptr_eq(r, room1)) {
                                r2.links.push(room1.clone());
                            }
                        }
                        continue;
                    }
                }
            }
            return format!("ERROR:Improper link format: {}", line);
        }
    }
    
    // Validation des endpoints
    if start_count != 1 || end_count != 1 {
        return format!("ERROR:Improper number of endpoints (start: {}, end: {})", start_count, end_count);
    }
    
    if farm.rooms.is_empty() {
        return "ERROR:There are no rooms".to_string();
    }
    
    // Affichage de la carte originale
    for line in lines {
        output.push_str(line);
        output.push('\n');
    }
    output.push('\n');
    
    // Résolution
    solve(&mut farm, &mut output);
    
    output
}

// ============================================================================
// ALGORITHME BFS
// ============================================================================

fn find_start(rooms: &[RoomRef]) -> Option<RoomRef> {
    rooms.iter().find(|r| r.borrow().start).cloned()
}

fn find_end(rooms: &[RoomRef]) -> Option<RoomRef> {
    rooms.iter().find(|r| r.borrow().end).cloned()
}

fn bfs_find_path(start: &RoomRef, end: &RoomRef, ignore_flow: bool) -> Option<Vec<RoomRef>> {
    use std::collections::VecDeque;
    
    let mut visited: HashSet<usize> = HashSet::new();
    let mut parent: HashMap<usize, usize> = HashMap::new();
    let mut queue: VecDeque<RoomRef> = VecDeque::new();
    
    queue.push_back(start.clone());
    visited.insert(Rc::as_ptr(start) as usize);
    
    while let Some(current) = queue.pop_front() {
        let current_ptr = Rc::as_ptr(&current) as usize;
        
        // Vérifier si on a atteint la fin
        if Rc::ptr_eq(&current, end) {
            // Reconstruire le chemin
            let mut path = Vec::new();
            let mut ptr = Rc::as_ptr(end) as usize;
            
            loop {
                // Trouver la room correspondant au ptr
                let room = rooms.iter().find(|r| Rc::as_ptr(r) as usize == ptr).unwrap().clone();
                path.push(room.clone());
                
                if Rc::ptr_eq(&room, start) {
                    break;
                }
                
                if let Some(&prev_ptr) = parent.get(&ptr) {
                    ptr = prev_ptr;
                } else {
                    break;
                }
            }
            
            path.reverse();
            return Some(path);
        }
        
        // Explorer les voisins
        let neighbors = {
            let curr = current.borrow();
            curr.links.clone()
        };
        
        for neighbor in neighbors {
            let neighbor_ptr = Rc::as_ptr(&neighbor) as usize;
            
            // Vérifier si déjà visité
            if visited.contains(&neighbor_ptr) {
                continue;
            }
            
            // Vérifier les contraintes de flux
            if !ignore_flow {
                let curr_borrow = current.borrow();
                let neighbor_borrow = neighbor.borrow();
                
                // Si on vient du start, ignorer si le voisin a un f_fm différent de current
                if curr_borrow.start && neighbor_borrow.f_fm.as_ref().map_or(false, |f| !Rc::ptr_eq(f, &current)) {
                    continue;
                }
                
                // Sinon, ignorer si current->f_to != neighbor
                if !curr_borrow.start && curr_borrow.f_to.as_ref().map_or(false, |f| !Rc::ptr_eq(f, &neighbor)) {
                    continue;
                }
            }
            
            visited.insert(neighbor_ptr);
            parent.insert(neighbor_ptr, current_ptr);
            queue.push_back(neighbor.clone());
        }
    }
    
    None
}

// ============================================================================
// GESTION DU FLUX
// ============================================================================

fn update_flow(path: &[RoomRef]) {
    for i in 0..path.len() - 1 {
        let current = &path[i];
        let next = &path[i + 1];
        
        let mut curr_borrow = current.borrow_mut();
        let mut next_borrow = next.borrow_mut();
        
        // Établir le flux forward
        if !curr_borrow.start {
            curr_borrow.f_to = Some(next.clone());
        }
        
        // Établir le flux backward
        if !next_borrow.end {
            next_borrow.f_fm = Some(current.clone());
        }
    }
}

// ============================================================================
// SIMULATION DES FOURMIS
// ============================================================================

fn solve(farm: &mut Farm, output: &mut String) {
    let start = match find_start(&farm.rooms) {
        Some(s) => s,
        None => {
            output.push_str("ERROR:No start found\n");
            return;
        }
    };
    
    let end = match find_end(&farm.rooms) {
        Some(e) => e,
        None => {
            output.push_str("ERROR:No end found\n");
            return;
        }
    };
    
    // Trouver les chemins avec BFS
    let mut paths: Vec<Vec<RoomRef>> = Vec::new();
    
    // Premier chemin
    if let Some(path) = bfs_find_path(&start, &end, true) {
        update_flow(&path);
        paths.push(path);
    } else {
        output.push_str("ERROR:Unreachable endpoint\n");
        return;
    }
    
    // Deuxième chemin (si possible)
    if farm.ants > 1 {
        if let Some(path) = bfs_find_path(&start, &end, false) {
            update_flow(&path);
            paths.push(path);
        }
    }
    
    // Simulation tour par tour
    let total_ants = farm.ants;
    let mut ants_sent = 0;
    let mut turn = 0;
    let mut ants_at_end = 0;
    
    while ants_at_end < total_ants {
        turn += 1;
        let mut moved = false;
        
        // Déplacer les fourmis existantes
        for path in &paths {
            for i in (0..path.len()).rev() {
                let room = &path[i];
                let mut room_borrow = room.borrow_mut();
                
                if room_borrow.ant > 0 {
                    let ant_id = room_borrow.ant;
                    
                    if room_borrow.end {
                        // Fourmi arrivée
                        room_borrow.ant = 0;
                        ants_at_end += 1;
                        output.push_str(&format!("L{}-{} ", ant_id, room_borrow.name));
                        moved = true;
                    } else if i < path.len() - 1 {
                        // Déplacer vers la prochaine salle
                        let next_room = &path[i + 1];
                        let mut next_borrow = next_room.borrow_mut();
                        
                        if next_borrow.ant == 0 {
                            room_borrow.ant = 0;
                            next_borrow.ant = ant_id;
                            output.push_str(&format!("L{}-{} ", ant_id, next_borrow.name));
                            moved = true;
                        }
                    }
                }
            }
        }
        
        // Envoyer de nouvelles fourmis depuis le start
        if ants_sent < total_ants {
            for path in &paths {
                if ants_sent >= total_ants {
                    break;
                }
                
                let first_room = &path[0];
                let first_borrow = first_room.borrow();
                
                if first_borrow.ant == 0 {
                    drop(first_borrow);
                    let mut first_borrow = first_room.borrow_mut();
                    ants_sent += 1;
                    first_borrow.ant = ants_sent;
                    output.push_str(&format!("L{}-{} ", ants_sent, first_borrow.name));
                    moved = true;
                }
            }
        }
        
        output.push('\n');
        
        if !moved && ants_sent >= total_ants {
            break; // Plus aucun mouvement possible
        }
    }
}

#[wasm_bindgen(start)]
pub fn main_web() {
    console_error_panic_hook::set_once();
}
