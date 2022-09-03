mod ai;
mod db;
mod dp_ai;
mod image;
mod isl;
mod refine_ai;
mod simulator;

use anyhow::bail;
use std::path::{Path, PathBuf};
use std::{ffi::OsStr, fs};
use structopt::StructOpt;

use crate::ai::{ChainedAI, HeadAI};

#[derive(Debug, StructOpt)]
#[structopt(name = "solver", about = "A solver of ICFPC 2022 problems")]
struct Opt {
    #[structopt(
        short = "a",
        long = "ai",
        help = "comma separated list of AIs, e.g. 'Cross,Refine'"
    )]
    ai: String,

    #[structopt(short = "i", long = "input", parse(from_os_str))]
    input_path: PathBuf,

    #[structopt(short = "o", long = "output-dir", parse(from_os_str))]
    output_dir: PathBuf,
}

fn parse_ai_string(ai_str: &str) -> anyhow::Result<(Box<dyn HeadAI>, Vec<Box<dyn ChainedAI>>)> {
    let parts = ai_str.split(',').collect::<Vec<_>>();
    let head_ai: Box<dyn ai::HeadAI> = match parts[0] {
        "OneColor" => Box::new(ai::OneColorAI {}),
        "Grid" => Box::new(ai::GridAI { rows: 4, cols: 4 }),
        "Cross" => Box::new(ai::CrossAI { size: 3 }),
        x => bail!("'{x}' is not a HeadAI"),
    };
    let mut chained_ais = vec![];
    for name in &parts[1..] {
        let chained_ai: Box<dyn ai::ChainedAI> = match *name {
            "Refine" => Box::new(refine_ai::RefineAi {}),
            x => bail!("'{x}' is not a ChainedAI"),
        };
        chained_ais.push(chained_ai);
    }
    Ok((head_ai, chained_ais))
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();

    let (mut head_ai, chained_ais) = parse_ai_string(&opt.ai)?;

    if !opt.output_dir.is_dir() {
        bail!("'{}' is not a directory", opt.output_dir.to_string_lossy());
    }

    let problem_id = opt
        .input_path
        .file_stem()
        .expect("--input should be a file name.")
        .to_string_lossy()
        .to_string();

    let img = image::open(opt.input_path)?;
    let mut program = head_ai.solve(&img);

    for mut chained_ai in chained_ais {
        program = chained_ai.solve(&img, &program);
    }

    let score = simulator::calc_score(&program, &img)?;

    println!("score: {}", score);
    let state = simulator::simulate_all(&program, &img)?;
    let output_image = simulator::rasterize_state(&state, img.width(), img.height());
    output_image.save("result.png")?;

    let output_basename = problem_id + ".isl";
    let output_filename = opt.output_dir.join(output_basename);
    println!("output to: {}", output_filename.to_string_lossy());
    fs::write(output_filename, format!("{program}"))?;

    Ok(())
}
