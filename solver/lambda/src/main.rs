use std::env;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

extern crate core;

mod db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let start = Instant::now();

    // run!!!!
    let output = core::run()?;

    let elapsed = start.elapsed();
    println!("{:?} sec elapsed", elapsed.as_secs());

    let now = SystemTime::now();
    let unixtime = now.duration_since(UNIX_EPOCH).expect("back to the future");
    println!("unixtime: {:?}", unixtime.as_secs());

    if let Some(run_id) = output.run_id {
        let commit = env::var("COMMIT")?;
        db::save(
            &run_id,
            &output.problem_id,
            &output.program,
            output.score,
            &output.output_image_filename,
            &output.ai,
            &commit,
            elapsed.as_secs(),
            unixtime.as_secs(),
        )
        .await?;
    }

    Ok(())
}
