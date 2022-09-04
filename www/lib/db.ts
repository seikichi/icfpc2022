import {
  DynamoDBClient,
  PutItemCommand,
  QueryCommand,
} from "@aws-sdk/client-dynamodb";

import { LambdaClient, InvokeCommand } from "@aws-sdk/client-lambda";

export interface Run {
  id: string;
  time: number;
  args: string;
  target: string;
  problems: number;
  score: number;
}

export interface RunResult {
  id: string;
  time: number;
  args: string;
  score: number;
  target: string;

  results: {
    commit: string;
    problemId: number;
    score: number;
    time: number | null;
    date: number | null;
  }[];
}

export interface Solution {
  runId: string;
  commit: string;
  problemId: number;
  score: number;
  ai: string;
  time: number | null;
  date: number | null;
}

// .env に書け
const region = "ap-northeast-1";
const TableName = "InfraStack-TableCD117FA1-1NAQ40LMS0E1G";
const FunctionName = "InfraStack-Solver4A42070C-PVa1uZZEnKmR";
const credentials =
  process.env.AWS_ACCESS_KEY_ID && process.env.AWS_SECRET_ACCESS_KEY
    ? {
        accessKeyId: process.env.AWS_ACCESS_KEY_ID!,
        secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY!,
      }
    : undefined;

const client = new DynamoDBClient({ region, credentials });
const lambda = new LambdaClient({ region, credentials });

export async function putRun(params: {
  uuid: string;
  args: string;
  target: string;
}) {
  const { uuid, args, target } = params;

  // 問題IDパース
  const ids: Set<number> = new Set([]);
  for (const ps of target.split(",")) {
    if (ps.includes("-")) {
      const [fromS, toS] = ps.split("-");
      const [from, to] = [parseInt(fromS, 10), parseInt(toS, 10)];
      for (let i = from; i <= to; i++) {
        ids.add(i);
      }
    } else {
      ids.add(parseInt(ps, 10));
    }
  }

  // Run 追加
  const pk = `R#${uuid}`;
  const time = Math.floor(new Date().getTime() / 1000);
  await client.send(
    new PutItemCommand({
      TableName,
      Item: {
        PK: { S: pk },
        SK: { S: pk },
        Args: { S: args },
        Target: { S: target },
        GSI1PK: { S: "R.Time" },
        GSI1SK: { N: `${time}` },
      },
    })
  );

  // Lambda 実行
  await Promise.all(
    Array.from(ids).map((problemId) => {
      const payload = JSON.stringify({
        problemId: `${problemId}`,
        args,
        runId: uuid,
      });
      return lambda.send(
        new InvokeCommand({
          FunctionName,
          InvocationType: "Event",
          Payload: Buffer.from(payload),
        })
      );
    })
  );

  return;
}

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

    const dateS = item["ExecDate"]?.N;
    const timeS = item["ExecTime"]?.N;
    const date = dateS ? parseInt(dateS, 10) : null;
    const time = timeS ? parseInt(timeS, 10) : null;

    results.push({ runId, commit, problemId, score, ai, date, time });
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
    target: "",
    results: [],
  };

  for (const item of items) {
    const sk = item["SK"]["S"]!;

    if (sk.startsWith("R")) {
      result.time = parseInt(item["GSI1SK"]["N"]!);
      result.args = item["Args"]["S"]!;
      result.target = item["Target"]["S"]!;
    }

    if (sk.startsWith("S")) {
      const commit = item["Commit"]["S"]!;
      const problemId = parseInt(sk.split("#")[1], 10);
      const score = parseInt(item["GSI1SK"]["N"]!, 10);
      const dateS = item["ExecDate"]?.N;
      const timeS = item["ExecTime"]?.N;
      const date = dateS ? parseInt(dateS, 10) : null;
      const time = timeS ? parseInt(timeS, 10) : null;
      result.score += score;
      result.results.push({ problemId, score, commit, date, time });
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
      target: item["Target"]["S"]!,
      problems,
      score,
    });
  }

  return results;
}
