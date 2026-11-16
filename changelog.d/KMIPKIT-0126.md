### Fixed

- Remediated second independent audit findings (KMIPKIT-0126):
  - Fixed corrupted KMIP 2.1 tags in `kmip-versioned-tags.json` and realigned `v21_tags` in attribute policy generator.
  - Added `Split Key Polynomial` enumeration and enforced fail-closed validation for unmapped enumeration tags.
  - Added verified OASIS KMIP 3.0 enumeration deltas to versioned values catalog.
  - Expanded typed attribute generation across Rust, C, Java, and Python from 62 baseline to all 99 normative attributes.
  - Corrected hex tags (`Digest`, `Key Value Present`, `Key Format Type`) and operation names in KMIP 2.0 VersionGate.
  - Added per-operation typed response readers for the 7 KMIP 3.0 operations across Rust, C, Java, and Python client batch APIs.
  - Unblocked live KMIP 2.0 routes in Java interop validator.
