use crate::models::{Path, RoomRef, Farm};
use std::rc::Rc;

/// Met à jour le flux après qu'un BFS a trouvé un chemin.
///
/// Portage fidèle de `update_flow` dans `flow.c` :
/// 1. Backtrack depuis end vers start via les parent pointers
/// 2. Pour chaque paire (prv, r) :
///    - Si pas de conflit (r.f_to != prv ET prv.f_fm != r) :
///      - Set r.f_fm = prv (si r n'est pas start, ou r est end et prv est start)
///      - Set prv.f_to = r (si prv n'est pas start)
///    - Annuler les conflits :
///      - Si r.f_to == prv → r.f_to = None
///      - Si prv.f_fm == r → prv.f_fm = None
///
/// Le chemin `path` contient [start, ..., end].
pub fn update_flow(path: &[RoomRef]) {
    // Backtrack depuis end (dernier élément) vers start (premier élément)
    for i in (1..path.len()).rev() {
        let r = &path[i];
        let prv = &path[i - 1];

        // Cloner les valeurs nécessaires pour éviter les borrow conflicts
        let (r_f_to, r_is_start, r_is_end) = {
            let rb = r.borrow();
            (rb.f_to.clone(), rb.start, rb.end)
        };
        let (prv_f_fm, prv_is_start) = {
            let pb = prv.borrow();
            (pb.f_fm.clone(), pb.start)
        };

        // Vérifier les conflits
        let r_f_to_is_prv = r_f_to.as_ref().map_or(false, |f| Rc::ptr_eq(f, prv));
        let prv_f_fm_is_r = prv_f_fm.as_ref().map_or(false, |f| Rc::ptr_eq(f, r));

        // Si pas de conflit : établir le flux
        if !r_f_to_is_prv && !prv_f_fm_is_r {
            // Set r.f_fm = prv (si r n'est pas start, ou r est end et prv est start)
            if (!r_is_start && !r_is_end) || (r_is_end && prv_is_start) {
                r.borrow_mut().f_fm = Some(prv.clone());
            }
            // Set prv.f_to = r (si prv n'est pas start)
            if !prv_is_start {
                prv.borrow_mut().f_to = Some(r.clone());
            }
        }

        // Annuler les conflits
        if r_f_to_is_prv {
            r.borrow_mut().f_to = None;
        }
        if prv_f_fm_is_r {
            prv.borrow_mut().f_fm = None;
        }
    }
}

/// Extrait tous les chemins valides depuis le start en suivant les `f_to`.
///
/// Portage de `path_list` dans `flow_treatment.c` :
/// Pour chaque voisin du start, suit `f_to` jusqu'à end.
/// Si le chemin atteint end, c'est un chemin valide.
///
/// Retourne les chemins triés par longueur (plus court en premier).
pub fn extract_paths(farm: &Farm) -> Vec<Path> {
    let start = match farm.find_start() {
        Some(s) => s,
        None => return Vec::new(),
    };

    let mut paths = Vec::new();

    // Cloner les liens du start
    let start_links = start.borrow().links.clone();

    for first_room in &start_links {
        // Suivre f_to jusqu'à end
        let mut path_rooms = Vec::new();
        let mut current = first_room.clone();
        let mut valid = false;

        loop {
            let curr = current.borrow();
            if curr.end {
                path_rooms.push(current.clone());
                valid = true;
                break;
            }
            path_rooms.push(current.clone());
            match &curr.f_to {
                Some(next) => {
                    let next = next.clone();
                    drop(curr);
                    current = next;
                }
                None => break, // pas de f_to → chemin invalide
            }
        }

        if valid {
            paths.push(Path {
                rooms: path_rooms,
                send_ants: 0,
                min_ants: 0,
            });
        }
    }

    // Trier par longueur (plus court en premier) — équivalent à `insertio` en C
    paths.sort_by_key(|p| p.path_size());

    paths
}
