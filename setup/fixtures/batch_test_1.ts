import type { CreateAndPut } from "../dynamo_util.ts";

export const batchTest1: CreateAndPut = {
  table: {
    TableName: "BatchTest1",
    KeySchema: [
      { AttributeName: "id", KeyType: "HASH" },
      { AttributeName: "year", KeyType: "RANGE" },
    ],
    AttributeDefinitions: [
      { AttributeName: "id", AttributeType: "S" },
      { AttributeName: "year", AttributeType: "N" },
    ],
    ProvisionedThroughput: { ReadCapacityUnits: 50, WriteCapacityUnits: 50 },
  },
  items: [...Array(250).keys()].map((i) => {
    return {
      id: { S: `id${i}` },
      name: { S: "bob" },
      year: { N: `${2000 + i}` },
      num: { N: `${i}` },
    };
  }),
};
