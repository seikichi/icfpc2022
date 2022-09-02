mod isl;
mod image;
mod ai;
use std::fs;
use std::env;

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
    let solver = ai::GridAI { rows: 10, cols: 10 };
    let program = solver.solve(&img);

    fs::write(output_path, format!("{program}")).unwrap();
}
