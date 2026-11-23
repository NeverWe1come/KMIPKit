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

To run the typed `Query` campaign against a server, pass the same runner
with `--scope live --connection /path/to/connection.json`. The connection
selects KMIP 2.0 or 2.1 with `api_level` set to `typed-protocol`. The probe
requests operations and object types once, checks the response through the
installed package and records the wire result codes. The broader standalone
`run` command retains its own reports.
