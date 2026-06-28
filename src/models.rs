use std::cell::RefCell;
use std::rc::Rc;

/// Coordonnées 2D d'une salle dans la ferme.
#[derive(Clone, Debug, Default)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

/// Type alias pour une référence partagée et mutable vers une salle.
/// `Rc<RefCell<Room>>` permet le partage de propriété (graphe cyclique)
/// et la mutation intérieure (modification de `ant`, `f_to`, `f_fm`).
pub type RoomRef = Rc<RefCell<Room>>;

/// Représente une salle dans le graphe de la ferme.
///
/// Champs de flux (`f_to` / `f_fm`) : forment le graphe résiduel
/// utilisé par l'algorithme d'Edmonds-Karp. Après un BFS réussi,
/// `update_flow` marque le chemin en setting ces pointeurs.
pub struct Room {
    pub name: String,
    pub coord: Coord,
    /// ID de la fourmi actuellement dans la salle (0 si vide).
    /// Pour `end`, c'est un compteur cumulatif d'arrivées.
    pub ant: i32,
    pub start: bool,
    pub end: bool,
    /// Salles voisines (graphe d'adjacence).
    pub links: Vec<RoomRef>,
    /// Salle suivante dans le flux de fourmis (forward).
    pub f_to: Option<RoomRef>,
    /// Salle précédente dans le flux (from / backward).
    pub f_fm: Option<RoomRef>,
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

/// Conteneur principal représentant toute la ferme.
pub struct Farm {
    /// Nombre total de fourmis à déplacer.
    pub ants: i32,
    /// Toutes les salles (Vec contigu pour bonne localité cache).
    pub rooms: Vec<RoomRef>,
    /// Lookup O(1) par nom de salle.
    pub name_map: std::collections::HashMap<String, RoomRef>,
}

impl Farm {
    pub fn new() -> Self {
        Farm {
            ants: 0,
            rooms: Vec::new(),
            name_map: std::collections::HashMap::new(),
        }
    }

    /// Ajoute une salle à la ferme.
    /// Note: en C, push_room ajoute en TÊTE (LIFO). On reproduit cet ordre
    /// car l'ordre des salles affecte find_start/find_end et l'exploration BFS.
    pub fn add_room(&mut self, room: RoomRef) {
        let name = room.borrow().name.clone();
        self.name_map.insert(name, room.clone());
        self.rooms.insert(0, room);
    }

    /// Trouve la salle de départ.
    pub fn find_start(&self) -> Option<RoomRef> {
        self.rooms.iter().find(|r| r.borrow().start).cloned()
    }

    /// Trouve la salle d'arrivée.
    pub fn find_end(&self) -> Option<RoomRef> {
        self.rooms.iter().find(|r| r.borrow().end).cloned()
    }

    /// Trouve une salle par son nom.
    pub fn find_by_name(&self, name: &str) -> Option<RoomRef> {
        self.name_map.get(name).cloned()
    }
}

/// Un chemin trouvé par BFS, utilisé pour la simulation.
/// `rooms` ne contient PAS le start — uniquement [première_salleAprèsStart, ..., end].
pub struct Path {
    pub rooms: Vec<RoomRef>,
    pub send_ants: i32,  // temporaire, utilisé pendant path_config
    pub min_ants: i32,   // allocation finale calculée par path_config
}

impl Path {
    pub fn path_size(&self) -> i32 {
        self.rooms.len() as i32
    }
}
