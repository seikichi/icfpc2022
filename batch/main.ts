import { DynamoDBClient, QueryCommand } from "@aws-sdk/client-dynamodb";
import FormData from "form-data";
import fetch from "node-fetch";

// .env に書け
const region = "ap-northeast-1";
const TableName = "InfraStack-TableCD117FA1-1NAQ40LMS0E1G";
// const credentials =
//   process.env.AWS_ACCESS_KEY_ID && process.env.AWS_SECRET_ACCESS_KEY
//     ? {
//         accessKeyId: process.env.AWS_ACCESS_KEY_ID!,
//         secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY!,
//       }
//     : undefined;

const TOKEN = process.env.TOKEN!;
const client = new DynamoDBClient({ region });

const MAX_PROBLEM_ID = 40;

(async () => {
  for (let problemId = 1; problemId <= MAX_PROBLEM_ID; problemId++) {
    const { Items: items } = await client.send(
      new QueryCommand({
        TableName,
        IndexName: "GSI1",
        KeyConditionExpression: "GSI1PK = :pk",
        ExpressionAttributeValues: {
          ":pk": { S: `P#${problemId}` },
        },
        ScanIndexForward: true,
      })
    );

    if (!items) {
      console.log("No solution:", problemId);
      continue;
    }

    const item = items[0];
    const pk = item["PK"]["S"]!;
    const sk = item["SK"]["S"]!;

    const runId = pk.split("#")[1];
    const commit = item["Commit"]["S"]!;
    const ai = item["AI"]["S"]!;
    const score = parseInt(item["GSI1SK"]["N"]!, 10);

    console.log(`Problem ${problemId}: ${score} (${ai}, ${commit}, ${runId})`);

    const url = `https://d30a5x02adw8tj.cloudfront.net/${runId}/${problemId}.isl`;
    const isl = await fetch(url).then((r) => r.text());

    const form = new FormData();
    form.append("file", Buffer.from(isl), { filename: `${problemId}.isl` });
    const res = await fetch(`https://robovinci.xyz/api/problems/${problemId}`, {
      method: "POST",
      headers: { Authorization: `Bearer ${TOKEN}` },
      body: form,
    });

    const { submission_id: submissionId } = await res.json();
    for (let i = 0; i < Infinity; i++) {
      const res = await fetch(
        `https://robovinci.xyz/api/submissions/${submissionId}`,
        {
          headers: { Authorization: `Bearer ${TOKEN}` },
        }
      );
      const { status, cost } = await res.json();
      if (status === "PROCESSING" || status === "QUEUED") {
        await new Promise((r) => setTimeout(r, 1000 * i));
        continue;
      }
      if (cost !== score) {
        console.log(`WRONG SCORE: expect ${score}, but ${cost} given`);
      }
      break;
    }
  }
})();
