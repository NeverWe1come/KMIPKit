# C interoperability validator

Build and install KMIPKit from the source commit recorded by the artifact
installer. Configure CMake with the resulting installation directory.

```sh
python3 verification/interop/c-kmip-validator/tools/install_artifact.py
cmake -S verification/interop/c-kmip-validator -B verification/interop/c-kmip-validator/build \
  -DKMIPKIT_C_INSTALL_DIR=verification/interop/.local/c-live/install/992efad552ec7f5a8d269daf49b432b9c2f2339e
cmake --build verification/interop/c-kmip-validator/build
ctest --test-dir verification/interop/c-kmip-validator/build --output-on-failure
```

The native validator exposes `plan`, `preflight` and `run`. Use a dedicated
KMIP 2.0 or 2.1 server and a connection file based on the shared example.
Explicitly enable mutations in both the connection file and the run command
to exercise state-changing recipes. The controlled TLS tests cover both
versions with known request and response fixtures.

From the repository root, generate local evidence for the seven KMIP 3.0
generic-payload constructors.

```sh
python3 verification/interop/shared/tools/campaign.py --version 3.0 \
  --runner c verification/interop/c-kmip-validator/tools/draft_campaign.py \
  --output-dir /tmp/kmip-c-draft
```

The report records the installed library digest and the constructor checks.
The local checks leave XML scenarios and live KMIP 3.0 exchanges pending.
