import type { CreateAndPut } from "../dynamo_util.ts";

export const useDefaultForNull: CreateAndPut = {
  table: {
    TableName: "UseDefaultForNull",
    KeySchema: [{ AttributeName: "id", KeyType: "HASH" }],
    AttributeDefinitions: [{ AttributeName: "id", AttributeType: "S" }],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  },
  items: [
    {
      id: { S: "id0" },
      flag: { NULL: true },
    },
  ],
};
