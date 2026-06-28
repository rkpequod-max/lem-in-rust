use crate::models::RoomRef;
use std::rc::Rc;

/// Vérifie si l'arête `from → to` fait partie du flux courant.
///
/// Logique fidèle à la version C (`flow_to` dans `bfs.c`) :
/// - Si `from` est le start : l'arête est "utilisée" si `to.f_fm` pointe vers `from`.
/// - Sinon : l'arête est "utilisée" si `from.f_to` pointe vers `to`.
fn flow_to(from: &RoomRef, to: &RoomRef) -> bool {
    let from_is_start = from.borrow().start;
    if from_is_start {
        let to_f_fm = to.borrow().f_fm.clone();
        to_f_fm.map_or(false, |f| Rc::ptr_eq(&f, from))
    } else {
        let from_f_to = from.borrow().f_to.clone();
        from_f_to.map_or(false, |f| Rc::ptr_eq(&f, to))
    }
}

/// Reconstruit le chemin depuis end jusqu'à start en suivant les parents.
fn reconstruct_path(
    end: &RoomRef,
    parent: &std::collections::HashMap<usize, RoomRef>,
    start: &RoomRef,
) -> Vec<RoomRef> {
    let mut path = vec![end.clone()];
    let start_ptr = Rc::as_ptr(start) as usize;
    let mut ptr = Rc::as_ptr(end) as usize;

    while ptr != start_ptr {
        match parent.get(&ptr) {
            Some(p) => {
                path.push(p.clone());
                ptr = Rc::as_ptr(p) as usize;
            }
            None => break,
        }
    }

    path.reverse();
    path
}

/// BFS itératif (avec VecDeque) pour trouver un chemin de start à end.
///
/// Implémente la logique de graphe résiduel d'Edmonds-Karp :
/// - `flow_to` : skip les arêtes déjà dans le flux
/// - `undo` : si le noeud courant a un `f_to` qui ne pointe pas vers son parent,
///   on est en mode "undo" — seul le voisin `f_fm` est autorisé (reverse edge)
pub fn bfs_find_path(start: &RoomRef, end: &RoomRef) -> Option<Vec<RoomRef>> {
    use std::collections::{HashMap, HashSet, VecDeque};

    let start_ptr = Rc::as_ptr(start) as usize;

    let mut visited: HashSet<usize> = HashSet::new();
    let mut parent: HashMap<usize, RoomRef> = HashMap::new();
    let mut queue: VecDeque<RoomRef> = VecDeque::new();

    visited.insert(start_ptr);
    queue.push_back(start.clone());

    while let Some(current) = queue.pop_front() {
        let current_ptr = Rc::as_ptr(&current) as usize;

        // Vérifier si on a atteint end
        if Rc::ptr_eq(&current, end) {
            return Some(reconstruct_path(&current, &parent, start));
        }

        // Déterminer le mode "undo"
        let undo = {
            let curr = current.borrow();
            if let Some(parent_r) = parent.get(&current_ptr) {
                if let Some(ref f_to) = curr.f_to {
                    !Rc::ptr_eq(f_to, parent_r)
                } else {
                    false
                }
            } else {
                false
            }
        };

        let f_fm = current.borrow().f_fm.clone();
        let neighbors = current.borrow().links.clone();

        for neighbor in neighbors {
            let neighbor_ptr = Rc::as_ptr(&neighbor) as usize;

            if visited.contains(&neighbor_ptr) {
                continue;
            }
            if flow_to(&current, &neighbor) {
                continue;
            }

            // Si mode undo : seul le voisin f_fm est autorisé
            if undo {
                let allow = f_fm.as_ref().map_or(false, |f| Rc::ptr_eq(f, &neighbor));
                if !allow {
                    continue;
                }
            }

            visited.insert(neighbor_ptr);
            parent.insert(neighbor_ptr, current.clone());
            queue.push_back(neighbor.clone());

            // Vérifier si ce voisin est end (après insertion, comme en C)
            if neighbor.borrow().end {
                return Some(reconstruct_path(&neighbor, &parent, start));
            }
        }
    }

    None
}
