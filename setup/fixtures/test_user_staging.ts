import type { CreateAndPut } from "../dynamo_util.ts";

export const testUserStaging: CreateAndPut = {
  table: {
    TableName: "test-user-staging",
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
  items: [],
};
