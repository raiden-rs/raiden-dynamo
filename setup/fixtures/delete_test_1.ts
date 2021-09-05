import type { CreateAndPut } from "../dynamo_util.ts";

export const deleteTest1: CreateAndPut = {
  table: {
    TableName: "DeleteTest1",
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
      id: { S: "id0" },
      name: { S: "alice" },
      year: { N: "1999" },
    },
  ],
};
