# SDD ledger — plan: specs/043-public-api-bindings/plan.md

Task 099: in progress — baseline and approved sources reviewed; Red/Green/Refactor implementation dispatched.
Ruling: reuse T040 four-language operation facades for semantic parity; make exact, owner-backed exclusions for Rust-only artifacts and helpers; no new public APIs.


Task 099: complete — 165 exact baseline candidates resolved as 43 four-language mappings and 122 owner-approved exclusions; the 229-identity SHA-256 is preserved. Red `d734f32134eabb0d7d96c550d0c0f25e9ad3f0ea`; Green `047bf9199313bd0f074351573a25884261474f1c`; Refactor `3e5ec939526eecf213b0826b3624d2d245549193`. Separate T040 canonical Rust operation-pair correction: `1ab4f8b8a8d6fc5b5e2f31b551114d0663a43edb`. Final focused suites passed 9/9; generator verified all 12 outputs. Full report: `.superpowers/sdd/KMIPKIT-0043-T099/report.md`.

T099 review correction: Red `d5a30df99` added the complete-result-container mapping assertion and failed on the 14 expected outcome rows. Green updates exactly those 14 C/Java/Python targets; typed-success selectors and fields remain unchanged. Owner counts/fingerprint are preserved; focused T099/T040 suites pass 10/10 and the pinned generator verifies all 12 outputs. Green manifest/evidence changes are committed separately from Red.