import type { CreateAndPut } from "../dynamo_util.ts";

export const emptyStringTestData0: CreateAndPut = {
  table: {
    TableName: "EmptyStringTestData0",
    KeySchema: [{ AttributeName: "id", KeyType: "HASH" }],
    AttributeDefinitions: [{ AttributeName: "id", AttributeType: "S" }],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  },
  items: [
    {
      id: { S: "id0" },
      name: { NULL: true },
    },
  ],
};
