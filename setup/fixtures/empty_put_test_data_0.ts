import type { CreateAndPut } from "../dynamo_util.ts";

export const emptyPutTestData0: CreateAndPut = {
  table: {
    TableName: "EmptyPutTestData0",
    KeySchema: [{ AttributeName: "id", KeyType: "HASH" }],
    AttributeDefinitions: [{ AttributeName: "id", AttributeType: "S" }],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  },
  items: [],
};
