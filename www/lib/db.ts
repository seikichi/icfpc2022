import { DynamoDBClient, QueryCommand } from "@aws-sdk/client-dynamodb";

export interface Run {
  id: string;
  time: number;
  args: string;
  problems: number;
  score: number;
}

export interface RunResult {
  id: string;
  time: number;
  args: string;
  score: number;

  results: {
    commit: string;
    problemId: number;
    score: number;
  }[];
}

export interface Solution {
  runId: string;
  commit: string;
  problemId: number;
  score: number;
  ai: string;
}

// .env に書け
const region = "ap-northeast-1";
const TableName = "InfraStack-TableCD117FA1-1NAQ40LMS0E1G";
const credentials =
  process.env.AWS_ACCESS_KEY_ID && process.env.AWS_SECRET_ACCESS_KEY
    ? {
        accessKeyId: process.env.AWS_ACCESS_KEY_ID!,
        secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY!,
      }
    : undefined;

const client = new DynamoDBClient({ region, credentials });

export async function fetchSolutionList(id: string): Promise<Solution[]> {
  const { Items: items } = await client.send(
    new QueryCommand({
      TableName,
      IndexName: "GSI1",
      KeyConditionExpression: "GSI1PK = :pk",
      ExpressionAttributeValues: {
        ":pk": { S: `P#${id}` },
      },
      ScanIndexForward: true,
    })
  );
  if (!items) {
    return [];
  }

  const results: Solution[] = [];
  for (const item of items) {
    const pk = item["PK"]["S"]!;
    const sk = item["SK"]["S"]!;

    const runId = pk.split("#")[1];
    const commit = item["Commit"]["S"]!;
    const ai = item["AI"]["S"]!;
    const problemId = parseInt(sk.split("#")[1], 10);
    const score = parseInt(item["GSI1SK"]["N"]!, 10);

    results.push({ runId, commit, problemId, score, ai });
  }
  return results;
}

export async function fetchRun(id: string): Promise<RunResult> {
  const { Items: items } = await client.send(
    new QueryCommand({
      TableName,
      KeyConditionExpression: "PK = :pk",
      ExpressionAttributeValues: {
        ":pk": { S: `R#${id}` },
      },
      ScanIndexForward: false,
    })
  );
  if (!items) {
    throw new Error(`failed to fetch result: ${id}`);
  }

  const result: RunResult = {
    id,
    time: 0,
    args: "",
    score: 0,
    results: [],
  };

  for (const item of items) {
    const sk = item["SK"]["S"]!;

    if (sk.startsWith("R")) {
      result.time = parseInt(item["GSI1SK"]["N"]!);
      result.args = item["Args"]["S"]!;
    }

    if (sk.startsWith("S")) {
      const commit = item["Commit"]["S"]!;
      const problemId = parseInt(sk.split("#")[1], 10);
      const score = parseInt(item["GSI1SK"]["N"]!, 10);
      result.score += score;
      result.results.push({ problemId, score, commit });
    }
  }

  result.results.sort((a, b) => a.problemId - b.problemId);
  return result;
}

export async function fetchRunList(): Promise<Run[]> {
  const { Items: items } = await client.send(
    new QueryCommand({
      TableName,
      IndexName: "GSI1",
      KeyConditionExpression: "GSI1PK = :pk",
      ExpressionAttributeValues: {
        ":pk": { S: "R.Time" },
      },
      ScanIndexForward: false,
    })
  );

  if (!items) {
    return [];
  }

  const results: Run[] = [];
  for (const item of items) {
    let problems = 0;
    let score = 0;
    for (const [key, value] of Object.entries(item)) {
      if (key.startsWith("S#")) {
        problems++;
        score += parseInt(value["N"] as any, 10);
      }
    }

    results.push({
      id: item["PK"]["S"]?.split("#")[1]!,
      time: parseInt(item["GSI1SK"]["N"]!, 10),
      args: item["Args"]["S"]!,
      problems,
      score,
    });
  }

  return results;
}
