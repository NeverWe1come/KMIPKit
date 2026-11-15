# T039 Refactor Report — KMIPKIT-0013

## Scope

T039 refines Hyper response-parser error mapping for the HTTPS adapter and
confirms malformed-response sessions cannot remain reusable. The change is
limited to the approved production-transport specification and its T037 HTTPS
response regression coverage.

## Red, Green, Refactor

- **Red — `30c1369`** (`test(transport): classify Hyper parser failures as HTTP`):
  strengthened T037's malformed-parser assertions to require the safe `Http`
  cause category and added a held-peer regression for malformed status parsing,
  `ResponseStarted` delivery evidence, and absence of a reusable cached
  session. The focused test failed before the fix as expected: actual category
  `Io`, expected `Http`.
- **Green — `de8a415`** (`fix(transport): preserve Hyper response parser errors`):
  retained the Hyper source error through the internal request helper and
  classified parse/incomplete-message errors as HTTP, timeout errors as
  timeout, and remaining Hyper errors as I/O. Public `TransportError`
  continues to discard the source and expose only safe categories. Failed
  exchanges are not published to the reusable-session slot; dropping the
  connection invalidates its I/O and aborts the Hyper driver.
- **Refactor — `c5e4949`**
  (`refactor(transport): centralize HTTPS error classification`): extracted
  Hyper and I/O cause classification into focused helpers without changing
  behavior.

## Verification

| Command | Result |
| --- | --- |
| `cargo test -p kmipkit-transport --test https --offline malformed_http_parser_error_invalidates_the_reusable_session -- --exact --nocapture` | Passed: 1/1 after Green |
| `cargo test -p kmipkit-transport --test https --offline -- --test-threads=1` | Passed: 70/70 after Refactor; T037 rerun |
| `cargo test -p kmipkit-transport --all-targets --all-features --offline -- --test-threads=1` | Passed: all package targets, exit code 0 |
| `cargo clippy -p kmipkit-transport --all-targets --all-features --offline -- -D warnings` | Passed |
| `cargo fmt --all --check` | Passed |
| `git diff --check 60965b8..c5e4949` | Passed; independent QA verification |

The regression verifies that a malformed status line returns the sanitized
HTTP category with `ResponseStarted`, does not leave a reusable session, and
is observed while the peer remains open. No raw Hyper error text is exposed.

## Independent reviews

- Independent QA reported no code findings. It reran T037 (70/70), the full
  transport package (341 tests), formatting, and the diff check.
- Independent security review reported no vulnerability. It noted that
  sender-readiness failures still map to a safe generic I/O category; the
  dispatch parser path covered here preserves and classifies Hyper errors,
  while readiness failures remain fail-closed and the failed session is
  discarded.
- Formal Codex Security diff scan `00dacada-c642-4005-9340-9542a80be223`
  reviewed the two changed production files and the changed HTTPS regression
  test. Coverage is complete and there are no reportable findings.

## Limitations

No live KMIP server, production credentials, or consuming deployment endpoint
was used. Independent human security review remains a 1.0 release requirement.
