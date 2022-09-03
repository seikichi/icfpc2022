const child_process = require("child_process");

exports.handler = async function (event, context) {
  const problemId = event.problemId;
  const ai = event.ai;
  const runId = event.runId;
  const commit = process.env.COMMIT;

  const command = `/code/target/release/icfpc2022 -a ${ai} -i /code/problems/${problemId}.png -o /tmp -r ${runId}`;
  child_process.execSync(command, {
    env: {
      ...event.env,
      RUST_BACKTRACE: "1",
      COMMIT: commit,
    },
  });
};
