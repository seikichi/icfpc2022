mod ai;
mod image;
mod isl;
mod refine_ai;
mod simulator;

use std::fs;
use std::path::PathBuf;
use std::process::exit;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "solver", about = "A solver of ICFPC 2022 problems")]
struct Opt {
    #[structopt(parse(from_os_str))]
    input_path: PathBuf,

    #[structopt(parse(from_os_str))]
    output_path: PathBuf,
}

fn main() {
    let opt = Opt::from_args();

    let img = image::open(opt.input_path);
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

    fs::write(opt.output_path, format!("{program}")).unwrap();
}
