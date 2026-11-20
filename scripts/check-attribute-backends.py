#!/usr/bin/env python3
"""Seeded differential exploration; persists corpus/results and fails on any difference.

Uses no AWS credentials/network. SDK error text and JSON map ordering are excluded.
The corpus contains single-tag DynamoDB values (including service-invalid empty
sets/NULL:false), nested containers and canonical optional capacity metadata.
"""
import argparse
import base64
import json
import os
from pathlib import Path
import random
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--seed", type=int, default=20261120)
parser.add_argument("--cases", type=int, default=10000)
parser.add_argument("--output-dir", type=Path)
parser.add_argument("--include-malformed", action="store_true",
                    help="also explore malformed raw AttributeValue JSON at the root and nested in L/M")
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
directory = args.output_dir or Path(tempfile.mkdtemp(prefix="raiden-differential-"))
directory.mkdir(parents=True, exist_ok=True)
rng = random.Random(args.seed)


def text():
    return "".join(rng.choices(["a", "0", " ", "\x00", "\\", '"', "あ", "😀", "\n"], k=rng.randrange(12)))


def number():
    return rng.choice([str(rng.randrange(-(2**80), 2**80)), "0", "-0", "1.5", "1e20",
                       "NaN", "Infinity", "", "invalid", str(2**64 - 1), str(-(2**63))])


def binary():
    return base64.b64encode(bytes(rng.randrange(256) for _ in range(rng.randrange(16)))).decode()


def attribute(depth=0):
    if args.include_malformed and rng.randrange(4) == 0:
        return rng.choice([{}, {"S": "a", "N": "1"}, {"UNKNOWN": "value"},
                           {"NULL": None}, {"B": "!invalid"}, {"L": [None]},
                           {"S": 1}, {"NS": [1]}, {"BOOL": "false"},
                           {"S": "a", "UNKNOWN": "value"}, {"S": None},
                           {"L": None}, {"M": None}])
    tag = rng.choice(["S", "N", "B", "BOOL", "NULL", "SS", "NS", "BS"] +
                     (["L", "M"] if depth < 4 else []))
    if tag == "S":
        value = text()
    elif tag == "N":
        value = number()
    elif tag == "B":
        value = binary()
    elif tag in ("BOOL", "NULL"):
        value = bool(rng.randrange(2))
    elif tag in ("SS", "NS", "BS"):
        generate = {"SS": text, "NS": number, "BS": binary}[tag]
        value = [generate() for _ in range(rng.randrange(5))]
    elif tag == "L":
        value = [attribute(depth + 1) for _ in range(rng.randrange(5))]
    else:
        value = {text(): attribute(depth + 1) for _ in range(rng.randrange(5))}
    return {tag: value}


def capacity():
    return {key: rng.choice([None, 0.0, 0.5, 1.0, 1e10])
            for key in ["CapacityUnits", "ReadCapacityUnits", "WriteCapacityUnits"]
            if rng.randrange(2)}


def fixture():
    metadata = capacity()
    if rng.randrange(2):
        metadata["TableName"] = rng.choice([None, "", text()])
    for key in ["Table", "LocalSecondaryIndexes", "GlobalSecondaryIndexes"]:
        if rng.randrange(2):
            metadata[key] = rng.choice([None, capacity() if key == "Table" else {text(): capacity()}])
    consumed = rng.choice([None, [], [metadata]])
    raw = attribute()
    return {"consumed_capacity": consumed,
            "unprocessed_items": [{"Item": {"raw": raw}}]}


corpus = directory / "corpus.jsonl"
with corpus.open("w") as stream:
    for _ in range(args.cases):
        stream.write(json.dumps(fixture(), ensure_ascii=True) + "\n")
print(f"seed={args.seed} cases={args.cases} artifacts={directory}", flush=True)
for backend in ["rusoto", "aws-sdk"]:
    env = dict(os.environ, RAIDEN_FUZZ_CORPUS=str(corpus),
               RAIDEN_FUZZ_OUTPUT=str(directory / f"{backend}.jsonl"))
    subprocess.run([env.get("CARGO", "cargo"), "+1.88.0", "test", "-p", "raiden",
                    "--no-default-features", "--features", backend,
                    "--test", "attribute_differential_fuzz", "--", "--ignored"],
                   cwd=root, env=env, check=True)
inputs = corpus.read_text().splitlines()
left = (directory / "rusoto.jsonl").read_text().splitlines()
right = (directory / "aws-sdk.jsonl").read_text().splitlines()
assert len(inputs) == len(left) == len(right) == args.cases
mismatches = []
panics = {backend: sum("\"panic\":true" in row.replace(" ", "") for row in rows)
          for backend, rows in [("rusoto", left), ("aws-sdk", right)]}
print(f"Inputs with a panicking read conversion: {panics}")
for index, (a, b) in enumerate(zip(left, right)):
    if json.loads(a) != json.loads(b):
        mismatches.append({"case": index, "input": json.loads(inputs[index]),
                           "rusoto": json.loads(a), "aws-sdk": json.loads(b)})
# Report the smallest observed counterexamples first; no shrinking claim.
mismatches.sort(key=lambda v: len(json.dumps(v["input"])))
(directory / "differences.json").write_text(json.dumps(mismatches, ensure_ascii=True, indent=2))
print(f"Compared {args.cases} inputs: {len(mismatches)} differences. Artifacts: {directory}")
for mismatch in mismatches[:3]:
    print(json.dumps(mismatch, ensure_ascii=True))
raise SystemExit(bool(mismatches))
