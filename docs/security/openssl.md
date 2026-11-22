# Native OpenSSL build and maintenance

KMIPKit's TLS implementation uses Rustls with the OpenSSL cryptographic
provider. The selected native dependency is OpenSSL 3.5 LTS. The exact source
version and archive SHA-256 are recorded once in
[`verification/security/openssl/native.json`](../../verification/security/openssl/native.json).
CI and the local helper read the same pin, verify the downloaded archive, and
check the built library version before using it.

## Build for a consumer

If the build host already has OpenSSL 3.5 and its development files, point
Cargo at that installation when auto-detection does not find it:

```sh
export OPENSSL_DIR=/path/to/openssl-3.5
cargo build --release
```

To compile the exact upstream archive used by CI, run the helper from the
repository root:

```sh
python3 verification/security/openssl/build.py
# Copy the OPENSSL_DIR and OPENSSL_STATIC commands printed by the helper.
cargo build --release
```

On Windows, run the helper in PowerShell:

```powershell
python verification/security/openssl/build.py
# Copy the OPENSSL_DIR and OPENSSL_STATIC commands printed by the helper.
cargo build --release
```

The helper prints the environment settings after a successful build. The
generated static build is also suitable for compiling the C, Java, and Python
native packages.

The source-build helper needs Python 3.12 or later, Perl, and a platform C
compiler and build tools. On Linux and macOS this means `make`; Windows uses
MSVC x64 build tools and `nmake`. NASM is optional; without it, the helper
configures OpenSSL's portable assembly fallback. Consumers can instead use a
system OpenSSL 3.5 installation and skip the source build.

## Update and security review

The weekly **Native OpenSSL update** workflow checks the GitHub release tags
for the selected 3.5 LTS series. A newer stable patch fails the check and opens
or updates one tracking issue; the tracker never changes the pin itself. Once
the version is updated, it closes its issue.

For each update, review the upstream release notes and the checksum published
by the [OpenSSL source page](https://openssl-library.org/source/). Then update
`version`, `archive_sha256`, and `release_url` together in `native.json`,
rebuild the source in CI, and review the OpenSSL build result on supported
operating systems. Keep the selected LTS series unless a separate review
approves changing it. OpenSSL lists support for the 3.5 series through 8 April
2030 on its source page.

The source archive SHA-256 establishes that the fetched file matches the
reviewed pin; it does not establish that the release has no vulnerabilities.
The update issue is a maintenance signal, not an automatic security approval.
Review each release's advisories and rebuild statically linked native packages
after adopting a security fix.
