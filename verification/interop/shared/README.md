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

## Cosmian/Eviden KMIP 2.x table coverage

Generate the version-specific catalog and table obligations before running the
validators. Pass that same catalog to the campaign when importing its evidence.

```sh
python3 verification/interop/shared/tools/cosmian_minimum.py --version 2.1 \
  --catalog-output /tmp/cosmian-2.1-catalog.json --inventory-output /tmp/cosmian-2.1-inventory.json
python3 verification/interop/shared/tools/campaign.py --version 2.1 --cosmian-minimum \
  --catalog-source /tmp/cosmian-2.1-catalog.json --controlled-report /tmp/kmip/results.json \
  --output-dir /tmp/cosmian-campaign
```

The campaign writes `cosmian-2.1.json` with the table expectations and matching
case results by language and construction API. A row can have several case
results with distinct assertions. Cases requiring a provider fixture or a
runtime version boundary remain visible in the inventory.

The four installed-package CI jobs run `table_ttlv.py` and upload the local
fixture reports. Each report records construction, decoding, and the public
version gate with the package digest. The fixtures exercise the eight
applicable managed objects, nine transparent key formats, 22 applicable
base objects, and 56 standard attributes at 2.0 or 63 at 2.1. Additional
messages cover the operation variants absent from the OASIS profile
scenarios. The version gate checks the seven Rotate tags in both versions.
The legacy fixtures check the reserved Template tags and the former Object
Type and Key Format Type values. Import each language's report with a separate
`--table-ttlv-report` argument alongside `--cosmian-minimum`. The table
report keeps local fixture checks and deployed KMS observations in distinct
fields. When importing `--oasis-construction-report` or `--oasis-ttlv-report`,
each applicable operation row includes the matching OASIS case, step and
batch item. Repeat the argument to import reports from separate languages.

The same CI jobs publish `table-key-material.json` for the format-aware
constructors and readers of the nine transparent key formats. They compare
the complete output and reject missing, duplicate, and mistyped material
fields. Pass `--table-key-material-report` to import these checks into the
typed-protocol cells.

## Campaign plans and evidence reports

From the repository root, export the obligations and open the generated
`index.html` to browse them. The HTML report works directly from disk.

```sh
python3 verification/interop/shared/tools/campaign.py --output-dir /tmp/kmip-plan
```

`plan.jsonl` adds an `obligation_id` to each coverage row. The identifier is the
SHA-256 of the canonical JSON row. The campaign fingerprint is the SHA-256 of
those binary identifiers concatenated in plan order. Export a fresh plan
when the inventory, fixtures or route manifest change.

A language consumer writes one
[campaign evidence file](schemas/campaign-evidence.schema.json) per scope.
Use `local` for public construction, codecs and accessors that run without
an exchange. Use `controlled` for assertions on a deterministic peer and
`live` for observations on a deployed KMS. Live evidence identifies the
server configuration. Provide the actual encoding and transport for every
controlled or live event. XML scenarios reference the ordered step and batch
item through their obligation IDs. The API symbols in their events identify
the public routes exercised by that consumer.

The optional parity group digest identifies the same normalized test
input and assertion across languages. Add the digest of the normalized
observed semantics only for deterministic, public fixture values. Compare
equivalent targets, preserve batch ordering and normalize KMIP status and
reason values to their wire codes. Keep credentials, raw KMIP messages
and generated key material out of the evidence file.

Aggregate evidence from a common plan in one report. Repeat the version
selection used when exporting the plan, if any.

```sh
python3 verification/interop/shared/tools/campaign.py \
  --profile verification/interop/shared/profiles/cosmian-kms.json \
  --output-dir /tmp/kmip-campaign /path/to/evidence-rust.json /path/to/evidence-java.json
```

The outputs include a summary in the terminal, `summary.json`,
`summary.md`, the complete `checks.jsonl` and a paginated HTML table.
`--require-complete` fails if a selected version and language lack coverage,
an applicable obligation lacks successful local or controlled evidence, or
normative inventory remains unmapped. Live observations remain visible
alongside that coverage. Expected rejections and policy restrictions can be
recorded independently of the assertion outcome.

### Runs from the installed validators

Build and install the four language consumers first, following their own
README files. Then run their execution catalogs against one configured test KMS.

```sh
python3 verification/interop/shared/tools/run_all.py \
  --connection /path/to/connection.json --profile /path/to/profile.json \
  --output-dir verification/interop/results/kms-2.1 --campaign
```

