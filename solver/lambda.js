const child_process = require("child_process");

exports.handler = async function (event, _context) {
  const problemId = event.problemId;
  const runId = event.runId;
  const commit = process.env.COMMIT;

  const command = "/code/target/release/lambda";
  const args = [
    "-q",
    "-i",
    `/code/problems/${problemId}.png`,
    "-o",
    "/tmp",
    "-r",
    `${runId}`,
    ...event.args.trim().split(/\s+/),
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

    if (result.status && result.status > 0) {
      console.log("Update Record");
      // Rust と JS どっちからも DynamoDB 書いてて厳しい...
      const {
        DynamoDBClient,
        PutItemCommand,
        UpdateItemCommand,
      } = require("@aws-sdk/client-dynamodb");

      const pk = `R#${runId}`;
      const sk = `S#${runId}`;
      const region = "ap-northeast-1";
      const TableName = "InfraStack-TableCD117FA1-1NAQ40LMS0E1G";
      const client = new DynamoDBClient({ region });

      await client.send(
        new PutItemCommand({
          TableName,
          Item: {
            PK: { S: pk },
            SK: { S: sk },
            Error: { S: result.stderr },
          },
        })
      );
      await client.send(
        new UpdateItemCommand({
          TableName,
          Key: {
            PK: { S: pk },
            SK: { S: pk },
          },
          UpdateExpression: "set #key = :v",
          ExpressionAttributeNames: {
            "#key": `E#${problemId}`,
          },
          ExpressionAttributeValues: {
            ":v": { N: "0" },
          },
        })
      );
    }
  } catch (e) {
    console.log("error:", e);
  }
};
