import { DynamoDBClient, QueryCommand } from "@aws-sdk/client-dynamodb";

export interface Run {
  id: string;
  time: number;
  args: string;
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

export async function fetchRun(id: string): Promise<RunResult> {
  const client = new DynamoDBClient({});
  const { Items: items } = await client.send(
    new QueryCommand({
      TableName: "InfraStack-TableCD117FA1-1NAQ40LMS0E1G",
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
  const client = new DynamoDBClient({});
  const { Items: items } = await client.send(
    new QueryCommand({
      TableName: "InfraStack-TableCD117FA1-1NAQ40LMS0E1G",
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
    results.push({
      id: item["PK"]["S"]?.split("#")[1]!,
      time: parseInt(item["GSI1SK"]["N"]!),
      args: item["Args"]["S"]!,
    });
  }

  return results;
}
