# Planned interoperability coverage

Generate the JSON summary or the JSON Lines matrix from the repository root.

```sh
python3 verification/interop/shared/tools/coverage_matrix.py
python3 verification/interop/shared/tools/coverage_matrix.py --jsonl --output /tmp/coverage-matrix.jsonl
python3 verification/interop/shared/tools/coverage_matrix.py --version 2.0 --version 2.1 --origin xml
```

Repeat `--version` to select multiple versions. `--origin` accepts `normative`
or `xml`. File export requires the `jsonschema` dependency used in CI. Export
validates each row and reports the SHA-256 of the JSONL output.

## Where the inputs and generators live

| What you need | Location |
| --- | --- |
| One command for the planned matrix | [`tools/coverage_matrix.py`](tools/coverage_matrix.py) |
| KMIP 2.0 and 2.1 normative cases | [`tools/matrix/kmip2x_catalog.py`](tools/matrix/kmip2x_catalog.py), which generates `catalog/kmipkit-2x-matrix-catalog.json` |
| KMIP 3.0 Working Draft normative inventory | [`tools/matrix/kmip30_inventory.py`](tools/matrix/kmip30_inventory.py), which reads the specification during matrix generation |
| Profile XML scenarios for all three versions | [`tools/matrix/oasis_xml.py`](tools/matrix/oasis_xml.py), which verifies the files under [`catalog/oasis/`](catalog/oasis/) |
| Shared PKCS #11 values | [`tools/matrix/pkcs11.py`](tools/matrix/pkcs11.py) reads the pinned PKCS #11 3.0 headers and supplies values for all three KMIP versions |
| Public library routes | [`tools/matrix/route_capabilities.py`](tools/matrix/route_capabilities.py) records the review commit and the available public entry points |
| Pinned specifications and profiles | [`verification/conformance/oasis/`](../../conformance/oasis/) |

The installed validators use [their own execution catalog](catalog/kmipkit-1.0-test-catalog.json).
To regenerate the planning catalog and route manifest, run these commands
from the repository root.

```sh
python3 verification/interop/shared/catalog/generate_matrix_catalog.py --write
python3 verification/interop/shared/tools/route_capabilities.py --write
```

Each XML row locates an exchange and batch item in its original fixture.
A link to a payload field records its presence. Conditional rules, value
semantics and absent fields need their own assertions. The KMIP 3.0 public
route rows identify the explicitly cataloged draft constructors.

The matrix expresses planned coverage. Results from running the scenarios
against installed packages belong in execution reports.

## Reference counts

| KMIP version | Origin | Cases or inventory entries | Planned rows |
| --- | --- | ---: | ---: |
| 2.0 | Normative | 2,289 | 154,004 |
| 2.0 | OASIS XML | 93 | 613 |
| 2.1 | Normative | 2,346 | 159,328 |
| 2.1 | OASIS XML | 93 | 613 |
| 3.0 | Normative | 5,206 | 5,234 |
| 3.0 | OASIS XML | 102 | 654 |

The 2.x portion retains the 2,689 cases from PR #180 and adds eight cases
for Date Time and Interval forms of Login and Delegated Login lease time. The
review API exposes seven additional routes. The port also applies the
version-specific payload and partial Stop response corrections from the
dependent PRs.

The 2.0 and 2.1 XML corpora each contain 578 request/response exchanges.
The 3.0 XML corpus contains 617. In total, 288 XML cases contribute 1,773
exchanges and 1,880 planned rows to the 320,446-row matrix. The 28 additional 3.0 normative rows are
constructor routes for the seven draft operations mapped across the four languages.
