import type { CreateAndPut } from "../dynamo_util.ts";

export const updateRemoveTestData0: CreateAndPut = {
  table: {
    TableName: "UpdateRemoveTestData0",
    KeySchema: [{ AttributeName: "id", KeyType: "HASH" }],
    AttributeDefinitions: [{ AttributeName: "id", AttributeType: "S" }],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  },
  items: [{ id: { S: "id1" }, name: { S: "world" } }, { id: { S: "id2" } }],
};