Supply `--project LANGUAGE PATH` for consumers built in separate worktrees
and `--python PATH` for the interpreter with the installed KMIPKit wheel.
The output directory has one fresh report per language, consolidated
catalog reports, and `campaign/index.html`. Repeat `--connection` with one
file per protocol, transport, encoding and API selection to group multiple
configurations in one report. Pass `--enable-mutations` when the configured
connections opt into mutations.

Import previous `results.json` reports by repeating `--controlled-report`
or `--live-report` on `campaign.py`. The report displays these results by
catalog case, language and execution settings. An individual planned obligation
receives a result when the consumer supplies evidence for its obligation ID.

### Generic TTLV decode checks from OASIS scenarios

`tools/oasis_ttlv.py` converts the pinned XML request and response messages
into generic TTLV vectors with deterministic bindings for fixture symbols.
With `--runner LANGUAGE EXECUTABLE`, the installed library decodes the vectors
and the tool compares the ordered tags, types, child counts and scalar values
for each message. The output retains fixture digests and artifact provenance.

Supply the Rust `kmip30-campaign` binary, the C `kmip30_c_constructors` binary,
the Java `tools/run_draft_campaign.sh` script, or the Python
`tools/run_campaign.sh` script for the respective language. Repeat
`--runner` to check multiple languages. Run the Python wrapper with the
installed wheel environment activated.

```sh
python3 verification/interop/shared/tools/oasis_ttlv.py \
  --runner rust verification/interop/rust-kmip-validator/target/debug/kmip30-campaign \
  --output /tmp/oasis-ttlv.jsonl --require-complete
python3 verification/interop/shared/tools/campaign.py \
  --oasis-ttlv-report /tmp/oasis-ttlv.jsonl --output-dir /tmp/kmip-campaign
```

Use `--construct --version 2.0 --version 2.1` with the same installed runners
to build each complete fixture from its raw tag numbers and scalar values
through the language's public generic item constructors. The runner walks
the constructed model and the tool checks its ordered tags, types, child
counts and scalar values. Each runner also re-encodes the constructed message
with the same serializer used for direct-tag exchanges and compares the bytes
against the fixture. Import this output into the campaign using
`--oasis-construction-report`. These checks exercise construction locally.

Each XML scenario step shows its request and response codec outcomes.
Scenario execution remains pending until it has controlled or live
exchange evidence.

To launch a campaign-capable consumer, supply `--runner LANGUAGE EXECUTABLE`
once per language and specify the scope. The runner is invoked directly
with `--plan`, `--matrix-sha256`, `--scope` and `--evidence`. A live campaign
takes `--connection`, enforces the configured version and checks the server
identity on the returned evidence. The optional profile is passed with
`--profile` and supplies capability declarations to the report. A runner
should write its evidence before exiting with a failure. Run a single
language with one executable or repeat the option for multiple languages.

### Receiving server-initiated operations

For KMIP 2.0 or 2.1 with `api_level: generic-ttlv`, `transport: raw_tls` and
`encoding: TTLV`, the four standalone validators can receive Discover Versions,
Notify, Put, Query and Set Endpoint Role over a mutually authenticated TLS
connection. The C listener uses the Unix OpenSSL build.

Provide the following options on the validator's `run` command:

```sh
--listen 127.0.0.1:0 --listener-cert /path/to/listener-cert.pem \
  --listener-key /path/to/listener-key.pem --listener-ready-file /path/to/address.txt
```

Select the incoming case ID with `--case` for one external trigger. When running
several incoming cases, configure the peer to initiate the operations in catalog
order. The readiness file reports the bound address. The configured CA must
trust the certificate presented by the connecting peer. Incoming cases retain
their mutation policy, so Java and Python also use `--enable-mutations` and C
uses `--allow-mutations` for the applicable cases.

The controlled peer suite exercises the installed executors, including reply
fields, correlation and a malformed Put. It also checks the exact reference
and returned value for Cryptographic Algorithm, Cryptographic Length,
Cryptographic Usage Mask, Object Type, State, Sensitive, Always Sensitive,
Extractable and Never Extractable on isolated keys. The multipart SHA-256
case checks Init, Update and Final, including the server-provided correlation
value and final digest, through direct tags, Typed Protocol and Fluent.
Run the suite with the installed Python wheel available and the Rust, C
and Java validators built:

```sh
python -m unittest verification.interop.shared.tests.test_incoming_validators -v
```

## Operation variants and API comparisons

Export the request and response field obligations and the managed-object
attribute rules for KMIP 2.0, 2.1 and 3.0.

```sh
python3 verification/interop/shared/tools/operation_variants.py --output /tmp/kmip-operation-variants.jsonl
```

Each row includes its source section, presence or cardinality branch, and
stable ID. These rows describe cases to implement. The distinct
`requiredness_review` branch identifies a rule requiring manual review.

