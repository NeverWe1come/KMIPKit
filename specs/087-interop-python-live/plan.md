# Implementation plan: KMIPKIT-0087

1. Replace the simulated execution path with the shared configuration, catalog, provenance and schema-2.0 report contract.
2. Build and install the pinned KMIPKit wheel outside the consumer source tree; enforce Python 3.12 in CI.
3. Add typed request recipes and a controlled mTLS peer, then classify all remaining catalog rows as explicit API limits or not-run coverage.
4. Run offline and controlled integration checks, update evidence and open a draft PR. External KMS runs remain pending until an operator deploys one.
