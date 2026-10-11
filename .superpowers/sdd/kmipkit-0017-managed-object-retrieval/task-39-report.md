# T039 implementer report — English Get and Locate guide

Date: 2026-10-11<br>
Base: `96151d41e008b76a3797b4adbafbe57c688b942f`

## Result

Added `docs/user-guide/en/object-retrieval.md` and included it in the default
source set for `scripts/test_user_guide_examples.py`. The guide covers typed
Get and Locate construction, caller-supplied optional Get format/wrapping
values, typed batch execution, successful/error/Pending handling, typed
response accessors, Locate order and repetitions, opaque Get object handling,
security notes, normative citations, and links to the existing guides.

## Red, Green, and Refactor evidence

**Red — DCO-signed commit** `17c521a4f3c226eb2cbd2424ebc0311fc36fbb8e`.

Command:

```text
python -B scripts/test_user_guide_examples.py --guide docs/user-guide/en/object-retrieval.md
```

Result: exited 1 as expected. Rust compilation returned `E0599` for the
deliberately misspelled `LocateRequest::with_offset_item`; the compiler
identified `with_offset_items` as the available method. This showed the new
guide was included by the focused example compiler and that the example
contract detects invalid API usage.

**Green — DCO-signed commits** `b064b30c3d7e471d5a1cf790ea01483c40441284`
and `100ca2ea995ddea6828835687d463593e3b37f7a`.

The first corrected the example and updated the runner/test module
descriptions. The follow-up explicitly states that Get retains the Any Object
as opaque generic TTLV and documents caller-owned copy limits.

Exact verification commands and results on the final guide:

```text
python -B scripts/test_user_guide_examples.py --guide docs/user-guide/en/object-retrieval.md
```

Exit 0; compiled 3 marked examples using offline Cargo checks, including the
locked check.

```text
python -B scripts/test_user_guide_examples.py
```

Exit 0; compiled 11 marked examples from the default English and Spanish
guide set, including the new English guide.

```text
python -B -m unittest discover -s scripts/tests -p test_user_guide_examples.py -v
```

Exit 0; all 5 example-runner unit tests passed.

```text
$links = @('docs/user-guide/en/attribute-operations.md', 'docs/user-guide/en/production-transports.md', 'docs/user-guide/en/client-execution.md'); foreach ($link in $links) { if (Test-Path $link) { "OK $link" } else { throw "Missing local guide link target: $link" } }
```

All three local guide targets existed. `git diff --check` exited 0 after the
final edits.

**Refactor — separate DCO-signed evidence commit.** Post-Green review found no
source refactor that would improve these changes. The examples already separate
request construction from execution/outcome inspection, and the runner's
explicit source list is the established integration point. Extracting helpers
or adding more runner machinery would make the user-facing examples less direct
without adding coverage. No production source or runtime behavior was changed.
This report and the T039 checklist evidence record the no-source-change
disposition in DCO-signed commit
`4555c584c9dd7d0faa5d60d48893cf9d86f182f1`.

## Changed files

- `docs/user-guide/en/object-retrieval.md` — English guide with three marked,
  compile-checked Rust examples.
- `scripts/test_user_guide_examples.py` — add the new page to default inputs;
  clarify the module description.
- `scripts/tests/test_user_guide_examples.py` — clarify the test module
  description.
- `specs/017-managed-object-retrieval/tasks.md` — mark T039 complete and record
  concise phase evidence.
- `.superpowers/sdd/kmipkit-0017-managed-object-retrieval/task-39-report.md` —
  detailed evidence and review record.

No Spanish guide, pinned OASIS source, generated file, normative catalog,
production/API code, or dependency was changed.

## Self-review and limitations

- Request and response references identify OASIS KMIP Specification v2.1
  §§6.1.19 and 6.1.28 and their exact tables; Pending semantics cite §§9.1–9.2
  and §8.6 Table 399. The batch example cites §9.2 Table 401 for its explicit
  Asynchronous Indicator.
