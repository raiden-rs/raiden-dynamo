#!/usr/bin/env python3
"""Prevent the dependency versions behind the November 2026 alerts returning.

This checks the known advisories addressed here, not a general advisory audit.
"""

from pathlib import Path
import re
import sys
import tomllib


MINIMUMS = {
    "aws-smithy-json": ((0, 62, 7), "GHSA-8ffr-xgwf-xj56"),
    "ring": ((0, 17, 12), "GHSA-4p46-pwfr-66x6"),
    "rustls": (
        (0, 23, 46),
        "GHSA-2mjx-qc3c-rqvc, GHSA-c8m5-c952-6cm4, "
        "GHSA-h2v8-394m-p7vv, GHSA-w44r-93j7-qv28",
    ),
}


def main():
    root = Path(__file__).resolve().parent.parent
    locks = [root / "Cargo.lock", root / "raiden/tests/downstream-backends/Cargo.lock"]
    failures = []
    for lock in locks:
        with lock.open("rb") as source:
            packages = tomllib.load(source)["package"]
        for package in packages:
            if package["name"] not in MINIMUMS:
                continue
            minimum, advisories = MINIMUMS[package["name"]]
            version = package["version"]
            numbers = tuple(map(int, re.match(r"^(\d+)\.(\d+)\.(\d+)", version).groups()))
            if numbers < minimum or (numbers == minimum and "-" in version):
                failures.append(
                    f"{lock.relative_to(root)}: {package['name']} {version} "
                    f"must be upgraded for {advisories}"
                )
    if failures:
        print("\n".join(failures), file=sys.stderr)
        return 1
    print(f"No known vulnerable versions in {len(locks)} lockfiles.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
