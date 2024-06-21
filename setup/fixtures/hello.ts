import type { CreateAndPut } from "../dynamo_util.ts";

export const hello: CreateAndPut = {
  table: {
    TableName: "hello",
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
      id: { S: "bokuweb" },
      year: { N: "2019" },
    },
    {
      id: { S: "raiden" },
      year: { N: "2020" },
    },
  ],
};
