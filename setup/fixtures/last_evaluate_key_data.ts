import type { CreateAndPut } from "../dynamo_util.ts";

export const lastEvaluateKeyData: CreateAndPut = {
  table: {
    TableName: "LastEvaluateKeyData",
    KeySchema: [{ AttributeName: "id", KeyType: "HASH" }],
    AttributeDefinitions: [
      { AttributeName: "id", AttributeType: "S" },
      { AttributeName: "ref_id", AttributeType: "S" },
    ],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
    GlobalSecondaryIndexes: [
      {
        IndexName: "testGSI",
        KeySchema: [{ AttributeName: "ref_id", KeyType: "HASH" }],
        Projection: {
          ProjectionType: "ALL",
        },
        ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
      },
    ],
  },
  items: [...Array(10).keys()].map((i) => {
    return {
      id: { S: `id${i}` },
      ref_id: { S: `id0` },
      long_text: { S: new Array(100000).fill("Test").join("") },
    };
  }),
};
