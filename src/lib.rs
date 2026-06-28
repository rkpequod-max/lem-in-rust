pub mod models;
pub mod parser;
pub mod bfs;
pub mod flow;
pub mod path_config;
pub mod simulation;
pub mod solver;

use wasm_bindgen::prelude::*;

/// Point d'entrée WASM : parse l'input et retourne la sortie complète.
#[wasm_bindgen]
pub fn parse_and_solve(input: &str) -> String {
    console_error_panic_hook::set_once();

    match parser::parse_input(input) {
        Ok(result) => {
            let mut farm = result.farm;
            let mut output = String::new();

            // Echo de la map originale
            for line in &result.map_lines {
                output.push_str(line);
                output.push('\n');
            }
            output.push('\n');

            // Résoudre
            solver::solve(&mut farm, &mut output);

            output
        }
        Err(e) => e,
    }
}

#[wasm_bindgen(start)]
pub fn main_web() {
    console_error_panic_hook::set_once();
}
