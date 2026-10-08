# T052 implementation and threat-model evidence

## Documentation changes

- Added an implementation-evidence section to ADR-0016 tying the accepted
  `ToSocketAddrs` policy to `resolver.rs`, the private worker, injected
  resolver/worker tests, and the raw-TLS/HTTPS late-result tests. It preserves
  the limits on what can be claimed about OS-owned DNS behavior and leaves the
  final platform/security gates open.
- Updated the threat model to cover the current typed production client,
  one-active/one-queued worker, shared 32-permit resolver governor, 16 retained
  addresses, OS-owned DNS activity and possible background resolver work,
  `SSL_CERT_FILE`, TLS resumption trust snapshots, HTTP parser limits,
  delivery/timeout races, raw-TLS versus HTTPS connection lifecycle, memory
  copies, lazy worker startup deadlines, and rejection of unrepresentable
  finite phase durations.
- Corrected the public API and transport architecture descriptions that still
  said a production typed client/backend did not exist. The docs distinguish
  this transport foundation from full KMIP operation coverage and note that
  independent platform, coverage, and human security gates remain open.

## Verification

- Verified 22 referenced source, specification, and documentation paths exist.
- `git diff --check`: passed.
- No transport behavior changed in this task. Full platform/coverage checks and
  independent security review remain assigned to T057/T060.
