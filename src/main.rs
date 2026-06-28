use lem_in_rust::parse_and_solve;
use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let output = parse_and_solve(&input);
    print!("{}", output);
}
