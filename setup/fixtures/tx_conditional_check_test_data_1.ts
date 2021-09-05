import type { CreateAndPut } from "../dynamo_util.ts";

export const txConditionalCheckTestData1: CreateAndPut = {
  table: {
    TableName: "TxConditionalCheckTestData1",
    KeySchema: [{ AttributeName: "id", KeyType: "HASH" }],
    AttributeDefinitions: [{ AttributeName: "id", AttributeType: "S" }],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  },
  items: [{ id: { S: "id1" }, name: { S: "world" } }],
};
