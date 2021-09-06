import type { CreateAndPut } from "../dynamo_util.ts";

export const renameTestData0: CreateAndPut = {
  table: {
    TableName: "RenameTestData0",
    KeySchema: [{ AttributeName: "id", KeyType: "HASH" }],
    AttributeDefinitions: [{ AttributeName: "id", AttributeType: "S" }],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  },
  items: [
    { id: { S: "id0" }, name: { S: "john" }, renamed: { N: "1999" } },
    { id: { S: "id1" }, name: { S: "bob" }, renamed: { N: "2003" } },
  ],
};
