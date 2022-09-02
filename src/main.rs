mod ai;
mod image;
mod isl;
mod simulator;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: solver input.png output.isl");
        return;
    }
    let input_path = &args[1];
    let output_path = &args[2];

    let img = image::open(input_path);
    // let solver = ai::OneColorAI {};
    // let solver = ai::GridAI { rows: 4, cols: 4 };
    let solver = ai::CrossAI { size: 3 };
    let program = solver.solve(&img);

    fs::write(output_path, format!("{program}")).unwrap();
}
