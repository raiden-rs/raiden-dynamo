#!/usr/bin/env python3
"""Compare real HTTP error decoding in separate Rusoto/AWS SDK feature builds.

Keeps every input, observation and mismatch. No AWS credentials/server needed.
This is a finite cross-product exploration, not coverage-guided fuzzing.
"""
import argparse
import json
import os
import random
from pathlib import Path
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--output-dir", type=Path)
parser.add_argument("--toolchain", default="1.91.1")
parser.add_argument("--exploratory-cases", type=int, default=2000)
parser.add_argument("--seed", type=int, default=20261124)
parser.add_argument("--responses", action="store_true", help="explore successful read-response decoding")
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
directory = (args.output_dir or Path(tempfile.mkdtemp(prefix="raiden-http-errors-"))).resolve()
directory.mkdir(parents=True, exist_ok=True)
operations = ["get", "batch_get", "transact_get", "query", "scan", "put",
              "batch_put", "batch_delete", "delete", "update", "transact_write"]
codes = ["InternalServerError", "InvalidEndpointException", "ValidationException",
         "ProvisionedThroughputExceededException", "RequestLimitExceeded",
         "ResourceNotFoundException", "ConditionalCheckFailedException",
         "ItemCollectionSizeLimitExceededException", "TransactionConflictException",
         "TransactionInProgressException", "IdempotentParameterMismatchException",
         "TransactionCanceledException", "ThrottlingException", "AccessDeniedException",
         "FutureServiceException"]
messages = [{"message": "detail あ\n😀"}, {"Message": "detail あ\n😀"},
            {"message": ""}, {}, {"message": None}]
cases = []
for operation in operations:
    for code in codes:
        for prefix in ["", "com.amazonaws.dynamodb.v20120810#"]:
            for status in [400, 500]:
                for message in messages:
                    cases.append({"operation": operation, "status": status,
                                  "body": {"__type": prefix + code, **message}})
rng = random.Random(args.seed)
for _ in range(args.exploratory_cases):
    code = rng.choice(codes)
    body = rng.choice([
        {"__type": code}, {"code": code}, {"Code": code},
        {"__type": "prefix#nested#" + code}, {"__type": code + ":detail"},
        {"__type": None}, {"__type": 1}, {},
        {"__type": code, "code": "ValidationException"},
    ])
    body.update(rng.choice(messages + [
        {"message": 1}, {"Message": []},
        {"message": "lower", "Message": "upper"},
        {"message": None, "Message": "upper"},
    ]))
    if rng.randrange(3) == 0:
        body["CancellationReasons"] = rng.choice([
            None, [], [{"Code": "None"}], [{"Code": "ConditionalCheckFailed"}],
            [{"Code": 1}], "invalid",
        ])
    cases.append({"operation": rng.choice(operations), "status": rng.choice([400, 403, 429, 500, 503]),
                  "body": body})
if args.responses:
    cases = []
    for operation in ["query", "scan"]:
        for count in [{}, {"Count": None}, {"Count": 0}, {"Count": 2}]:
            for scanned in [{}, {"ScannedCount": None}, {"ScannedCount": 0}, {"ScannedCount": 2}]:
                for items in [[], [{"id": {"S": "id"}}], [{"id": {"S": "あ😀"}}, {"id": {"S": "id"}}]]:
                    for capacity in [{}, {"ConsumedCapacity": None}, {"ConsumedCapacity": {"CapacityUnits": 0.5}}]:
                        cases.append({"operation": operation, "body": {"Items": items, **count, **scanned, **capacity}})
    for operation in ["get", "batch_get", "transact_get", "query", "scan"]:
        for attribute in [{"S": "id"}, {"S": ""}, {"NULL": True}, {"NULL": False},
                          {"N": "1"}, {"BOOL": False}, {"L": []}, {"M": {}}, {"B": "AA=="}]:
            item = {"id": attribute}
            body = {"get": {"Item": item},
                    "batch_get": {"Responses": {"OperationalCompat": [item]}},
                    "transact_get": {"Responses": [{"Item": item}]},
                    "query": {"Items": [item], "Count": 1, "ScannedCount": 1},
                    "scan": {"Items": [item], "Count": 1, "ScannedCount": 1}}[operation]
            cases.append({"operation": operation, "body": body})
    # Optional/empty result containers and nested capacity metadata.
    for operation in ["get", "batch_get", "transact_get", "query", "scan"]:
        for body in [{}, {"ConsumedCapacity": None}]:
            cases.append({"operation": operation, "body": body})
    for operation in ["query", "scan"]:
        for items in [{}, {"Items": None}, {"Items": []}]:
            for capacity in [{}, {"TableName": "OperationalCompat"},
                             {"CapacityUnits": 0, "ReadCapacityUnits": 0, "WriteCapacityUnits": 0},
                             {"Table": {}, "LocalSecondaryIndexes": {}, "GlobalSecondaryIndexes": {}},
                             {"Table": {"CapacityUnits": 0.5},
                              "GlobalSecondaryIndexes": {"gsi": {"ReadCapacityUnits": 0.25}}}]:
                cases.append({"operation": operation,
                              "body": {**items, "ConsumedCapacity": capacity}})
corpus = directory / "corpus.jsonl"
corpus.write_text("".join(json.dumps(case, ensure_ascii=True) + "\n" for case in cases))
print(f"seed={args.seed} cases={len(cases)} artifacts={directory}", flush=True)
for backend in ["rusoto", "aws-sdk"]:
    env = dict(os.environ, RAIDEN_ERROR_CORPUS=str(corpus),
               RAIDEN_ERROR_OUTPUT=str(directory / f"{backend}.jsonl"))
    subprocess.run([env.get("CARGO", "cargo"), f"+{args.toolchain}", "test", "--locked",
                    "-p", "raiden", "--no-default-features", "--features", backend,
                    "--test", "operational_compat",
                    "differential_success_response_corpus" if args.responses else "differential_service_error_corpus",
                    "--", "--ignored"], cwd=root, env=env, check=True)
observations = [(directory / f"{backend}.jsonl").read_text().splitlines()
                for backend in ["rusoto", "aws-sdk"]]
assert all(len(rows) == len(cases) for rows in observations)
differences = [{"case": index, "input": case, "rusoto": json.loads(left),
                "aws-sdk": json.loads(right)}
               for index, (case, left, right) in enumerate(zip(cases, *observations))
               if json.loads(left) != json.loads(right)]
(directory / "differences.json").write_text(json.dumps(differences, ensure_ascii=True, indent=2))
print(f"Compared {len(cases)} inputs: {len(differences)} differences. Artifacts: {directory}")
for difference in differences[:3]:
    print(json.dumps(difference, ensure_ascii=True))
raise SystemExit(bool(differences))
