import type { CreateAndPut } from "../dynamo_util.ts";

export const queryTestData0: CreateAndPut = {
  table: {
    TableName: "QueryTestData0",
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
      name: { S: "john" },
      year: { N: "1999" },
      num: { N: "1000" },
    },
    {
      id: { S: "id0" },
      name: { S: "john" },
      year: { N: "2000" },
      num: { N: "2000" },
    },
    {
      id: { S: "id1" },
      name: { S: "bob" },
      year: { N: "2003" },
      num: { N: "300" },
    },
    {
      id: { S: "id2" },
      name: { S: "alice" },
      year: { N: "2013" },
      num: { N: "4000" },
    },
    {
      id: { S: "id3" },
      name: { S: "bar0" },
      year: { N: "1987" },
      num: { N: "4000" },
    },
    {
      id: { S: "id3" },
      name: { S: "bar1" },
      year: { N: "2000" },
      num: { N: "4000" },
    },
    {
      id: { S: "id3" },
      name: { S: "bar2" },
      year: { N: "2029" },
      num: { N: "4000" },
    },
    {
      id: { S: "id4" },
      name: { S: "bar0" },
      year: { N: "2029" },
      num: { N: "4000" },
    },
    {
      id: { S: "id4" },
      name: { S: "bar1" },
      year: { N: "2000" },
      num: { N: "4000" },
      option: { S: "option2" },
    },
    {
      id: { S: "id4" },
      name: { S: "bob" },
      year: { N: "1999" },
      num: { N: "4000" },
      option: { S: "option2" },
    },
  ],
};
