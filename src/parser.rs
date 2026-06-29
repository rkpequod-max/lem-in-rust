use crate::models::{Coord, Farm, Room};
use std::collections::HashSet;
use std::rc::Rc;

/// Résultat du parsing : la ferme construite + les lignes de la map (pour echo).
pub struct ParseResult {
    pub farm: Farm,
    pub map_lines: Vec<String>,
}

/// Parse l'entrée stdin (texte brut) et construit la ferme.
///
/// FSM à 3 sections :
/// - Section 0 : nombre de fourmis
/// - Section 1 : salles (`nom x y`)
/// - Section 2 : liens (`room1-room2`)
///
/// Commandes spéciales : `##start` (prochaine salle = départ), `##end` (prochaine salle = arrivée).
pub fn parse_input(input: &str) -> Result<ParseResult, String> {
    let mut farm = Farm::new();
    let mut map_lines = Vec::new();

    let mut section: i32 = 0; // 0=ants, 1=rooms, 2=links
    let mut io: i32 = -1;     // -1=none, 0=start, 1=end
    let mut start_count = 0;
    let mut end_count = 0;

    let mut room_names: HashSet<String> = HashSet::new();
    let mut room_coords: HashSet<(i32, i32)> = HashSet::new();

    for raw_line in input.lines() {
        let line = raw_line.trim();

        // Skip empty lines but still record them
        if line.is_empty() {
            continue;
        }

        // Gestion des commandes spéciales
        if line == "##start" {
            if io >= 0 {
                return Err("ERROR:Improper or repeated start".to_string());
            }
            io = 0;
            map_lines.push(line.to_string());
            continue;
        } else if line == "##end" {
            if io >= 0 {
                return Err("ERROR:Improper or repeated end".to_string());
            }
            io = 1;
            map_lines.push(line.to_string());
            continue;
        } else if line.starts_with('#') {
            // Commentaire — ignoré mais echo
            map_lines.push(line.to_string());
            continue;
        }

        // Section 0 : nombre de fourmis
        if section == 0 {
            if let Ok(ants) = line.parse::<i32>() {
                if ants < 0 {
                    return Err("ERROR:Invalid number of ants".to_string());
                }
                farm.ants = ants;
                section = 1;
                map_lines.push(line.to_string());
                continue;
            } else {
                section = 1; // transition vers rooms
            }
        }

        // Section 1 : salles
        if section == 1 {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 3 {
                let name = parts[0].to_string();
                // Le nom ne doit pas commencer par 'L' ou '#'
                if name.starts_with('L') || name.starts_with('#') {
                    section = 2; // transition vers links
                } else if let (Ok(x), Ok(y)) = (parts[1].parse::<i32>(), parts[2].parse::<i32>()) {
                    // Valider l'unicité
                    if room_names.contains(&name) {
                        return Err(format!("ERROR:Room already exists: {}", name));
                    }
                    if room_coords.contains(&(x, y)) {
                        return Err(format!("ERROR:Coordinates not available: ({}, {})", x, y));
                    }

                    let is_start = io == 0;
                    let is_end = io == 1;

                    if is_start {
                        start_count += 1;
                    }
                    if is_end {
                        end_count += 1;
                    }

                    let room = Rc::new(std::cell::RefCell::new(Room::new(
                        name.clone(),
                        Coord { x, y },
                        is_start,
                        is_end,
                    )));
                    room_names.insert(name);
                    room_coords.insert((x, y));
                    farm.add_room(room);

                    io = -1; // reset
                    map_lines.push(line.to_string());
                    continue;
                } else {
                    section = 2; // pas une salle valide → transition
                }
            } else {
                section = 2; // pas une salle → transition
            }
        }

        // Section 2 : liens
        if section == 2 {
            if io >= 0 {
                return Err("ERROR:Improper data type".to_string());
            }

            // Vérifier le format : exactement un '-'
            let dash_count = line.matches('-').count();
            if dash_count != 1 {
                return Err(format!("ERROR:Improper format: {}", line));
            }

            let parts: Vec<&str> = line.split('-').collect();
            if parts.len() != 2 {
                return Err(format!("ERROR:Improper format: {}", line));
            }

            let name1 = parts[0];
            let name2 = parts[1];

            if name1 == name2 {
                return Err("ERROR:Improper linked room".to_string());
            }

            let room1 = farm.find_by_name(name1);
            let room2 = farm.find_by_name(name2);

            match (room1, room2) {
                (Some(r1), Some(r2)) => {
                    // Lien bidirectionnel — éviter les doublons
                    // Note: en C, ft_lstadd ajoute en TÊTE (LIFO).
                    // On utilise insert(0, ...) pour reproduire cet ordre,
                    // car l'ordre des voisins affecte l'exploration BFS.
                    {
                        let mut r1b = r1.borrow_mut();
                        if !r1b.links.iter().any(|r| Rc::ptr_eq(r, &r2)) {
                            r1b.links.insert(0, r2.clone());
                        }
                    }
                    {
                        let mut r2b = r2.borrow_mut();
                        if !r2b.links.iter().any(|r| Rc::ptr_eq(r, &r1)) {
                            r2b.links.insert(0, r1.clone());
                        }
                    }
                    map_lines.push(line.to_string());
                    continue;
                }
                _ => {
                    return Err("ERROR:Improper linked room".to_string());
                }
            }
        }
    }

    // Validation : exactement 1 start et 1 end
    if start_count != 1 || end_count != 1 {
        return Err(format!(
            "ERROR:Improper number of endpoints (start: {}, end: {})",
            start_count, end_count
        ));
    }

    if farm.rooms.is_empty() {
        return Err("ERROR:There are no rooms".to_string());
    }

    Ok(ParseResult { farm, map_lines })
}
