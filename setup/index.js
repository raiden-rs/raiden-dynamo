const AWS = require('aws-sdk');

AWS.config.update({
  region: 'ap-northeast-1',
  // @ts-ignore
  endpoint: 'http://localhost:8000',
});

const dynamodb = new AWS.DynamoDB();

const createTable = (params) =>
  new Promise((resolve, reject) => {
    dynamodb.createTable(params, (err, data) => {
      if (err) {
        reject(err);
      }
      resolve(data);
    });
  });

const put = (params) =>
  new Promise((resolve, reject) => {
    dynamodb.putItem(params, (err, data) => {
      if (err) {
        reject(err);
      }
      resolve(data);
    });
  });

(async () => {
  await createTable({
    TableName: 'user',
    KeySchema: [{ AttributeName: 'id', KeyType: 'HASH' }],
    AttributeDefinitions: [{ AttributeName: 'id', AttributeType: 'S' }],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  });

  await put({
    TableName: 'user',
    Item: {
      id: { S: 'user_primary_key' },
      name: { S: 'bokuweb' },
      num_usize: { N: '42' },
      num_u8: { N: '255' },
      num_i8: { N: '-127' },
      option_i16: { N: '-1' },
    },
  });

  await createTable({
    TableName: 'QueryTestData0',
    KeySchema: [
      { AttributeName: 'id', KeyType: 'HASH' },
      { AttributeName: 'year', KeyType: 'RANGE' },
    ],
    AttributeDefinitions: [
      { AttributeName: 'id', AttributeType: 'S' },
      { AttributeName: 'year', AttributeType: 'N' },
    ],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  });

  await put({
    TableName: 'QueryTestData0',
    Item: { id: { S: 'id0' }, name: { S: 'john' }, year: { N: '1999' }, num: { N: '1000' } },
  });
  await put({
    TableName: 'QueryTestData0',
    Item: { id: { S: 'id0' }, name: { S: 'john' }, year: { N: '2000' }, num: { N: '2000' } },
  });
  await put({
    TableName: 'QueryTestData0',
    Item: { id: { S: 'id1' }, name: { S: 'bob' }, year: { N: '2003' }, num: { N: '300' } },
  });
  await put({
    TableName: 'QueryTestData0',
    Item: { id: { S: 'id2' }, name: { S: 'alice' }, year: { N: '2013' }, num: { N: '4000' } },
  });

  await createTable({
    TableName: 'RenameTestData0',
    KeySchema: [{ AttributeName: 'id', KeyType: 'HASH' }],
    AttributeDefinitions: [{ AttributeName: 'id', AttributeType: 'S' }],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  });
  await put({
    TableName: 'RenameTestData0',
    Item: { id: { S: 'id0' }, name: { S: 'john' }, renamed: { N: '1999' } },
  });
  await put({
    TableName: 'RenameTestData0',
    Item: { id: { S: 'id1' }, name: { S: 'bob' }, renamed: { N: '2003' } },
  });

  await createTable({
    TableName: 'RenameAllCamelCaseTestData0',
    KeySchema: [{ AttributeName: 'partitionKey', KeyType: 'HASH' }],
    AttributeDefinitions: [{ AttributeName: 'partitionKey', AttributeType: 'S' }],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  });
  await put({
    TableName: 'RenameAllCamelCaseTestData0',
    Item: { partitionKey: { S: 'id0' }, fooBar: { S: 'john' }, projectId: { N: '1' } },
  });
  await put({
    TableName: 'RenameAllCamelCaseTestData0',
    Item: { partitionKey: { S: 'id1' }, fooBar: { S: 'bob' }, projectId: { N: '2' } },
  });

  await createTable({
    TableName: 'RenameAllPascalCaseTestData0',
    KeySchema: [{ AttributeName: 'PartitionKey', KeyType: 'HASH' }],
    AttributeDefinitions: [{ AttributeName: 'PartitionKey', AttributeType: 'S' }],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  });
  await put({
    TableName: 'RenameAllPascalCaseTestData0',
    Item: { PartitionKey: { S: 'id0' }, FooBar: { S: 'john' }, ProjectId: { N: '1' } },
  });
  await put({
    TableName: 'RenameAllPascalCaseTestData0',
    Item: { PartitionKey: { S: 'id1' }, FooBar: { S: 'bob' }, ProjectId: { N: '2' } },
  });

  await createTable({
    TableName: 'UpdateTestData0',
    KeySchema: [{ AttributeName: 'id', KeyType: 'HASH' }],
    AttributeDefinitions: [{ AttributeName: 'id', AttributeType: 'S' }],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  });
  await put({
    TableName: 'UpdateTestData0',
    Item: { id: { S: 'id0' }, name: { S: 'john' }, age: { N: '12' }, num: { N: '1' } },
  });
  await put({
    TableName: 'UpdateTestData0',
    Item: { id: { S: 'id1' }, name: { S: 'bob' }, age: { N: '18' }, num: { N: '1' } },
  });

  await createTable({
    TableName: 'PutItemConditionData0',
    KeySchema: [{ AttributeName: 'id', KeyType: 'HASH' }],
    AttributeDefinitions: [{ AttributeName: 'id', AttributeType: 'S' }],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
  });

  await put({
    TableName: 'user',
    Item: {
      id: { S: 'id0' },
      name: { S: 'bokuweb' },
      num: { N: '1000' },
    },
  });

  await createTable({
    TableName: 'LastEvaluateKeyData',
    KeySchema: [{ AttributeName: 'id', KeyType: 'HASH' }],
    AttributeDefinitions: [
      { AttributeName: 'id', AttributeType: 'S' },
      { AttributeName: 'ref_id', AttributeType: 'S' },
    ],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
    GlobalSecondaryIndexes: [
      {
        IndexName: 'testGSI',
        KeySchema: [{ AttributeName: 'ref_id', KeyType: 'HASH' }],
        Projection: {
          ProjectionType: 'ALL',
        },
        ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
      },
    ],
  });

  for (let i = 0; i < 10; i++) {
    await put({
      TableName: 'LastEvaluateKeyData',
      Item: {
        id: { S: `id${i}` },
        ref_id: { S: `id0` },
        long_text: { S: new Array(100000).fill('Test').join('') },
      },
    });
  }

  await createTable({
    TableName: 'Project',
    KeySchema: [{ AttributeName: 'id', KeyType: 'HASH' }],
    AttributeDefinitions: [
      { AttributeName: 'id', AttributeType: 'S' },
      { AttributeName: 'orgId', AttributeType: 'S' },
      { AttributeName: 'updatedAt', AttributeType: 'S' },
    ],
    ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
    GlobalSecondaryIndexes: [
      {
        IndexName: 'orgIndex',
        KeySchema: [
          {
            AttributeName: 'orgId',
            KeyType: 'HASH',
          },
          {
            AttributeName: 'updatedAt',
            KeyType: 'RANGE',
          },
        ],
        ProvisionedThroughput: { ReadCapacityUnits: 5, WriteCapacityUnits: 5 },
        Projection: {
          ProjectionType: 'ALL',
        },
      },
    ],
  });

  for (let i = 0; i < 10; i++) {
    await put({
      TableName: 'Project',
      Item: {
        id: { S: `id${i}` },
        orgId: { S: `myOrg` },
        updatedAt: { S: '2019-03-11T00:00+0900' },
      },
    });
  }
})();
