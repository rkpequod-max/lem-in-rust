use crate::bfs::bfs_find_path;
use crate::flow::update_flow;
use crate::simulation::send_ants;
use crate::models::Farm;

/// Point d'entrée de la résolution.
///
/// Portage fidèle de `solve` dans `solve.c` :
/// 1. Valider qu'il y a exactement 1 start et 1 end
/// 2. Boucle de découverte de chemins :
///    - BFS pour trouver un chemin
///    - update_flow pour marquer le chemin
///    - Répéter jusqu'à ce qu'aucun chemin ne soit trouvé (ou ants <= 1)
/// 3. send_ants pour la simulation
pub fn solve(farm: &mut Farm, output: &mut String) -> bool {
    let start = match farm.find_start() {
        Some(s) => s,
        None => {
            output.push_str("ERROR:No start found\n");
            return false;
        }
    };

    let end = match farm.find_end() {
        Some(e) => e,
        None => {
            output.push_str("ERROR:No end found\n");
            return false;
        }
    };

    // Boucle de découverte de chemins
    loop {
        if let Some(path) = bfs_find_path(&start, &end) {
            update_flow(&path);
        } else {
            break; // plus de chemin trouvé
        }

        if farm.ants <= 1 {
            break; // pas besoin de multiple paths pour 1 fourmi
        }
    }

    // Lancer la simulation
    send_ants(farm, output);
    true
}
