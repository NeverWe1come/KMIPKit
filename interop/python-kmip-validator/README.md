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
