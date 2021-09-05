import type { CreateAndPut } from "../dynamo_util.ts";

export const scanLargeDataTest: CreateAndPut = {
  table: {
    TableName: "ScanLargeDataTest",
    KeySchema: [{ AttributeName: "id", KeyType: "HASH" }],
    AttributeDefinitions: [{ AttributeName: "id", AttributeType: "S" }],
    ProvisionedThroughput: { ReadCapacityUnits: 50, WriteCapacityUnits: 50 },
  },
  items: [...Array(100).keys()].map((i) => {
    return {
      id: { S: `id${i}` },
      ref_id: { S: "ref" },
      name: { S: [...new Array(100000)].map((_) => "test").join("") }, // 400KB
    };
  }),
};
