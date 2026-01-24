use clap::{Arg, Command};
use electro_solve_core::{graph::CircuitGraph, units::AngularFrequency};
use std::fs;

fn main() {
    let matches = Command::new("ElectroSolve")
        .version("0.1.0")
        .about("Circuit solver and reducer")
        .subcommand(
            Command::new("solve")
                .about("Solve a circuit from a file")
                .arg(Arg::new("file").required(true).help("Input circuit file"))
                .arg(Arg::new("frequency").required(true).help("Frequency in Hz")),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("solve", sub_matches)) => {
            let file_path = sub_matches.get_one::<String>("file").unwrap();
            let freq_hz = sub_matches.get_one::<String>("frequency").unwrap();

            let freq_hz: f64 = freq_hz.parse().expect("Frequency must be a number");
            let omega = AngularFrequency::new(freq_hz * 2.0 * std::f64::consts::PI).unwrap();

            // Read and parse circuit file
            let content = fs::read_to_string(file_path).expect("Failed to read file");

            // Create graph and solve
            let mut graph = CircuitGraph::new();
            // TODO: Convert parsed circuit to graph

            println!("Circuit parsed successfully!");
            println!("Frequency: {} Hz", freq_hz);
            println!("Omega: {} rad/s", omega.get());
        }
        _ => {
            eprintln!("No subcommand provided. Use 'solve' to solve a circuit.");
            std::process::exit(1);
        }
    }
}
