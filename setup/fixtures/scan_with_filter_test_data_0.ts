import type { CreateAndPut } from "../dynamo_util.ts";

export const scanWithFilterTestData0: CreateAndPut = {
  table: {
    TableName: "ScanWithFilterTestData0",
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
  items: [...new Array(100)].map((_, i) => {
    return {
      id: { S: `scanId${i}` },
      name: { S: `scanAlice${i}` },
      year: { N: "2001" },
      num: { N: i % 2 ? "1000" : "2000" },
      option: i % 2 ? { S: `option${i}` } : null,
    };
  }),
};
