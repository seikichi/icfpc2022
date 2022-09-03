import { DynamoDBClient, QueryCommand } from "@aws-sdk/client-dynamodb";

export interface Run {
  id: string;
  time: number;
  ai: string;
}

export async function fetchRun(id: string): Promise<any> {
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
  return items;
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
      ai: item["AI"]["S"]!,
    });
  }

  return results;
}
