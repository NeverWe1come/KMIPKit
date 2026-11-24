# C interoperability validator

Build and install KMIPKit from the source commit recorded by the artifact
installer. Configure CMake with the resulting installation directory.

```sh
validator=verification/interop/c-kmip-validator
python3 "$validator/tools/install_artifact.py"
install=$(PYTHONPATH="$validator/tools" python3 -c 'from artifact_support import INSTALL; print(INSTALL)')
cmake -S verification/interop/c-kmip-validator -B verification/interop/c-kmip-validator/build \
  -DKMIPKIT_C_INSTALL_DIR="$install"
cmake --build verification/interop/c-kmip-validator/build
ctest --test-dir verification/interop/c-kmip-validator/build --output-on-failure
```

The native validator exposes `plan`, `preflight` and `run`. Use a dedicated
KMIP 2.0, 2.1 or 3.0 test server and a connection file based on the shared example.
Explicitly enable mutations in both the connection file and the run command
to exercise state-changing recipes. The controlled TLS tests cover both
versions with known request and response fixtures.

From the repository root, generate local evidence for the seven KMIP 3.0
generic-payload constructors.

```sh
python3 verification/interop/shared/tools/campaign.py --version 3.0 \
  --runner c verification/interop/c-kmip-validator/tools/campaign_runner.py \
  --output-dir /tmp/kmip-c-draft
```

The report records the installed library digest and the constructor checks.
The local checks leave XML scenarios and live KMIP 3.0 exchanges pending.

The same local runner accepts `--version 2.0 --version 2.1` to check the
numeric values and allocation of concrete generic TTLV tags from the
shared matrix.

For a live typed KMIP 2.x Query Operations exchange, reuse the C runner
with `--scope live --connection PATH`. It selects the matching wire
obligation and invokes the native validator with `--case test-op-query`.
The native response assertions and numeric result codes feed the shared report.

Export `KMIPKIT_INTEROP_LIBRARY_COMMIT` before building the installed artifact
and invoking the campaign to select the common source revision. Configure
CMake with the installation directory for that commit.
