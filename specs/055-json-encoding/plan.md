# KMIPKIT-0055 implementation plan

Implement only the private `kmipkit-client::json_encoding` codec, using the
approved common `Item`/`CodecLimits` contract and generated `WIRE_NAMES`.
No dispatcher, public API, manifest, or generated output changes.

1. RED: Add scalar, structure, mask, malformed, limits, and official-vector
   tests against an inert codec boundary. Record the actual failed run.
2. GREEN: Implement bounded document scanning before payload allocation,
   decode/encode representations, offline name normalization, exact unknown
   values, payload-free shared errors, and zeroizing wire ownership.
3. REFACTOR: Extract normalization and scalar helpers; inspect security and
   resource accounting; rerun focused/client tests, formatting, lint, coverage.
4. Record requirement traceability, official fixture provenance, results,
   remaining integration/profile gates, and commit evidence.

Sources: KMIP Specification v2.1 §10.2.2; KMIP Profiles v2.1
§§5.5.1.1–5.5.1.6.13, 5.5.2, 5.5.4.1; ADR-0018 and the approved
KMIPKIT-0054 common wire codec contract.
