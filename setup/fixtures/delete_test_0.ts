import type { CreateAndPut } from "../dynamo_util.ts";

export const deleteTest0: CreateAndPut = {
  table: {
    TableName: "DeleteTest0",
    KeySchema: [{ AttributeName: "id", KeyType: "HASH" }],
    AttributeDefinitions: [{ AttributeName: "id", AttributeType: "S" }],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  },
  items: [
    {
      id: { S: "id0" },
      name: { S: "bokuweb" },
      number_set: { NS: ["1"] },
    },
    {
      id: { S: "id1" },
      name: { S: "bokuweb" },
      removable: { BOOL: true },
    },
  ],
};
