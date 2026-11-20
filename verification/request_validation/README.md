# Request schema generation

This directory contains the audited input and generator for the explicit KMIP v2.1 Request Payload descriptors used by `kmipkit-protocol` pre-send validation.

- `operation-schema-matrix.md` is the field-level input: operation references, field tags and types, cardinality, nested schema identities, and stable rule IDs. It is the only input imported from the KMIPKIT-0051 development record; planning documents, ADRs, and reports are not needed by the generator.
- `generate.py` converts that matrix into `crates/kmipkit-protocol/src/validation/schemas_generated.rs`. It does not scrape OASIS documents or infer field order from numeric tags.
- `tests/test_generate.py` checks selected parser and descriptor behavior.

From the repository root:

```sh
python verification/request_validation/generate.py --check
python -m unittest verification.request_validation.tests.test_generate
```

OASIS source documents and their checksums are maintained separately under `verification/conformance/oasis/`.
