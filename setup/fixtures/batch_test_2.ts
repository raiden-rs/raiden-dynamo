import type { CreateAndPut } from "../dynamo_util.ts";

export const batchTest2: CreateAndPut = {
  table: {
    TableName: "BatchTest2",
    KeySchema: [{ AttributeName: "id", KeyType: "HASH" }],
    AttributeDefinitions: [{ AttributeName: "id", AttributeType: "S" }],
    ProvisionedThroughput: { ReadCapacityUnits: 50, WriteCapacityUnits: 50 },
  },
  items: [...Array(250).keys()].map((i) => {
    return {
      id: { S: `id${i}` },
      name: { S: [...new Array(100000)].map((_) => "test").join("") },
    };
  }),
};
