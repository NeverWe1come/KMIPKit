# Quickstart: Validate the Normative Inventory

This guide describes the validation entry point planned by this feature. Run it after the inventory tooling and catalog have been implemented.

## Prerequisites

- Python 3.12 or newer.
- A checkout containing the pinned OASIS documents and normative catalog.
- No network connection or external Python packages are required.

## Validate the catalog

From the repository root, run:

```powershell
$base = git rev-parse origin/release/1.0.0
python -m unittest discover -s tools/normative_catalog/tests -v
python tools/normative_catalog/check_immutable_sources.py --base-sha $base
python tools/normative_catalog/audit_sources.py --check --base-sha $base
python tools/normative_catalog/validate.py
python tools/normative_catalog/report.py --check
```

Expected result: all unit tests pass; validation exits with code 0 and prints the record counts; report check exits with code 0 and confirms that `specification/catalog/coverage-report.md` matches generated output.

## Review source integrity

```powershell
$base = git rev-parse origin/release/1.0.0
python tools/normative_catalog/check_immutable_sources.py --base-sha $base
python tools/normative_catalog/audit_sources.py --check --base-sha $base
```

Expected result: the immutable-source command first proves that the full source tree and manifest match the exact base commit; the auditor then reads only allowlisted local Git blobs and checks their hashes and normative candidate locators. CI obtains the base SHA from the pull-request event, fails closed if absent, and runs with read-only permissions and no secrets. The parser has no network capability.

## Regenerate the report

```powershell
python tools/normative_catalog/report.py --write
python tools/normative_catalog/report.py --check
```

Expected result: the first command writes the deterministic report; the second reports no differences. Catalog changes and generated report updates are reviewed together.

## Negative checks

The test suite includes malformed JSON, duplicate IDs, unresolved cross-references, invalid direction/scope combinations, missing exact source citations, incorrect aggregate counts, absent required negative-test markers, missing fixture status, and nondeterministic report output. Tests use temporary copies and do not modify pinned OASIS sources.
