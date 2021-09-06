import type { CreateAndPut } from "../dynamo_util.ts";

export const renameAllPascalCaseTestData0: CreateAndPut = {
  table: {
    TableName: "RenameAllPascalCaseTestData0",
    KeySchema: [{ AttributeName: "PartitionKey", KeyType: "HASH" }],
    AttributeDefinitions: [
      {
        AttributeName: "PartitionKey",
        AttributeType: "S",
      },
    ],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  },
  items: [
    {
      PartitionKey: { S: "id0" },
      FooBar: { S: "john" },
      ProjectId: { N: "1" },
    },
    { PartitionKey: { S: "id1" }, FooBar: { S: "bob" }, ProjectId: { N: "2" } },
  ],
};
