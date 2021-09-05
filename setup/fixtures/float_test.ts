import type { CreateAndPut } from "../dynamo_util.ts";

export const floatTest: CreateAndPut = {
  table: {
    TableName: "FloatTest",
    KeySchema: [{ AttributeName: "id", KeyType: "HASH" }],
    AttributeDefinitions: [{ AttributeName: "id", AttributeType: "S" }],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  },
  items: [
    {
      id: { S: "primary_key" },
      float32: { N: "1.23" },
      float64: { N: "2.34" },
    },
  ],
};
