import type { CreateAndPut } from "../dynamo_util.ts";

export const batchTest0: CreateAndPut = {
  table: {
    TableName: "BatchTest0",
    KeySchema: [{ AttributeName: "id", KeyType: "HASH" }],
    AttributeDefinitions: [{ AttributeName: "id", AttributeType: "S" }],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  },
  items: [...Array(101).keys()].map((i) => {
    return {
      id: { S: `id${i}` },
      name: { S: "bob" },
    };
  }),
};
