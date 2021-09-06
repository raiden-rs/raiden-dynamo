import type { CreateAndPut } from "../dynamo_util.ts";

export const updateWithContainsInSetCondition: CreateAndPut = {
  table: {
    TableName: "UpdateWithContainsInSetCondition",
    KeySchema: [{ AttributeName: "id", KeyType: "HASH" }],
    AttributeDefinitions: [{ AttributeName: "id", AttributeType: "S" }],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  },
  items: [
    {
      id: { S: "id0" },
      name: { S: "bokuweb" },
      sset: { SS: ["Hello"] },
    },
  ],
};
