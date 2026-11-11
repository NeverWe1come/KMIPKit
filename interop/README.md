# KMIPKit Interoperability Validators

This directory contains independent Rust, C, Java, and Python consumers for
observing KMIPKit behavior against KMIP 2.0 and 2.1 servers. A profile is a
planning hint only; it is never evidence that a server supports a capability.
Runs probe every version-applicable case and record the exact server and
configuration. These suites do not make certification or blanket profile claims.

## Projects

- `shared/`: schemas, versioned test catalog, vectors, profiles, and tools.
- `rust-kmip-validator/`: independent Rust consumer.
- `c-kmip-validator/`: independent C ABI consumer.
- `java-kmip-validator/`: independent Java consumer.
- `python-kmip-validator/`: independent Python consumer.

## Offline contract checks

```bash
python3 interop/shared/catalog/generate_test_catalog.py --check
python3 interop/shared/vectors/generate_vectors.py --check
python3 -B -m unittest discover -s interop/shared/tests -p 'test_*.py' -v
python3 interop/shared/tools/coverage_matrix.py
```

The catalog targets KMIP 2.0 and 2.1 and records each case as planned,
implemented, or not applicable. A planned case is not a test result. The
contract also records that the current public API has no generic-TTLV outbound
request path; a test of that API level must not be reported as server
incompatibility.

The source-audited inventory currently emits 2,560 normative targets and
2,689 planned cases. It includes every inventoried operation payload row,
explicit TTLV type variants found in payload descriptions, the PKCS #11 3.0
function offsets and return codes referenced by KMIP, and explicit batch and
asynchronous state scenarios. Those scenarios cover Stop, Continue and Undo;
batch order and count defaults; placeholder ordering and response correlation;
Pending-to-terminal polling; all five Cancellation Result values; cancellation
races; Process; and outstanding-request filtering. It also requires every
combination of the three assigned masks (81,928 values per KMIP version). The
catalog generator pins those external OASIS header hashes and fails offline if
they change. `coverage_matrix.py` expands applicable cases across each
language's installed public construction, serialization, response-reading, and
execution routes, protocol version, encoding, and transport. Every row retains
the normative target and any ordered workflow step or repeated batch item
index. Add `--jsonl` to stream the rows or `--jsonl --output PATH` to write
and schema-check the deterministic JSONL artifact. The manifest currently tracks
1,479 exact public API entrypoints plus 16 separate execution entrypoints,
expanding to 273,824 route-action/target/workflow cells. The matrix SHA-256 is
`3e0979f5b80bf68eb0d0bfca349e290e11866ec3e0a58d100e781c0238d89263`. This matrix is a
coverage obligation, not evidence that a route was called. Each cell remains a
required implementation task until the corresponding validator records and
verifies its execution. These counts do not mean that validators or live KMS
runs are complete.

`validate_route_evidence.py` checks runner events against each planned cell and
the installed package digest. Its four-language `--parity` mode requires all
four complete matrices to identify the same KMIP server/configuration, library
commit, catalog, and route manifest. It compares normalized request/response
trees across matching actions and checks that a request constructed for a
network route is the request sent. Batch cases with per-item expected outcomes
must report the peer-observed KMIP Result Status/Reason codes (and effects when
the scenario specifies them). Missing four-language tree groups make the
parity command fail. The older run aggregator may compare declared result
fields, but it does not independently verify installed-package provenance;
package provenance is checked through the route artifact probe and route
evidence validator.

Each validator must record construction, encoding, actual network send, and
response decode as separate route events. Events name the exact public API
symbol for the action and, for network events, the separate public execution
symbol; they include an invocation ID and store
only SHA-256 hashes of normalized trees and (when deterministic) wire bytes.
They must never contain KMIP bodies or key material. The checker compares the
report's artifact digest with the digest supplied by the package installer,
and validates every selected cell against the catalog, route manifest, and
server identity. Several run reports can be combined to cover all protocol
versions and encoding/transport pairs:

```text
python3 interop/shared/tools/coverage_matrix.py --jsonl --output /tmp/coverage-matrix.jsonl
python3 interop/shared/tools/validate_route_evidence.py \
  --report results-ttlv-raw-tls.json \
  --report results-json-https.json \
  --artifact-sha256 INSTALLED_ARTIFACT_SHA256
```

The checker validates the runner's report assertions against the exact
catalog, route manifest, installed artifact digest, selected server
configuration, and every matrix cell. The language CI must generate these
events from test hooks placed around the actual public API calls and the
controlled peer's parsed request; a hand-authored report is not invocation
evidence. Batch events include the ordered operation list observed by the
peer and the matching response count. Only reports emitted by validators
after those checks can satisfy route obligations.

## Preparing a live run

Copy `shared/profiles/connection.example.json` and set the protocol version,
endpoint, transport, encoding, and API level. TLS certificate and key paths
must point to files readable by the test process. KMIP username and password
are read from the named environment variables, never from the JSON file.
Set `server_configuration_fingerprint` to a stable 64-character lowercase
SHA-256 identifier for the deployed server configuration. Reuse it across
client protocol, transport, encoding, API, and language variants; change it
when the server software version or server-side configuration changes.
Keep `environment.kind` set to `dedicated_test` and retain its explicit
confirmation. Mutating cases remain disabled until
`execution_policy.allow_mutations` is set to `true`; the validators must reject
live mutation runs without both safety conditions. Do not target a shared or
production key manager.

Build artifacts from one exact `release/1.0.0` commit and install each
language's artifact into an isolated consumer environment. Keep the commit and
artifact SHA-256 with every result. The matrix groups evidence by server
configuration and KMIP version. It requires matching server identity/version,
configuration fingerprint, catalog hash, and KMIPKit source commit for
comparability; package artifact hashes may differ by language. Every run stays
visible with its transport, encoding, API level, and language. Compare
normalized KMIP semantics across those routes; byte equality applies only to
deterministic requests using the same encoding.

Server results remain evidence for the exact product version, server
configuration, protocol version, transport, encoding, and executed case. A
failure does not by itself establish whether the problem is KMIPKit, the
server, or a local policy; reports retain those dimensions separately.
