# Implementation plan

1. Record approved scope, exact normative references and failing CLI contracts.
2. Build/install Rust packages from a clean archive of the pinned commit;
   independently link the consumer and embed provenance.
3. Implement validated shared config, offline planning/preflight, safe case
   dispatch, redacted schema 2.0 reports and controlled local exchange tests.
4. Run isolated Rust 1.94 formatting, Clippy and tests. Review every catalog row
   and publish a draft PR only after the pushed CI succeeds.
