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
shared generator. It verifies the recorded wheel digest and compares every
installed `kmipkit/` file with that wheel. Each positive row requires observing
the named installed public function body. Common synchronization wrappers do
not count as evidence of another public function. Network rows also require
observing the named executor. Every row retains the target, workflow indices,
KMIP version, route, encoding, transport and exact artifact provenance.

The runner compares public TTLV constructors and readers with public decoding
of independently encoded fixture bytes. All eleven TTLV value types and
numeric enumeration/mask representatives have executable generic recipes.
The installed package has no public generic encoder: the small fixture encoder
belongs to the validator and is never labeled as a public library encoder.
Full positive network evidence currently covers Ping in KMIP 2.1, using
TLS 1.3/mTLS for TTLV, JSON and XML over raw TLS and HTTPS. The complete parsed
request and response shapes and numeric Result Status are checked. The
normalized response excludes only Time Stamp; ordered children, operations,
versions and result codes remain intact. Reports retain tree and deterministic
wire hashes, never raw bodies or fixture values.

This is a partial implementation of the exhaustive validator. Typed model,
attribute/object, stateful positive operation, batch and asynchronous matrix
recipes remain pending. Earlier controlled Operation Not Supported tests prove
transport and status handling; they do not satisfy these positive matrix
obligations. Missing recipes appear in `pending-coverage.json`, with full axes
and a fixed sanitized failure code. They are never converted into API
unavailability or a positive row. `closure.json` records the exact verified and
pending counts. The command and the independent shared gate exit nonzero for
any missing or duplicate cell, incorrect public symbol, failed invocation or
incomplete scope. CI uploads these three sanitized artifacts even when the
completeness gate fails; the PR must remain `[WIP]`.

A shared-contract conflict also remains: the matrix currently requires a peer
response decode for KMIP 2.0 requests that must be rejected before connecting
(for example `test-v20-reject-ping`). The validator does not fabricate a
response for those obligations. Reserved/unused Tag allocation cases similarly
need expected-rejection semantics for checked constructors. Resolving those
applicability rules belongs to the common contract; it does not authorize
changing library behavior or accepting those invalid inputs.

| Requirement | Python evidence and remaining scope |
|---|---|
| KMIPKIT-0092-FR-001/002/007 | Shared matrix expansion; strict closure and independent shared validation; missing recipes remain pending. |
| KMIPKIT-0092-FR-003/004 | Generic numeric/type roundtrips and ordered normalized Ping trees; other named routes remain pending. |
| KMIPKIT-0092-FR-005 | Positive Ping controlled peers in all six network combinations; stateful positive fixtures remain pending. |
| KMIPKIT-0092-FR-006 | Exact release wheel digest plus installed-file verification. |
| KMIPKIT-0092-FR-008/009 | Fixed sanitized diagnostics; separate pending coverage; source-backed encoder/send limitations stay in the shared manifest. |

These controlled fixtures establish no vendor interoperability or certification.
Python-owned temporary copies cannot be zeroized reliably; use only the
checked-in deterministic fixture values when running this command.
