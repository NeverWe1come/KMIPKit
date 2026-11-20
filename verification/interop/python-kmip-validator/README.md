# KMIPKit Standalone Python Validator

Independent KMIP 2.1 validator project in Python.
Tests KMS implementations and client bindings against the canonical KMIP 2.1 test catalog.

## Usage

```bash
# Plan run (check supported vs unsupported for a KMS)
python3 -m kmipkit_validator plan --capabilities ../shared/profiles/cosmian-kms.json

# Execute run and generate matrix reports
python3 -m kmipkit_validator run --capabilities ../shared/profiles/cosmian-kms.json --output-dir ./results
```
