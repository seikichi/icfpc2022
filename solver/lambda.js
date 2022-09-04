const child_process = require("child_process");

exports.handler = async function (event, _context) {
  const problemId = event.problemId;
  const runId = event.runId;
  const commit = process.env.COMMIT;

  const command = "/code/target/release/icfpc2022";
  const args = [
    ...event.args.trim().split(/\s+/),
    "-i",
    `/code/problems/${problemId}.png`,
    "-o",
    "/tmp",
    "-r",
    `${runId}`,
  ];

  console.log("args", args);

  try {
    const result = child_process.spawnSync(command, args, {
      env: {
        ...event.env,
        ...process.env,
        RUST_BACKTRACE: "1",
        COMMIT: commit,
      },
      encoding: "utf-8",
    });
    console.log("stdout:", result.stdout);
    console.log("stderr:", result.stderr);
  } catch (e) {
    console.log("error:", e);
  }
};
