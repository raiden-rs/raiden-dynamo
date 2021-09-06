import {
  AttributeValue,
  CreateTableCommand,
  CreateTableCommandInput,
  DynamoDBClient,
  PutItemCommand,
  PutItemCommandInput,
} from "./deps.ts";

export type CreateAndPut = {
  table: CreateTableCommandInput;
  items: Array<{
    [key: string]: AttributeValue;
  }>;
};

export async function createTableAndPutItems(
  client: DynamoDBClient,
  { table, items }: CreateAndPut,
) {
  await createTable(client, table);

  // NOTE: Running `put` operations concurrently with `Promise.all` would lead to running out of write buffer.
  for (const item of items) {
    await put(client, {
      TableName: table.TableName,
      Item: item,
    });
  }
}

export function getCredFromEnv(): {
  accessKeyId: string;
  secretAccessKey: string;
} {
  const accessKeyId = Deno.env.get("AWS_ACCESS_KEY_ID");
  const secretAccessKey = Deno.env.get("AWS_SECRET_ACCESS_KEY");

  if (accessKeyId === undefined || secretAccessKey === undefined) {
    throw new Error(
      "Failed to get aws credentials. Make sure environment variables `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` are set.",
    );
  }

  return { accessKeyId, secretAccessKey };
}

async function createTable(
  client: DynamoDBClient,
  input: CreateTableCommandInput,
) {
  await client.send(new CreateTableCommand(input));
}

async function put(client: DynamoDBClient, input: PutItemCommandInput) {
  await client.send(new PutItemCommand(input));
}
