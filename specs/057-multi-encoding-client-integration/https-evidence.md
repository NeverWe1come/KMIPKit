# KMIPKIT-0057 HTTPS integration evidence

## Scope and requirement traceability

This slice implements Task 3 only. The transport accepts already encoded
bytes; typed message validation and same-format body decoding remain in the
single client execution path. Empty-container transport fixtures distinguish
wire formats without claiming valid typed requests or official profile cases.

| Stable requirement | Exact source | Implementation | Verification |
| --- | --- | --- | --- |
| KMIPKIT-0057-HTTPS-001 | OASIS KMIP Profiles v2.1 §5.3.1, list items 2, 5–9; ADR-0018 decision 4 | Selected-format request Content-Type, unchanged body, POST, Content-Length, no-cache | `selected_encoding_posts_exact_body_and_headers_and_accepts_the_same_format_at_the_byte_cap` |
| KMIPKIT-0057-HTTPS-002 | Profiles v2.1 §5.3.2, list items 2, 4–7; ADR-0018 decisions 1, 4–5 | Require one response Content-Type matching this exchange; retain status and length validation | `selected_encoding_rejects_mismatched_duplicate_missing_and_invalid_response_types_without_allocation`; `selected_encoding_changes_on_a_reused_https_connection` |
| KMIPKIT-0057-HTTPS-003 | ADR-0018 decision 6; approved 0057 acceptance criteria | Actual request/response byte caps, allocation guard, partial response zeroization | `selected_encoding_preserves_request_limits_before_connection`; `selected_encoding_preserves_response_byte_limits_and_partial_response_zeroization`; exact-cap success test |
| KMIPKIT-0057-HTTPS-004 | ADR-0018 decision 5; constitution IV; approved 0057 acceptance criteria | Redacted HTTP mismatch error, ResponseStarted evidence, invalidated session, no retry/fallback | `selected_encoding_mismatch_never_retries_or_falls_back`; response type rejection test |

The contextual body rule is KMIPKit's interpretation of the internal source
inconsistency recorded by ADR-0018. This evidence does not claim an OASIS
resolution, full HTTPS/JSON/XML profile conformance, or certification.

## RED

Command (Windows x86_64 MSVC):

```text
cargo test -p kmipkit-transport --test https encoding_tests -- --nocapture
```

Result: exit 1; 1 passed, 5 failed. Expected failures show JSON/XML outgoing
Content-Type remains octet-stream, JSON/XML responses are rejected, text
exchanges accept a TTLV response, and media-type rejection masks the selected
format's response-byte limit. The request-limit compatibility case passes.
No production code was changed before this run.

Cargo reports existing AWS-LC build diagnostics about the optional stdalign
compiler probe and missing NASM; the Rust target builds and runs the tests.

## GREEN

The HTTPS exchange resolves `RequestOptions::wire_encoding()` once, with TTLV
for `None`, and carries the matching static media type through its worker.
Request construction and response validation consume that same value.
Response type rejection happens before the KMIPKit body allocation. The
existing framing, byte limits, zeroizing owners, delivery tracking, and
connection cleanup continue to apply to every format.

```text
cargo fmt --all
cargo test -p kmipkit-transport --test https encoding_tests -- --nocapture
cargo test -p kmipkit-transport --all-features
```

Results: all commands exit 0. The six focused encoding tests pass. The full
transport crate suite passes 624 unit/integration tests and 5 doc tests,
including 115 tests in the HTTPS target. The existing AWS-LC build diagnostics
described above remain present.

## REFACTOR

Name the settings `HttpsExchangeSettings` to make their per-exchange lifetime
explicit even on a reused connection, isolate the exact OASIS media-type
mapping, and document direct-call TTLV compatibility. Extend the reuse test
with an option-free TTLV call after XML. Add a public-adapter exchange test
alongside the source-included tests so both public dispatch and private
zeroization observers exercise JSON and XML. Retain the existing request
construction error branch without reformatting its unchanged lines.

```text
cargo fmt --all --check
git diff --check
cargo clippy -p kmipkit-transport --all-targets --all-features -- -D warnings
cargo test -p kmipkit-transport --test https encoding_tests -- --quiet
cargo llvm-cov clean --workspace
cargo llvm-cov -p kmipkit-transport --all-features --json --output-path target/0057-https-coverage.json -- --quiet
```

Formatting, whitespace validation, Clippy, and all seven focused tests pass.
The first Clippy run identified five new test assertion-style lints; their
assertions were clarified without suppressing a lint or printing wire bodies.
Coverage artifacts were cleaned because old instrumented client binaries
otherwise contributed obsolete HTTPS source locations. Normalize the fresh
export with the repository coverage tools and project its records onto the
executable transport source inventory for this slice. Test-support source
and external macro records do not belong to the transport coverage scope.

Final instrumented result: exit 0; 625 unit/integration tests pass, including
116 HTTPS target tests. Scoped coverage parsed by the unchanged repository
coverage tools is 29/29 changed executable lines (100.00%), HTTPS source
923/1011 lines (91.30%), and transport 3741/3932 lines (95.14%). No changed
executable line is uncovered. These measurements exceed the 95% changed-code
and 85% transport thresholds; they do not substitute for the full release
coverage matrix.

The following Python input was run with `python -` from the worktree root to
calculate the scoped measurement against the Task 2 plus plan baseline:

```python
from functools import cache
from pathlib import Path
import json
import subprocess
from scripts import coverage_gate as gate

root = Path.cwd()
sources = {
    p.as_posix()
    for p in Path("crates/kmipkit-transport/src").glob("*.rs")
    if gate.rust_source_has_function_body(p.read_text(encoding="utf-8"))
}
absolute_sources = {(root / p).resolve() for p in sources}

@cache
def in_scope(filename):
    return Path(filename).resolve() in absolute_sources

data = json.loads(Path("target/0057-https-coverage.json").read_text())
for item in data["data"]:
    item["files"] = [f for f in item["files"] if in_scope(f["filename"])]
    item["functions"] = [
        f for f in item["functions"] if any(in_scope(n) for n in f["filenames"])
    ]
report = gate.parse_llvm_export(
    gate.normalize_llvm_export(data, root), root, source_filter=sources
)
diff = subprocess.check_output(
    ["git", "diff", "--unified=0", "15461c8bd", "--",
     "crates/kmipkit-transport/src/https.rs"], text=True
)
changed = {
    (p, n) for p, lines in gate.parse_added_rust_lines(diff).items() for n in lines
}
print(gate.changed_code_result(changed, report))
covered = sum(n > 0 for lines in report.values() for n in lines.values())
total = sum(len(lines) for lines in report.values())
total += sum(report.summary_uncovered_excess.values())
print(f"Transport: {covered}/{total} ({covered * 100 / total:.2f}%)")
```

Workspace-wide, foreign-language, generated-output, platform-matrix, and
interoperability evidence remains assigned to Tasks 5–6.
