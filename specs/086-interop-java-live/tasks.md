# Tasks

- [x] Install and verify the JNI JAR from an exact release commit in an
  isolated repository.
- [x] Replace regex parsing and the incomplete catalog model with the shared
  JSON contract.
- [x] Replace the mock driver with public Java typed routes and generic TTLV
  checks. Several catalog operations and routes remain pending coverage.
- [x] Classify every catalog operation against the installed Java request API;
  reject new catalog operations without an explicit route or API-limit reason.
- [x] Implement configuration, `plan`, `preflight`, `run`, and safety guards.
- [x] Create a controlled TLS peer with deterministic requests and responses.
- [x] Validate reports against the shared schema and redact sensitive data.
- [ ] Complete public API recipes for every applicable catalog case and gate
  CI on completed recipe coverage; current route and API-limit gaps remain
  visible in `plan` and per-case run diagnostics.
- [x] Update consumer guidance and document public API limitations.
