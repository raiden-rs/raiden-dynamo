import type { CreateAndPut } from "../dynamo_util.ts";

export const batchDeleteTest1: CreateAndPut = {
  table: {
    TableName: "BatchDeleteTest1",
    KeySchema: [
      { AttributeName: "id", KeyType: "HASH" },
      { AttributeName: "year", KeyType: "RANGE" },
    ],
    AttributeDefinitions: [
      { AttributeName: "id", AttributeType: "S" },
      { AttributeName: "year", AttributeType: "N" },
    ],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  },
  items: [...Array(101).keys()].map((i) => {
    const year = 1999 + i;
    return {
      id: { S: `id${i}` },
      name: { S: "alice" },
      year: { N: year.toString() },
    };
  }),
};
