import type { CreateAndPut } from "../dynamo_util.ts";

export const scanTestData0: CreateAndPut = {
  table: {
    TableName: "ScanTestData0",
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
  items: [
    {
      id: { S: "scanId0" },
      name: { S: "scanAlice" },
      year: { N: "2001" },
      num: { N: "2000" },
    },
  ],
};
