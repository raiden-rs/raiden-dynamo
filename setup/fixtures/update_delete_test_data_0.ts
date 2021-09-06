import type { CreateAndPut } from "../dynamo_util.ts";

export const updateDeleteTestData0: CreateAndPut = {
  table: {
    TableName: "UpdateDeleteTestData0",
    KeySchema: [{ AttributeName: "id", KeyType: "HASH" }],
    AttributeDefinitions: [{ AttributeName: "id", AttributeType: "S" }],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  },
  items: [
    {
      id: { S: "id0" },
      sset: { SS: ["foo", "bar"] },
    },
  ],
};
