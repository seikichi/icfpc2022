mod ai;
mod db;
mod image;
mod initial_config;
mod isl;
mod simulator;

use anyhow::bail;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
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

    // Lambda で同パラメーターで複数の問題に対して並列実行する時、
    // 最初に適当な run-id を採番して、それがここに渡ってくる (妄想)
    #[structopt(short = "r", long = "run-id")]
    run_id: Option<String>,

    #[structopt(long = "refine-iters", default_value = "50000")]
    refine_iters: usize,

    #[structopt(long = "annealing-seconds", default_value = "10")]
    annealing_seconds: u64,

    #[structopt(long = "dp-divide-num", default_value = "8")]
    dp_divide_num: usize,

    #[structopt(long = "dp-color-num", default_value = "10")]
    dp_color_num: usize,
}

fn parse_ai_string(
    ai_str: &str,
    opt: &Opt,
) -> anyhow::Result<(Box<dyn HeadAI>, Vec<Box<dyn ChainedAI>>)> {
    let parts = ai_str.split(',').collect::<Vec<_>>();
    let head_ai: Box<dyn ai::HeadAI> = match parts[0] {
        "OneColor" => Box::new(ai::OneColorAI {}),
        "Grid" => Box::new(ai::GridAI { rows: 4, cols: 4 }),
        "Cross" => Box::new(ai::CrossAI { size: 3 }),
        "DP" => Box::new(ai::DpAI::new(opt.dp_divide_num, opt.dp_color_num)),
        // "Merge" => Box::new(ai::MergeAI::new()),
        "ChangeColor" => Box::new(ai::ChangeColorAI {}),
        x => bail!("'{x}' is not a HeadAI"),
    };
    let mut chained_ais = vec![];
    for name in &parts[1..] {
        let chained_ai: Box<dyn ai::ChainedAI> = match *name {
            "Refine" => Box::new(ai::RefineAi {
                n_iters: opt.refine_iters,
            }),
            "Annealing" => Box::new(ai::AnnealingAI {
                time_limit: Duration::from_secs(opt.annealing_seconds),
            }),
            x => bail!("'{x}' is not a ChainedAI"),
        };
        chained_ais.push(chained_ai);
    }
    Ok((head_ai, chained_ais))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();

    let (mut head_ai, chained_ais) = parse_ai_string(&opt.ai, &opt)?;

    if !opt.output_dir.is_dir() {
        bail!("'{}' is not a directory", opt.output_dir.to_string_lossy());
    }

    let problem_id = opt
        .input_path
        .file_stem()
        .expect("--input should be a file name.")
        .to_string_lossy()
        .to_string();

    let mut score_history = vec![];

    let img = image::open(opt.input_path.clone())?;

    let initial_state = initial_config::load_initial_state(
        &opt.input_path
            .parent()
            .unwrap()
            .join(format!("{}.initial.json", problem_id))
            .to_str()
            .unwrap(),
        &img,
    );

    let mut program = head_ai.solve(&img, &initial_state);

    score_history.push(simulator::calc_score(&program, &img, &initial_state)?);

    for mut chained_ai in chained_ais {
        score_history.push(simulator::calc_score(&program, &img, &initial_state)?);
        program = chained_ai.solve(&img, &initial_state, &program);
    }

    println!("Score History:");
    for (i, score) in score_history.iter().enumerate() {
        println!("    {i}: {score}")
    }

    let score = simulator::calc_score(&program, &img, &initial_state)?;
    let state = simulator::simulate_all(&program, &initial_state)?;
    let output_image = simulator::rasterize_state(&state, img.width(), img.height());

    let output_filename = opt.output_dir.join(problem_id.clone() + ".isl");
    println!("output ISL to: {}", output_filename.to_string_lossy());
    fs::write(output_filename, format!("{program}"))?;

    let output_image_filename = opt.output_dir.join(problem_id.clone() + ".png");
    println!("output PNG to: {}", output_image_filename.to_string_lossy());
    output_image.save(output_image_filename.clone())?;

    if let Some(run_id) = opt.run_id {
        let commit = env::var("COMMIT")?;
        db::save(
            &run_id,
            &problem_id,
            &program,
            score,
            &output_image_filename.to_string_lossy(),
            &opt.ai,
            &commit,
        )
        .await?;
    }

    Ok(())
}
