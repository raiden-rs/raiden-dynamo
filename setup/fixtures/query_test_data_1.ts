import type { CreateAndPut } from "../dynamo_util.ts";

export const queryTestData1: CreateAndPut = {
  table: {
    TableName: "QueryTestData1",
    KeySchema: [
      { AttributeName: "id", KeyType: "HASH" },
      { AttributeName: "name", KeyType: "RANGE" },
    ],
    AttributeDefinitions: [
      { AttributeName: "id", AttributeType: "S" },
      { AttributeName: "name", AttributeType: "S" },
    ],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  },
  items: [
    { id: { S: "id0" }, name: { S: "john" } },
    { id: { S: "id0" }, name: { S: "jack" } },
    { id: { S: "id0" }, name: { S: "bob" } },
  ],
};
