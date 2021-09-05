import type { CreateAndPut } from "../dynamo_util.ts";

export const updateTestData0: CreateAndPut = {
  table: {
    TableName: "UpdateTestData0",
    KeySchema: [{ AttributeName: "id", KeyType: "HASH" }],
    AttributeDefinitions: [{ AttributeName: "id", AttributeType: "S" }],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  },
  items: [
    {
      id: { S: "id0" },
      name: { S: "john" },
      age: { N: "12" },
      num: { N: "1" },
    },
    { id: { S: "id1" }, name: { S: "bob" }, age: { N: "18" }, num: { N: "1" } },
  ],
};
