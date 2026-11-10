# C interoperability consumer

This standalone C11 consumer links the public KMIPKit C ABI installed from
release commit `52a2be180340e5da3166401a52f652dfd368ad37`. It targets KMIP 2.0
and 2.1 and uses the shared 218-case catalog. A successful build or controlled
fixture does not assert compatibility with a deployed vendor KMS.

Build the exact library artifact and the consumer from the repository root:

```sh
python3 interop/c-kmip-validator/tools/install_artifact.py
python3 interop/c-kmip-validator/tools/verify_artifact.py
cmake -S interop/c-kmip-validator -B interop/c-kmip-validator/build \
  -DKMIPKIT_C_INSTALL_DIR=interop/.local/c-live/install/52a2be180340e5da3166401a52f652dfd368ad37
cmake --build interop/c-kmip-validator/build
ctest --test-dir interop/c-kmip-validator/build --output-on-failure
```

After preparing a dedicated test KMS connection profile and mTLS files, run:

```sh
interop/c-kmip-validator/build/kmip_validator_c preflight \
  --connection path/to/connection.json
interop/c-kmip-validator/build/kmip_validator_c run \
  --connection path/to/connection.json \
  --catalog interop/shared/catalog/kmipkit-1.0-test-catalog.json \
  --capabilities interop/shared/profiles/generic-kmip-2.1.json \
  --output results-c.json --output-markdown results-c.md --output-html results-c.html
```

When the server requires KMIP Username/Password authentication, configure only
the environment variable names in `authentication.username_password`:

```json
"authentication": {
  "username_password": {
    "username_env": "KMIPKIT_USERNAME",
    "password_env": "KMIPKIT_PASSWORD"
  }
}
```

Set those variables in the process environment before `run`. The driver builds
the Credential through KMIPKit's zeroizing secret API and never writes either
credential value to its reports or console output. An unset variable stops the
run before opening a network connection.

Each mutating case is reported as `policy_restricted` unless both the connection
profile and the `--allow-mutations` command-line option enable it. Reports use
the shared schema and record the configured server identity, selected KMIP
version, catalog digest, exact library commit and installed library digest.
The capability profile is informational; it never suppresses a probe.

The pinned release artifact currently supports KMIP 2.1 live requests. KMIP 2.0
client execution remains blocked until the separate FFI protocol-version
preservation fix is integrated into `release/1.0.0` and this consumer is rebuilt
from that updated exact release commit. The public C API exposes typed request
builders plus generic TTLV construction and navigation. It does not currently
expose a high-level client API or a generic TTLV request dispatch path; the
consumer records those route limits as library/API gaps. Live runs require a
dedicated test KMS, valid mutual-TLS credentials and explicit opt-in for
mutations.
