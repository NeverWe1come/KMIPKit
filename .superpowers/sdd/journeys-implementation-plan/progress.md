# SDD ledger — KMIPKIT-0107 fluent journeys

- Parent PR: #193, exact head dc9819f610d9f34322f34e3b0b4fbfdbe8f5f776.
- Requirement: approved KMIPKIT-0093 FR-013 / F001.
- Branch: feature/KMIPKIT-0107-fluent-journeys.
- Pre-flight audit: five end-to-end journeys are incomplete across the four languages; existing tests are per-operation or legacy-facade tests. See the read-only FR-013 audit in the parent task.
- Tasks 1–6 are dependency ordered. Task 1 establishes the fixture contract and end-to-end lifecycle semantics consumed by tasks 2–5.
- Constraints: do not modify generated output manually; no production behavior change except if an approved fluent contract gap is found, in which case report it instead of expanding this task.

## Task 2: AES-GCM journey

- Red command: `sh tests/consumers/0093-journeys/verify.sh`.
- Red result: failed on request 5 (`index=4`): fixture expected Encrypt (`0x1f`) but received Get (`0x0a`); the current C lifecycle consumer exits with status 9. The four consumers currently omit the AES-GCM create/encrypt/decrypt/destroy sequence.
- The fixture now requires seven ordered requests per language: lifecycle Create/Get/Destroy followed by AES-GCM Create/Encrypt/Decrypt/Destroy. Green work is pending.
