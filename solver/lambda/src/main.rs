use std::env;

extern crate core;

mod db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let output = core::run()?;

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
        )
        .await?;
    }

    Ok(())
}
