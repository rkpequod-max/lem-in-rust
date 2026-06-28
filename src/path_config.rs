use crate::models::Path;

/// Calcule la distribution optimale des fourmis sur les chemins.
///
/// Portage fidèle de `path_config`, `condemn_path`, `populate_path` et `save_min`
/// dans `path_computation.c`.
///
/// Algorithme :
/// 1. Pour chaque sous-ensemble de chemins (1 chemin, 2 chemins, ..., N chemins) :
///    - Calculer le nombre de tours nécessaires
///    - Garder la configuration avec le minimum de tours
/// 2. Stocker le résultat dans `min_ants` de chaque chemin
///
/// Formule clé (dans `populate_path`) :
/// `send_ants[i+1] = send_ants[i] - abs(path_size[i+1] - path_size[i])`
/// Les chemins plus longs reçoivent moins de fourmis car ils "occupent" le pipeline plus longtemps.
pub fn path_config(paths: &mut Vec<Path>, ants: i32) {
    if paths.is_empty() || ants <= 0 {
        return;
    }

    // Les chemins sont déjà triés par longueur (plus court en premier) par extract_paths
    let mut min_turns = i32::MAX;
    let mut best_send_ants: Vec<i32> = vec![0; paths.len()];

    for i in 0..paths.len() {
        let path_refs: Vec<&Path> = paths[..=i].iter().collect();
        let (turns, send_ants) = condemn_path(&path_refs, ants);

        if turns < min_turns {
            min_turns = turns;
            // Sauvegarder send_ants dans best_send_ants
            for j in 0..=i {
                best_send_ants[j] = send_ants[j];
            }
            for j in (i + 1)..paths.len() {
                best_send_ants[j] = 0;
            }
        }

        // Si send_ants[i] == 0, ce chemin ne reçoit pas de fourmis → pas la peine d'en ajouter plus
        if send_ants[i] <= 0 {
            break;
        }
    }

    // Appliquer les meilleures allocations
    for (i, p) in paths.iter_mut().enumerate() {
        p.min_ants = best_send_ants[i];
    }
}

/// Calcule le nombre de tours nécessaires pour `ants` fourmis
/// en utilisant les chemins `paths[0..=last]`.
///
/// Portage de `condemn_path` : incrémente `population` jusqu'à ce que
/// `reach >= ants`, puis retourne `population + path_size[0]`.
fn condemn_path(paths: &[&Path], ants: i32) -> (i32, Vec<i32>) {
    let n = paths.len();
    let mut send_ants = vec![0i32; n];
    let mut population = 0;
    let mut reach;

    loop {
        population += 1;
        reach = populate_path(population, paths, &mut send_ants);
        if reach >= ants {
            break;
        }
    }

    let turns = population + paths[0].path_size();
    (turns, send_ants)
}

/// Distribue `first` fourmis sur les chemins et calcule le total (reach).
///
/// Portage de `populate_path` :
/// `send_ants[0] = first`
/// `send_ants[i+1] = send_ants[i] - abs(path_size[i+1] - path_size[i])`
fn populate_path(first: i32, paths: &[&Path], send_ants: &mut [i32]) -> i32 {
    send_ants[0] = first;
    for i in 0..paths.len() - 1 {
        let diff = (paths[i + 1].path_size() - paths[i].path_size()).abs();
        send_ants[i + 1] = (send_ants[i] - diff).max(0);
    }
    send_ants.iter().sum()
}
