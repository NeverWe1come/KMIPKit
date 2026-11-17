## Added

- Comprehensive pre-release hardening campaign (KMIPKIT-0125) covering Miri strict-provenance checks, extended negative test suites (hostile XML, malformed JSON, truncated transport framing), FFI AddressSanitizer and UndefinedBehaviorSanitizer sweeps for C and JNI native bindings, and coverage-guided fuzzing targets for JSON, XML/wire preflight, and server-to-client multi-version dispatch.
- Added executable verification runner `scripts/run_hardening_campaign.py` and pre-release evidence ledger `docs/security/pre-release-hardening-evidence.md`.
