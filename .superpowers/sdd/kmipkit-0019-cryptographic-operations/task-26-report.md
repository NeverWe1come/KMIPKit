# Task 26 — Multipart Data Matrix Red Tests

Date: 2026-10-10

Branch: `feature/KMIPKIT-0019-encrypt-decrypt-implementation`

## Red test evidence

Added protocol request-builder matrix tests for both Encrypt and Decrypt via `to_ttlv_payload`, covering valid unframed, initial, middle, final, and framed single-request forms, plus malformed Data/correlation combinations. The valid rows pass. The invalid rows assert the payload-free local validation error kind, category, exact display text, and absence of request Data.

The initial test commit was `ed83e39 test(KMIPKIT-0019): add multipart Data matrix red tests`. QA identified that its negative case “middle part without Correlation Value” used `shape(true, None, None, false)`, which is byte-for-byte the valid unframed single-part shape. The stateless request model has no separate position marker, so it cannot reject that shape as a middle part while accepting it as unframed. The test was corrected to remove that impossible negative assertion. An absent-indicator, no-correlation request with Data is validated as unframed single-part; a caller represents continuation using the server-issued Correlation Value. No API, state, or production code was added.

Focused command run through WSL Ubuntu-26.04:

```powershell
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-protocol --lib multipart_validation_tests --offline'
```

Expected Red result after correction: 1 valid-matrix test passed; 1 invalid-matrix test failed because the current request builders accepted the 12 remaining invalid shapes (six cases for each operation). The removed indistinguishable shape is no longer asserted invalid.

Formatting and whitespace checks:

```powershell
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo fmt --all --check'
git diff --check
```

Both checks passed. This task adds Red tests only; no production validation is implemented here.