- The examples compile against current public Rust types and accessors. They
  do not contact a server or execute the client because production client
  construction requires application-owned identity and trust inputs.
- Locate is described as server-side matching; result order and duplicate IDs
  are preserved. Get returns opaque generic TTLV and is not locally decoded or
  processed cryptographically. Pending does not trigger polling, retry, or
  Recover/Get follow-ups.
- The example compiler checks Rust type correctness and offline/locked
  dependency resolution. It does not prove server interoperability or OASIS
  conformance. No full official case, profile support, certification, or
  external behavior is claimed.
- Existing runner unit tests and local link targets were checked. No separate
  Markdown renderer or Markdown linter was run; fence structure was covered by
  the guide-example extractor and claims/links were reviewed in the Markdown
  source.

## Independent-review follow-up

### Red evidence

The independent review identified two documentation defects: the guide
understated Asynchronous Indicator value 1, and its Get security note permitted
inspection of the Any Object structure. Before editing the guide, this
acceptance audit was run:

```powershell
@'
from pathlib import Path
text = Path('docs/user-guide/en/object-retrieval.md').read_text(encoding='utf-8')
issues = []
if 'Asynchronous Indicator of `1` is Mandatory' not in text:
    issues.append('missing Mandatory semantics for Asynchronous Indicator 1')
if '§11.3 Tables 431–432' not in text:
    issues.append('missing §11.3 Tables 431–432 citation')
if "generic Any Object's tag; do not inspect its structure" not in text:
    issues.append('missing tag-only opaque-object inspection guidance')
if 'generic Any Object\'s tag or\nstructure' in text:
    issues.append('permits inspection of secret-bearing object structure')
print('\n'.join(issues) if issues else 'all guide contract checks passed')
raise SystemExit(bool(issues))
'@ | python -B -
```

Result: exit 1 with four expected findings: missing Mandatory semantics,
missing §11.3 Tables 431–432 citation, missing tag-only inspection wording,
and the current text explicitly permitting structure inspection. No guide
change was present when this Red check ran.

The pinned OASIS KMIP v2.1 §11.3 Tables 431–432 confirm that value 1 is
Mandatory and that the server SHALL process all request batch items
asynchronously, returning an Asynchronous Correlation Value for each item.

### Green evidence

DCO-signed guide correction commit:
`35f15795ea22657428c9693bc21d27ab0047b25d`.

The guide now identifies value 1 as Mandatory, cites §9.2 Table 401 and §11.3
Tables 431–432, states the per-item server requirement, and cross-references
Pending semantics in §§9.1–9.2 and §8.6 Table 399. The Get note limits
inspection to Object Type, Unique Identifier, and the generic Any Object tag;
it explicitly prohibits inspection of the Any Object structure and
descendants.

The post-edit acceptance audit exited 0 with all four assertions passing:

```text
PASS: Mandatory value and request-field citation
PASS: required server behavior
PASS: tag-only opaque-object inspection
PASS: no permissive structure guidance
```

Exact verification commands and results after the guide edit:

```text
python -B scripts/test_user_guide_examples.py --guide docs/user-guide/en/object-retrieval.md
```

Exit 0; compiled 3 marked examples offline and locked.

```text
python -B scripts/test_user_guide_examples.py
```

Exit 0; compiled all 11 marked examples in the default English and Spanish
guide set.

```text
python -B -m unittest discover -s scripts/tests -p test_user_guide_examples.py -v
```

Exit 0; all 5 example-runner unit tests passed. `git diff --check` exited 0.

### Refactor disposition

Review after Green found no further restructuring that would improve the
guide. The changes remain documentation-only: the response example already
restricts access to the object tag, and the additional prose states the
normative server requirement without changing API examples or runner behavior.
No source refactor was justified; this disposition is recorded in the separate
DCO-signed Refactor evidence commit.
