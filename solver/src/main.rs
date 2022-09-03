mod ai;
mod db;
mod image;
mod isl;
mod refine_ai;
mod simulator;

use anyhow::bail;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

use crate::ai::{ChainedAI, HeadAI};

#[derive(Debug, StructOpt)]
#[structopt(name = "solver", about = "A solver of ICFPC 2022 problems")]
struct Opt {
    ai: String,

    #[structopt(parse(from_os_str))]
    input_path: PathBuf,

    #[structopt(parse(from_os_str))]
    output_path: PathBuf,
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();

    let (mut head_ai, chained_ais) = parse_ai_string(&opt.ai)?;

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

    fs::write(opt.output_path, format!("{program}"))?;

    // db::save(
    //     "482eb33b-b510-4e06-bec9-9222159deaee",
    //     1,
    //     &program,
    //     score,
    //     "result.png",
    // )
    // .await?;

    Ok(())
}
