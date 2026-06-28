use crate::models::{Farm, Path, RoomRef};
use std::rc::Rc;

/// Simule le déplacement des fourmis tour par tour et produit la sortie.
///
/// Portage fidèle de `send_ants` dans `flow.c` :
/// 1. Extraire les chemins (via `extract_paths`)
/// 2. Calculer la distribution optimale (via `path_config`)
/// 3. Boucle de simulation :
///    a. `move_ants` : fait avancer toutes les fourmis sur tous les chemins
///    b. `select_paths` : injecte de nouvelles fourmis depuis start
///    c. Afficher le tour
/// 4. Condition de fin : toutes les fourmis sont arrivées à end
pub fn send_ants(farm: &mut Farm, output: &mut String) {
    let total_ants = farm.ants;
    if total_ants <= 0 {
        return;
    }

    // Extraire et optimiser les chemins
    let mut paths = crate::flow::extract_paths(farm);
    if paths.is_empty() {
        return;
    }

    crate::path_config::path_config(&mut paths, total_ants);

    // Trouver end
    let end = match farm.find_end() {
        Some(e) => e,
        None => return,
    };

    // Boucle de simulation
    while end.borrow().ant < total_ants {
        let mut moved = false;

        // Phase 1 : faire avancer les fourmis existantes
        moved |= move_ants(&paths, output);

        // Phase 2 : injecter de nouvelles fourmis depuis start
        moved |= select_paths(&mut paths, farm, total_ants, output);

        output.push('\n');

        if !moved {
            break; // plus aucun mouvement possible
        }
    }
}

/// Fait avancer toutes les fourmis sur tous les chemins.
///
/// Portage de `move_ants` : appelle `push_ants` pour chaque chemin.
fn move_ants(paths: &[Path], output: &mut String) -> bool {
    let mut entro = false;
    for path in paths {
        if push_ants(path, output) {
            entro = true;
        }
    }
    entro
}

/// Fait avancer les fourmis sur UN chemin (shift forward).
///
/// Portage fidèle de `push_ants` dans `flow.c` :
/// Parcourt le chemin du début vers la fin via les rooms du Vec.
/// Chaque salle reçoit l'ant de la salle précédente (shift).
/// La première salle reçoit 0 (sera remplie par `select_paths`).
/// Si une fourmi arrive à end, end.ant est incrémenté.
fn push_ants(path: &Path, output: &mut String) -> bool {
    let mut prev_ant: i32 = 0;
    let mut entro = false;

    for room_ref in &path.rooms {
        let (is_end, name, ant) = {
            let r = room_ref.borrow();
            (r.end, r.name.clone(), r.ant)
        };

        if is_end {
            if prev_ant > 0 {
                room_ref.borrow_mut().ant += 1;
                output.push_str(&format!("L{}-{} ", prev_ant, name));
                entro = true;
            }
            break;
        }

        // Sauvegarder l'ant actuel, puis setter la précédente
        let aux_ant = ant;
        room_ref.borrow_mut().ant = prev_ant;

        // Si la nouvelle ant est non-nulle, l'afficher
        if prev_ant > 0 {
            output.push_str(&format!("L{}-{} ", prev_ant, name));
            entro = true;
        }

        prev_ant = aux_ant;
    }

    entro
}

/// Injecte de nouvelles fourmis depuis le start.
///
/// Portage fidèle de `select_paths` dans `flow.c` :
/// Pour chaque chemin (trié par longueur), si `min_ants > 0` et qu'il reste
/// des fourmis à envoyer :
/// - Décrémente `min_ants`
/// - Place une nouvelle fourmi (ID = total_ants - farm.ants + 1) dans la première salle
/// - Décrémente `farm.ants`
fn select_paths(paths: &mut [Path], farm: &mut Farm, total_ants: i32, output: &mut String) -> bool {
    let mut entro = false;

    for path in paths.iter_mut() {
        if farm.ants <= 0 {
            break;
        }
        if path.min_ants <= 0 {
            continue;
        }

        // Vérifier que la première salle est vide
        if path.rooms.is_empty() {
            continue;
        }
        let first_room = &path.rooms[0];
        let first_ant = first_room.borrow().ant;
        if first_ant != 0 {
            continue; // la première salle est occupée
        }

        // Injecter une nouvelle fourmi
        path.min_ants -= 1;
        let ant_id = total_ants - farm.ants + 1;
        let name = {
            let mut r = first_room.borrow_mut();
            r.ant = ant_id;
            r.name.clone()
        };
        output.push_str(&format!("L{}-{} ", ant_id, name));
        farm.ants -= 1;
        entro = true;
    }

    entro
}
