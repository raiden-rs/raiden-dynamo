import type { CreateAndPut } from "../dynamo_util.ts";

export const renameAllCamelCaseTestData0: CreateAndPut = {
  table: {
    TableName: "RenameAllCamelCaseTestData0",
    KeySchema: [{ AttributeName: "partitionKey", KeyType: "HASH" }],
    AttributeDefinitions: [
      {
        AttributeName: "partitionKey",
        AttributeType: "S",
      },
    ],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  },
  items: [
    {
      partitionKey: { S: "id0" },
      fooBar: { S: "john" },
      projectId: { N: "1" },
    },
    { partitionKey: { S: "id1" }, fooBar: { S: "bob" }, projectId: { N: "2" } },
  ],
};
