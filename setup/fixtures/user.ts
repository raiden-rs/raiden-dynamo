import type { CreateAndPut } from "../dynamo_util.ts";

export const user: CreateAndPut = {
  table: {
    TableName: "user",
    KeySchema: [{ AttributeName: "id", KeyType: "HASH" }],
    AttributeDefinitions: [{ AttributeName: "id", AttributeType: "S" }],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  },
  items: [
    {
      id: { S: "user_primary_key" },
      name: { S: "bokuweb" },
      num_usize: { N: "42" },
      num_u8: { N: "255" },
      num_i8: { N: "-127" },
      option_i16: { N: "-1" },
      string_set: { SS: ["Hello"] },
      number_set: { NS: ["1"] },
    },
    {
      id: { S: "id0" },
      name: { S: "bokuweb" },
      num: { N: "1000" },
    },
  ],
};
