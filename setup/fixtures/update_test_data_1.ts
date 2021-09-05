import type { CreateAndPut } from "../dynamo_util.ts";

export const updateTestData1: CreateAndPut = {
  table: {
    TableName: "UpdateTestData1",
    KeySchema: [
      { AttributeName: "id", KeyType: "HASH" },
      { AttributeName: "age", KeyType: "RANGE" },
    ],
    AttributeDefinitions: [
      { AttributeName: "id", AttributeType: "S" },
      { AttributeName: "age", AttributeType: "N" },
    ],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  },
  items: [{ id: { S: "id0" }, name: { S: "john" }, age: { N: "36" } }],
};
