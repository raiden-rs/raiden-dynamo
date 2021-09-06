import type { CreateAndPut } from "../dynamo_util.ts";

export const project: CreateAndPut = {
  table: {
    TableName: "Project",
    KeySchema: [{ AttributeName: "id", KeyType: "HASH" }],
    AttributeDefinitions: [
      { AttributeName: "id", AttributeType: "S" },
      { AttributeName: "orgId", AttributeType: "S" },
      { AttributeName: "updatedAt", AttributeType: "S" },
    ],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
    GlobalSecondaryIndexes: [
      {
        IndexName: "orgIndex",
        KeySchema: [
          {
            AttributeName: "orgId",
            KeyType: "HASH",
          },
          {
            AttributeName: "updatedAt",
            KeyType: "RANGE",
          },
        ],
        ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
        Projection: {
          ProjectionType: "ALL",
        },
      },
    ],
  },
  items: [...Array(10).keys()].map((i) => {
    return {
      id: { S: `id${i}` },
      orgId: { S: `myOrg` },
      updatedAt: { S: "2019-03-11T00:00+0900" },
    };
  }),
};
