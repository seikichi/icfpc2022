mod ai;
mod image;
mod isl;
mod refine_ai;
mod simulator;

use std::env;
use std::fs;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: solver input.png output.isl");
        exit(2);
    }
    let input_path = &args[1];
    let output_path = &args[2];

    let img = image::open(input_path);
    // let solver = ai::OneColorAI {};
    // let solver = ai::GridAI { rows: 4, cols: 4 };
    let solver = ai::CrossAI { size: 3 };
    let program = solver.solve(&img);

    let solver2 = refine_ai::RefineAi {};
    let program = solver2.solve(&program, &img);

    if let Some(score) = simulator::calc_score(&program, &img) {
        println!("score: {}", score);
        let state = simulator::simulate_all(&program, &img).unwrap();
        let output_image = simulator::rasterize_state(&state, img.width(), img.height());
        output_image.save("result.png");
    } else {
        eprintln!("score: Invalid program");
        exit(1);
    }

    fs::write(output_path, format!("{program}")).unwrap();
}
