# Research: Local Cosmian KMS Integration

## Decisions

### Use the binary KMIP socket

**Decision**: Connect to Cosmian's raw TCP socket using KMIP binary TTLV and TLS 1.3/mTLS through the production `kmipkit_client::Client` API.

**Rationale**: Cosmian documents a native KMIP socket that carries binary TTLV, while its HTTP KMIP API is a JSON TTLV API. KMIPKit 1.0's existing production raw-TLS transport is the matching path. The KMS 5.28.0 configuration supports a listener on `5696`, server PEM certificate/key, and a client CA.

**Sources**:

- Cosmian KMS 5.28.0 server configuration reference: [server_configuration_file.md](https://github.com/Cosmian/kms/blob/5.28.0/documentation/docs/configuration/server_configuration_file.md), database defaults at lines 171–196, TLS and KMIP socket settings at lines 197–233.
- Cosmian's [KMIP socket and JSON TTLV documentation](https://docs.cosmian.com/key_management_system/kmip_support/json_ttlv_api.html).
- KMIPKit production transport contract and TLS policy: `specs/013-production-transport/spec.md`, `specs/013-production-transport/plan.md`, and OASIS KMIP Profiles v2.1 Operating System Profile §5.3.1 items 3–4 as the existing profile context.

### Pin the local server and isolate it

**Decision**: Use `ghcr.io/cosmian/kms:5.28.0@sha256:7b60fd4484930969906caa5722b727054ff96d49339ce81e3c64ca9e4278540e` and `alpine:3.22.2@sha256:4b7ce07002c69e8f3d704a9c5d6fd3053be500b7f1c69fc0d80990c2ad8dd412`; run them only for local tests, publish KMIP port `5696` to `127.0.0.1`, use SQLite in the disposable container filesystem with database clearing enabled, and do not mount persistent KMS data.

**Rationale**: This makes the requested KMS version reproducible and prevents this smoke test from writing to a shared or public service. Stopping and removing the container discards all database state. The operation itself is read-only.

### Generate temporary mutual-TLS credentials in Compose

**Decision**: A short-lived Alpine/OpenSSL Compose service creates a local CA and `localhost` server/client certificates in an ignored `.local/cosmian-kms/certs` directory, then exits successfully before KMS starts.

**Rationale**: Contributors do not need host OpenSSL, checked-in credentials, a secret manager, or an externally issued certificate. The root CA is supplied explicitly to KMIPKit and to Cosmian for client-certificate validation.

### Keep the first live test opt-in and narrow

**Decision**: Add one ignored test for the existing typed Discover Versions request. The user later authorized executing it while the KMIPKIT-0013 coverage blocker was still open.

**Rationale**: The branch already has production raw-TLS, mTLS, TTLV framing, typed request and typed response support for this one operation. OASIS KMIP Specification v2.1 §6.1.16, Tables 211–213, defines the operation and response; §9.16, Table 421, defines the Protocol Version structure. No broader operation support is required to test this vertical slice.

## Risks and limitations

- The GitHub container registry image must be reachable and the `5.28.0` image tag must be published for the contributor's platform.
- Additional KMIP operations are outside this smoke test and may reveal separate interoperability gaps.
- This smoke test exercises one request/response on one implementation. It is not a conformance suite, full KMIP 2.1 compatibility result, or certification.
- TLS certificate files are generated as local ignored artifacts. Removing `.local/cosmian-kms` deletes those files; Compose can regenerate them on the next start.

## Initial live-test failure and diagnosis

The user later authorized the live test before KMIPKIT-0013's coverage gate was cleared. The first run reached the KMS over verified mTLS; Cosmian logged one `DiscoverVersions` request for `kmipkit-integration-client`. KMIPKit then returned a sanitized transport error with `ResponseStarted` and cause `Other`. Temporarily logging only the eight-byte TTLV response-header metadata showed `tag=0x42007b`, `type=0x01`, and `value_len=168`; the diagnostic code was removed immediately afterward. OASIS KMIP v2.1 §8.4, Table 397, defines Response Message as the root structure, and §11.56 assigns its tag `0x42007B`. The transport had been expecting `0x420078`, the Request Message tag. This identifies a KMIPKit response-frame validation defect rather than a missing Cosmian setting.

## Deployment-only evidence

On 2026-10-08, `docker compose config --quiet` and the PowerShell parser check succeeded. The test-PKI Compose service generated the certificates and exited successfully; the Cosmian KMS 5.28.0 container started and its log reported `Socket server listening on 0.0.0.0:5696`. Docker showed only `127.0.0.1:5696->5696/tcp` published on the host. The resolved image digest was `ghcr.io/cosmian/kms@sha256:7b60fd4484930969906caa5722b727054ff96d49339ce81e3c64ca9e4278540e`. Container inspection showed mounts for the read-only KMS config and the server-certificate directory; the client certificate/key and CA private key are not mounted into the KMS container. `git check-ignore` confirmed the CA and client private-key paths are ignored.

After the response-tag fix, `cargo test -p kmipkit-client --test cosmian_kms -- --ignored` passed (1 passed, 0 failed). The server therefore requires no additional configuration for this Discover Versions exchange. `cargo test -p kmipkit-transport --test raw_tls` passed (90 passed, 0 failed). The production-client suite passed (14 passed, 0 failed) using `cargo test -p kmipkit-client --test production_client --features kmipkit-test-support/fixtures`. The Cosmian container remained running after the test.
