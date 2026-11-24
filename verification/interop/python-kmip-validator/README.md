# Installed Python interoperability validator

Use Python 3.12 in an isolated environment and install the native build
dependencies and the verified KMIPKit wheel.

```sh
python3.12 -m venv .venv
. .venv/bin/activate
python -m pip install 'maturin==1.9.4' 'cffi==2.0.0' 'jsonschema==4.23.0'
python verification/interop/python-kmip-validator/tools/install_artifact.py
python verification/interop/python-kmip-validator/tools/verify_artifact.py
```

The native build uses the OpenSSL version described in
[OpenSSL build and maintenance](../../../docs/security/openssl.md).
The installer builds the wheel from the exact reviewed commit in
`library-pin.json` and records its digest. The validator verifies the
installed bytes against that wheel before recording campaign results.

## KMIP 2.0 and 2.1 against a configured server

From the validator directory, set `PYTHONPATH=src` and run the independent CLI.

```sh
PYTHONPATH=src python -m kmipkit_validator plan --connection /path/to/connection.json
PYTHONPATH=src python -m kmipkit_validator preflight --connection /path/to/connection.json
PYTHONPATH=src python -m kmipkit_validator run --connection /path/to/connection.json --output-dir results
PYTHONPATH=src python -m unittest discover -s tests -p 'test_*.py' -v
```

Use a dedicated test KMS with client TLS credentials and the shared
connection schema. State-changing recipes require both the connection
policy and `--enable-mutations`. The test suite uses controlled peers with
TTLV, JSON and XML over direct TLS and HTTPS. Version-boundary tests assert
local rejection and an empty peer transcript.

## Local KMIP 3.0 draft campaign

From the repository root, launch the validator in the activated environment.

```sh
python verification/interop/shared/tools/campaign.py --version 3.0 \
  --runner python verification/interop/python-kmip-validator/tools/run_campaign.sh \
  --output-dir /tmp/kmip-python-draft
```

The runner checks the operation code from each of the seven catalogued
generic-payload constructors and records that local evidence. XML replay and
live KMIP 3.0 exchanges remain pending in the campaign report.

The same local runner accepts `--version 2.0 --version 2.1` to check the
numeric values and allocation of concrete generic TTLV tags from the
shared matrix.

To run the typed `Query` campaign against a server, pass the same runner
with `--scope live --connection /path/to/connection.json`. The connection
selects KMIP 2.0 or 2.1 with `api_level` set to `typed-protocol`. The probe
requests operations and object types once, checks the response through the
installed package and records the wire result codes. The broader standalone
`run` command retains its own reports.

Use `--scope controlled --version VERSION` to execute the pinned OASIS
Query Operations scenario for KMIP 2.0, 2.1 or 3.0 against a loopback TLS
peer. The peer verifies the version, batch count, correlation value and
query functions, then returns the ordered operation list from the XML.
KMIP 2.0 and 2.1 require Protection Storage Masks in the structured
response, but their published fixtures omit that field. The validator
preserves the fixtures and records the decoder rejection as a failed
scenario. The 3.0 draft makes the field conditional on the requested
Query Function, and the wheel pin includes the corresponding decoder fix.

Export `KMIPKIT_INTEROP_LIBRARY_COMMIT` before building the wheel and invoking
the campaign to select a specific common source revision. In a source revision
with the Query decoder correction, the 3.0 scenario checks the decoded
operation list for equality, including order.

The `generic-ttlv` route sends Locate, Query, Discover Versions, RNG Retrieve,
and single-part SHA-256 Hash on KMIP 2.0 and 2.1, plus Ping on 2.1. It uses
the installed KMIPKit tag and response models and Python's certificate-authenticated
raw TLS transport. RNG Retrieve follows the catalog's mutation opt-in.

With mutation opt-in, the direct tag route also runs Create, Get, Get Attributes, Destroy,
Archive and Recover. Each case creates an isolated AES-256 key and uses the
server-returned Text String identifier for all follow-up exchanges. The
validator checks the response identifier and performs cleanup with Destroy.
