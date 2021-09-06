import type { CreateAndPut } from "../dynamo_util.ts";

export const emptySetTestData0: CreateAndPut = {
  table: {
    TableName: "EmptySetTestData0",
    KeySchema: [{ AttributeName: "id", KeyType: "HASH" }],
    AttributeDefinitions: [{ AttributeName: "id", AttributeType: "S" }],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  },
  items: [
    {
      id: { S: "id0" },
      nset: { NS: ["2000"] },
      sset: { SS: ["Hello"] },
    },
    {
      id: { S: "id1" },
      nset: { NS: ["2001"] },
      sset: { SS: ["World"] },
    },
  ],
};
