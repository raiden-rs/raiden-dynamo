import type { CreateAndPut } from "../dynamo_util.ts";

export const reservedTestData0: CreateAndPut = {
  table: {
    TableName: "ReservedTestData0",
    KeySchema: [{ AttributeName: "id", KeyType: "HASH" }],
    AttributeDefinitions: [{ AttributeName: "id", AttributeType: "S" }],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  },
  items: [{ id: { S: "id0" }, type: { S: "reserved" } }],
};