Run the validator twice with the same connection settings and catalog,
selecting `api_level: generic-ttlv` in one configuration and
`api_level: fluent` in the other. The report also compares `typed-protocol`
with `fluent` when both runs are available. Keep the artifact and server configuration
the same. Then combine their reports.

```sh
python3 verification/interop/shared/tools/aggregate_matrix.py \
  /path/to/typed/results.json /path/to/fluent/results.json \
  --format html --output /tmp/kmip-api-comparison.html
```

The paired columns show outcome agreement per case. `MATCHED_PASS` records
the same successful assertion on both routes. `MATCHED_UNSUPPORTED` records
the same server rejection. Cases omitted, withheld by policy, or validated
locally without a server exchange remain pending in the paired columns.


## Full operation selection

Generate an exchange catalog for the target protocol version. Use the
generated file as the validator's `--catalog` input for a dedicated test
endpoint. Each row carries its exact specification section and direction.

```sh
python3 verification/interop/shared/tools/operation_catalog.py --version 3.0 --output /tmp/kmip-operations-3.0.json
```

Run the complete selections in one campaign by passing a connection file
per version and API level.

```sh
python3 verification/interop/shared/tools/run_all.py --all-operations --campaign \
  --connection /path/to/2.0.json --connection /path/to/2.1.json \
  --connection /path/to/3.0.json --output-dir /tmp/kmip-operation-run
```

The catalogs include 56 operation/direction pairs for 2.0, 62 for 2.1,
and 69 for the 3.0 Working Draft. Existing recipes are selected once for
each direction. A planned row requires a tested recipe in the chosen
consumer before the validator can report exchange evidence. A server
rejection or a local delivery error remains visible in its result.

Add `--with-variants` to either command to include the catalogued
attribute, object, cryptography, batch and boundary cases alongside the
operation cases. This selection retains the variants explicitly assigned
to each version. KMIP 3.0 receives its operation cases and any variants
explicitly assigned to 3.0 in the shared catalog.

Select an exact operation name with `--operation` or an exact catalog ID
with `--case`. Both options are repeatable and take the union of matches.
Operation selection retains every matching direction. Both
`operation_catalog.py` and `run_all.py --all-operations` accept these
selectors. The generated catalog can also be passed to a standalone
validator through its `--catalog` option. Each standalone validator also
accepts `run --case ID` to execute one case from that catalog. The result
keeps the original catalog digest so that runs of the same case can be compared.

Add `--require-complete` to a live `run_all.py --all-operations` run to
require evidence for every applicable selected case in each run. An
observed KMIP rejection for an unsupported operation accounts for the
execution while retaining its server compatibility status. A missing
case, a blocked case, or a failed local assertion fails the check.
The `completion.json` report lists the missing case IDs for each validator and
records reports whose catalog, server or execution route differs from the selection.

## Cosmian and Eviden KMS support scope

Select the KMIP 2.x cases associated with the published support table. The
inventory records the declared support, specification targets, local model
cases and planned server exchanges for each published row.
[Cosmian and Eviden KMS support table](https://docs.cosmian.com/key_management_system/kmip_support/support.html)

```sh
python3 verification/interop/shared/tools/run_all.py --all-operations --cosmian-minimum \
  --dry-run --connection /path/to/2.0.json --connection /path/to/2.1.json \
  --output-dir /tmp/kmip-cosmian-scope
```

Remove `--dry-run` to execute against a dedicated endpoint. Pass one
connection per protocol version and API level you plan to exercise. Use
`--profile` with a capability profile whose server ID matches the connection.
Use `--case` or `--operation` to run a subset. Use `--require-complete` for
a live run to require exchange evidence and, for the entire selection,
report published rows that lack any mapped server case. Inspect
`catalogs/kmip-*-cosmian-scope.json` for local-only rows and vendor
extensions that still require fixtures. Runtime reports identify cases
without an executable route in the chosen binding.

Use `--normative` with `--all-operations` to select the full KMIP 2.0 and
2.1 normative matrix and the existing executable variants. This includes
the attribute policies, protocol fields, object structures, enumerations
and bitmasks defined in the specification catalog. A broader normative
obligation sharing an identifier with an existing baseline case receives a
`-normative` suffix and keeps its own assertions.

To include incoming operations in a coordinated run, pass
`--listen`, `--listener-cert` and `--listener-key` to `run_all.py`.
Each validator publishes its bound address in `listener-address.txt` in
its result directory for the peer trigger on KMIP 2.x raw TLS connections.
For `Notify` and `Put`, select `typed-protocol` to exercise the installed
listener and dispatcher or `generic-ttlv` to exercise the tag constructors.
