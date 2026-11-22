# Planned interoperability coverage

Generate the JSON summary or the JSON Lines matrix from the repository root.

```sh
python3 verification/interop/shared/tools/coverage_matrix.py
python3 verification/interop/shared/tools/coverage_matrix.py --jsonl --output /tmp/coverage-matrix.jsonl
python3 verification/interop/shared/tools/coverage_matrix.py --version 3.0 --origin xml
```

Repeat `--version` to select multiple versions. `--origin` accepts `normative`
or `xml`. File export requires the `jsonschema` dependency used in CI. Export
validates each row and reports the SHA-256 of the JSONL output.

The exhaustive 2.x matrix uses a separate generated catalog from the existing
catalog consumed by the validators. The capability manifest records the
review commit used to inspect the public API. To regenerate its inputs, run
`catalog/generate_matrix_catalog.py --write` and
`tools/route_capabilities.py --write` under this directory.

The 3.0 inventory uses the pinned OASIS Working Draft and identifies its
numbered sections, table rows, normative prose and externally referenced
PKCS #11 values. The generator also reads the 102 profile XMLs under
`catalog/oasis/kmip-3.0/`. Each XML row locates a specific exchange and batch
item in its original fixture. A link to a payload field records its presence
in that fixture. Conditional rules, value semantics and absent fields need
their own assertions. The public route rows identify the explicitly cataloged
draft constructors. Other requirements remain visible as inventory rows while
their public routes and executable recipes are mapped.

The matrix expresses planned coverage. Results from running the scenarios
against installed packages belong in execution reports.

## Reference counts

| KMIP version | Origin | Cases or inventory entries | Planned rows |
| --- | --- | ---: | ---: |
| 2.0 | Normative | 2,289 | 154,004 |
| 2.1 | Normative | 2,346 | 159,328 |
| 3.0 | Normative | 5,206 | 5,234 |
| 3.0 | OASIS XML | 102 | 654 |

The 2.x portion retains the 2,689 cases from PR #180 and adds eight cases
for Date Time and Interval forms of Login and Delegated Login lease time. The
review API exposes seven additional routes. The port also applies the
version-specific payload and partial Stop response corrections from the
dependent PRs.

The 3.0 XML cases contain 617 request/response exchanges and show 94 distinct
payload fields. The 28 additional 3.0 normative rows are constructor routes
for the seven draft operations mapped across the four languages.
