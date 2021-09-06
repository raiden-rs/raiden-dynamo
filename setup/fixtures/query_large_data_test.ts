import type { CreateAndPut } from "../dynamo_util.ts";

export const queryLargeDataTest: CreateAndPut = {
  table: {
    TableName: "QueryLargeDataTest",
    KeySchema: [{ AttributeName: "id", KeyType: "HASH" }],
    AttributeDefinitions: [
      { AttributeName: "id", AttributeType: "S" },
      { AttributeName: "ref_id", AttributeType: "S" },
    ],
    ProvisionedThroughput: { ReadCapacityUnits: 50, WriteCapacityUnits: 50 },
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
  items: [...Array(100).keys()].map((i) => {
    return {
      id: { S: `id${i}` },
      ref_id: { S: "ref" },
      name: { S: [...new Array(100000)].map((_) => "test").join("") }, // 400KB
    };
  }),
};
