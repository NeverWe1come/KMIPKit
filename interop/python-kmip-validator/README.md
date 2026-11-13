# KMIPKit Python interoperability validator

This is an independent Python 3.12 consumer of the KMIPKit wheel built from
the exact release commit in `library-pin.json`. It imports the installed wheel
and does not import the library from the source checkout. The catalog and
connection/result contracts are shared with the other language validators.

## Install and inspect

```sh
python3.12 -m venv .venv
. .venv/bin/activate
python -m pip install --only-binary ':all:' maturin==1.9.4 cffi==2.0.0 jsonschema==4.23.0
rustup toolchain install 1.94 --profile minimal
python tools/install_artifact.py
python tools/verify_artifact.py
```

`plan` and `preflight` are offline. Plans enumerate the cases applicable to
KMIP 2.0 or 2.1, including API routes that the pinned package cannot issue.
Capability profiles are hypotheses; they do not suppress probes.

```sh
PYTHONPATH=src python -m kmipkit_validator plan \
  --connection ../shared/profiles/connection.example.json
PYTHONPATH=src python -m kmipkit_validator preflight \
  --connection path/to/connection.json
```

## Live runs

`run` requires an explicit server configuration, a verified installed
artifact, TLS credentials and a dedicated test environment. State-changing
cases need both the connection policy and `--enable-mutations`:

```sh
PYTHONPATH=src python -m kmipkit_validator run \
  --connection path/to/connection.json \
  --capabilities path/to/capabilities.json \
  --enable-mutations \
  --output-dir results
```

The typed client recipes cover 140 server-exchange catalog rows across the
two protocol versions, including asynchronous operations. The high-level
route exercises the package's public attribute and asynchronous request APIs.
Local checks cover every applicable catalog row: all TTLV data type
constructors and readers, enumeration and mask values, tag allocation ranges,
structure and payload field types, and the checked-in valid and malformed wire
vectors. They also build every advertised typed `ClientBatchItem` and exercise
controlled TLS exchanges, authentication, status interpretation and
version-gated requests. For the seven attribute operations with both public
routes, tests send the high-level and typed requests through the installed
client and require identical TTLV request and response bytes for the same
deterministic input.

`plan` and `preflight` identify the same unavailable routes and local checks.
The pinned Python package has no public server-initiated request handler,
multipart Init/Update/Final client workflow, Batch ID Placeholder builder,
offline request validator, encoded Request Header view, or outbound
generic-TTLV client request API. Therefore the version rejection checks under
KMIP 2.0, empty-batch validation, single-item header-option inspection, and
placeholder construction are reported as documented API gaps. The Python API
also does not expose independent high-level request models for the full
operation catalog. These cases remain `not_run_documented_api_gap`; they are
never counted as server incompatibility. Cross-language normalized
request/response parity is not claimed by this validator run.

Report output is written beneath the current working directory. The artifact
installer writes wheels and its provenance manifest beneath the validator
project directory; paths that escape those roots are rejected.

Reports use shared schema 2.0 and produce JSON, Markdown and HTML. They
separate KMIPKit correctness, observed server compatibility, policy
restrictions, parity and pending coverage. Credentials, key material, TTLV
bodies and remote error text are excluded.

CI installs the pinned wheel in Python 3.12 and runs contract tests with
controlled local TLS fixtures. This verifies the consumer and its request
handling; it is not evidence of interoperability with Cosmian KMS, Thales
CipherTrust or ESKM. Vendor interoperability remains to be observed by running
against a deployed KMIP 2.0 or 2.1 endpoint.

## Exhaustive route-matrix closure (KMIPKIT-0092)

Run the public construction-route matrix independently of a vendor connection:

```sh
PYTHONPATH=src python -m kmipkit_validator matrix --output-dir build/matrix
python ../shared/tools/validate_route_evidence.py \
  --report build/matrix/route-execution-evidence.json \
  --artifact-sha256 <artifact_sha256-from-build/artifact-manifest.json>
```

The runner expands the shared catalog and capability manifest through the
shared generator. It verifies the recorded wheel digest, checks the installed
archive identity, and compares every installed `kmipkit/` file with that wheel.
Each positive row requires observing
the named installed public function body. Common synchronization wrappers do
not count as evidence of another public function. Network rows also require
observing the named executor. Every row retains the target, workflow indices,
KMIP version, route, encoding, transport and exact artifact provenance.

The runner compares public TTLV constructors and readers with public decoding
of independently encoded fixture bytes. All eleven TTLV value types, assigned
and unknown enumeration values, masks, attributes, object structures,
operation payloads, message fields and boundary vectors have executable
recipes. The installed package has no public generic encoder: the small
fixture encoder belongs to the validator and is never labeled as a public
library encoder.

The latest installed-package matrix run completed all **74,648/74,648** cells:
45,470 local construction/encoding/decoding cells and 29,178 controlled-peer network
cells across 823 catalog cases. It covers KMIP 2.0 and 2.1, TTLV/JSON/XML and
raw TLS/HTTPS with TLS 1.3 mutual authentication. All 74,648 invocations
passed; there were no duplicate or pending cells. The controlled peer checks
the received request and response shapes, ordered batch items, protocol
version, result status, state changes, asynchronous flows and cleanup where
the case requires them. A passed matrix cell means the installed library route
was exercised and its controlled fixture assertions held; it does not claim
that every KMIP server supports the operation or that every server response
will be successful. Reports retain normalized tree and deterministic wire
hashes, never raw bodies or fixture values.

The generated `closure.json` and `pending-coverage.json` contain the exact
counts for each run. The independent shared validator also passed against the
same pinned library commit, artifact digest, catalog and route manifest, with
zero duplicate events. It exits nonzero for a missing or duplicate cell,
incorrect public symbol, failed invocation or incomplete scope. The installed
package has no public generic outbound TTLV request API; that API boundary
remains documented in the shared route manifest and is not presented as an
executed route. The matrix uses typed and high-level public requests wherever
the package exposes them.

The KMIP 2.0 version-boundary recipes prove native `NotSent` delivery against
a listening endpoint with zero queued connections in every network
combination. They retain a hash of the explicit public request inputs and emit
no response tree or wire-byte claim. The common contract distinguishes these
construct/send obligations from response decoding. Reserved and unused Tag
allocations use their expected-rejection semantics for checked constructors.

| Requirement | Python evidence and remaining scope |
|---|---|
| KMIPKIT-0092-FR-001/002/007 | 74,648 required route cells executed; strict closure and independent shared validation passed with no pending cells. |
| KMIPKIT-0092-FR-003/004 | Generic TTLV, typed model and request/response route trees are constructed, decoded and checked through the installed package. |
| KMIPKIT-0092-FR-005 | Controlled TLS peers cover all six version/encoding/transport combinations and catalogued operation, batch, stateful and asynchronous cases. |
| KMIPKIT-0092-FR-006 | Exact release wheel digest plus installed-file verification. |
| KMIPKIT-0092-FR-008/009 | Sanitized JSON evidence and fixed diagnostic codes; no raw request/response bodies or fixture secret values. |

Cross-language parity still requires the joint run to consume matching reports
from all four language branches. This Python report alone makes no parity claim.

These controlled fixtures establish no vendor interoperability or certification.
Python-owned temporary copies cannot be zeroized reliably; use only the
checked-in deterministic fixture values when running this command.
