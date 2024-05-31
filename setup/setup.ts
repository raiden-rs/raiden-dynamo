import { DynamoDBClient } from "./deps.ts";
import { batchDeleteTest0 } from "./fixtures/batch_delete_test_0.ts";
import { batchDeleteTest1 } from "./fixtures/batch_delete_test_1.ts";
import { batchTest0 } from "./fixtures/batch_test_0.ts";
import { batchTest1 } from "./fixtures/batch_test_1.ts";
import { batchTest2 } from "./fixtures/batch_test_2.ts";
import { createTableAndPutItems, getCredFromEnv } from "./dynamo_util.ts";
import { deleteTest0 } from "./fixtures/delete_test_0.ts";
import { deleteTest1 } from "./fixtures/delete_test_1.ts";
import { emptyPutTestData0 } from "./fixtures/empty_put_test_data_0.ts";
import { emptySetTestData0 } from "./fixtures/empty_set_test_data_0.ts";
import { emptyStringTestData0 } from "./fixtures/empty_string_test_data_0.ts";
import { floatTest } from "./fixtures/float_test.ts";
import { hello } from "./fixtures/hello.ts";
import { lastEvaluateKeyData } from "./fixtures/last_evaluate_key_data.ts";
import { project } from "./fixtures/project.ts";
import { putItemConditionData0 } from "./fixtures/put_item_condition_data_0.ts";
import { queryLargeDataTest } from "./fixtures/query_large_data_test.ts";
import { queryTestData0 } from "./fixtures/query_test_data_0.ts";
import { queryTestData1 } from "./fixtures/query_test_data_1.ts";
import { renameAllCamelCaseTestData0 } from "./fixtures/rename_all_camel_case_test_data_0.ts";
import { renameAllPascalCaseTestData0 } from "./fixtures/rename_all_pascal_case_test_data_0.ts";
import { renameTestData0 } from "./fixtures/rename_test_data_0.ts";
import { reservedTestData0 } from "./fixtures/reserved_test_data_0.ts";
import { scanLargeDataTest } from "./fixtures/scan_large_data_test.ts";
import { scanTestData0 } from "./fixtures/scan_test_data_0.ts";
import { scanWithFilterTestData0 } from "./fixtures/scan_with_filter_test_data_0.ts";
import { testUserStaging } from "./fixtures/test_user_staging.ts";
import { txConditionalCheckTestData0 } from "./fixtures/tx_conditional_check_test_data_0.ts";
import { txConditionalCheckTestData1 } from "./fixtures/tx_conditional_check_test_data_1.ts";
import { txDeleteTestData0 } from "./fixtures/tx_delete_test_data_0.ts";
import { updateAddTestData0 } from "./fixtures/update_add_test_data_0.ts";
import { updateDeleteTestData0 } from "./fixtures/update_delete_test_data_0.ts";
import { updateRemoveTestData0 } from "./fixtures/update_remove_test_data_0.ts";
import { updateTestData0 } from "./fixtures/update_test_data_0.ts";
import { updateTestData1 } from "./fixtures/update_test_data_1.ts";
import { updateWithContainsInSetCondition } from "./fixtures/update_with_contains_in_set_condition.ts";
import { useDefaultForNull } from "./fixtures/use_default_for_null_data.ts";
import { useDefaultTestData0 } from "./fixtures/use_default_test_data_0.ts";
import { user } from "./fixtures/user.ts";

const client = new DynamoDBClient({
  region: "ap-northeast-1",
  endpoint: "http://localhost:8000",
  credentials: getCredFromEnv(),
});

const data = [
  batchDeleteTest0,
  batchDeleteTest1,
  batchTest0,
  batchTest1,
  batchTest2,
  deleteTest0,
  deleteTest1,
  emptyPutTestData0,
  emptySetTestData0,
  emptyStringTestData0,
  floatTest,
  hello,
  lastEvaluateKeyData,
  project,
  putItemConditionData0,
  queryLargeDataTest,
  queryTestData0,
  queryTestData1,
  renameAllCamelCaseTestData0,
  renameAllPascalCaseTestData0,
  renameTestData0,
  reservedTestData0,
  scanLargeDataTest,
  scanTestData0,
  scanWithFilterTestData0,
  testUserStaging,
  txConditionalCheckTestData0,
  txConditionalCheckTestData1,
  txDeleteTestData0,
  updateAddTestData0,
  updateDeleteTestData0,
  updateRemoveTestData0,
  updateTestData0,
  updateTestData1,
  updateWithContainsInSetCondition,
  useDefaultForNull,
  useDefaultTestData0,
  user,
];

// NOTE: Running these operations concurrently with `Promise.all` would lead to running out of write buffer.
for (const d of data) {
  console.log(`Processing ${d.table.TableName}...`);
  await createTableAndPutItems(client, d);
}
