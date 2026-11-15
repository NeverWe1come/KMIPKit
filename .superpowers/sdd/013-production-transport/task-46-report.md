# T046 Red — public production-client API

## Scope and traceability

T046 records the public Rust contract for KMIPKIT-0013 User Story 4. The
integration tests cover KMIPKIT-0013 FR-001, FR-011, and FR-016; the retained
client configuration boundary in FR-018 is exercised by the constructor call.
The typed Discover Versions exchange follows OASIS KMIP Specification v2.1
§6.1.16, Tables 211–212. Its mutually authenticated TLS profile context is
OASIS KMIP Profiles v2.1 Operating System Profile §5.3.1 items 3–4. Opaque
direct-adapter frames do not make a profile-conformance claim.

## Red evidence

Red development commit: `6e2ea68 test(client): add production transport API red contracts`.

`cargo test -p kmipkit-client --test production_client --offline` fails during
compilation at the intended missing surface: `Client::new` is not yet defined,
and `RequestOptions` is not yet re-exported by `kmipkit-client`. The remaining
test code and local TLS peer helpers type-check far enough to report no
unrelated compile errors. No production implementation was changed in this
stage.

The test-only contract includes:

- Validated raw-TLS and HTTPS client construction without opening a socket.
- Exact Discover Versions request capture and typed response through both
  transports, including a synchronous call from inside an existing current-thread
  Tokio runtime.
- Per-exchange variants for `execute`, Poll, Cancel, Process, and Query
  Asynchronous Requests.
- Direct raw-TLS and HTTPS adapter calls that prove a per-exchange total
  timeout overrides a zero-duration client default and that request and
  response bytes are preserved exactly.
- A compile-fail case attempting to construct the production client with a
  caller-implemented `Transport`.

All peers use loopback listeners and ephemeral test PKI. The client crate adds
only test-time `rustls` and Tokio dependencies; the production dependency
edges are unchanged.

## Checks

- `cargo test -p kmipkit-client --test production_client --offline` — expected
  Red failure for the absent constructor and shared options export.
- `cargo fmt --all --check` — passed.
- `git diff --check` — passed.

The compile-fail fixture's final `trybuild` diagnostic is deferred until the
production constructor exists, so it can assert the constructor rejects the
caller transport type rather than failing because the constructor itself is
absent.
