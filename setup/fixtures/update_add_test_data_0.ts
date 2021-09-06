import type { CreateAndPut } from "../dynamo_util.ts";

export const updateAddTestData0: CreateAndPut = {
  table: {
    TableName: "UpdateAddTestData0",
    KeySchema: [{ AttributeName: "id", KeyType: "HASH" }],
    AttributeDefinitions: [{ AttributeName: "id", AttributeType: "S" }],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  },
  items: [
    {
      id: { S: "id0" },
      sset: { SS: ["foo", "bar"] },
    },

    {
      id: { S: "id1" },
      sset: { NULL: true },
    },

    {
      id: { S: "id2" },
    },
  ],
};
