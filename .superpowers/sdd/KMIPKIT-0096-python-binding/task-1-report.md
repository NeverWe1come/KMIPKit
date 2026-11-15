# KMIPKIT-0096 Python binding delivery

This records the initial Python delivery. The subsequent FR-003 review
follow-up, authorized by the parent after rebase onto `5791bcadd`, is
documented in [task-2-report.md](task-2-report.md). That follow-up resolves
the cross-binding native metadata gap described below.

Base: `3c48b824e3bdc6ab0ff24310eed437a5ebf736d2` on
`feature/KMIPKIT-0096-python-expert-attributes`.

## Red

Command:

```sh
PYTHONPATH=bindings/python/src /tmp/kmipkit-0095-py312/bin/python -m pytest -q bindings/python/tests/test_fluent_expert_attributes.py tools/api_manifest/tests/test_0096_python_expert_attribute_positions.py -k 'not wire_order' --tb=short
```

Result: exit 1; 19 failed, 4 deselected. Eighteen failures report the absent
`SymmetricKeyBuilder.attribute_item_at`; one reports its absent manifest
declaration. Wire-order tests are present and deferred to Green to avoid
starting TLS peers for an absent method.

Additional FR-003 Red command after the initial adapter was staged locally:

```sh
PYTHONPATH=bindings/python/src /tmp/kmipkit-0095-py312/bin/python -m pytest -q bindings/python/tests/test_fluent_expert_attributes.py -k bad_final_positions --tb=short
```

Result: exit 1; 3 failed, 24 deselected. The existing native build ABI records
generic validation errors with delivery `NONE`; the tests require `NOT_SENT`.
The parent confirmed the spec requires narrowly normalizing this metadata in
the Python fluent build path. The global status helper and native ABI remain
unchanged. The final Red commit includes these explicit assertions, fixture
corrections (Description is `0x4200FC`, and closed Items use InvalidInputError),
and additional catalogued/Vendor Attribute preservation cases.

The same metadata requirement was tested for insertion-time catalog/type and
Vendor Attribute rejection with `-k 'catalogued_item_keeps or table_150'`:
exit 1; 3 failed, 1 passed, 23 deselected, again because delivery was `NONE`.

Python 3.12.13 is provided by `/tmp/kmipkit-0095-py312`. The local ignored
`_native_ffi` directory is linked to the previous 0095 stack build at the same
base revision; tests import Python source from this worktree via `PYTHONPATH`.
The native insertion ABI already exists and no native implementation changes
are required.

The requested `specs/096-fluent-expert-attributes/checklists/requirements.md`
does not exist in this baseline; the approved spec, plan, tasks, traceability,
task brief, and `/tmp/KMIPKIT-0096-python-binding-plan.md` govern this delivery.

## Green

Added `SymmetricKeyBuilder.attribute_item_at(index, item)` and the private
CFFI declaration for the existing C ABI. Native insertion/build validation
exceptions retain their existing sanitized category/cause and now report
`NOT_SENT` in this Python fluent path. Python argument errors retain the
existing payload-free `InvalidInputError`. No Rust behavior or native symbol
was changed.

Commands/results:

- Focused Python suite plus manifest contract:
  `PYTHONPATH=bindings/python/src /tmp/kmipkit-0095-py312/bin/python -m pytest -q bindings/python/tests/test_fluent_expert_attributes.py bindings/python/tests/test_fluent.py tools/api_manifest/tests/test_0096_python_expert_attribute_positions.py --tb=short`:
  exit 0; 46 passed.
- Full Python suite before final metadata assertions:
  `PYTHONPATH=bindings/python/src /tmp/kmipkit-0095-py312/bin/python -m pytest -q bindings/python/tests --tb=short`:
  exit 0; 1,327 passed, 119 subtests passed.
- `python3 -B tools/api_manifest/generate.py --write`: exit 0; 12 outputs.
  The generator initially rejected declared-to-blocked targets. The two
  existing Rust `attribute_at` declarations and Java `attributeItemAt` were
  promoted to `declared` with four-language mappings, enabling the new Python
  declaration. No declarations were promoted to `verified`.
- The repository Python source audit (`inventory_python_sources` followed by
  `compare`, using `tools/api_manifest/audit_binding_exports.py`) reports
  complete/matching: 3,606 observed, 2,597 manifest and exact matched, zero
  missing/not-observed/kind mismatches or issues. Its JSON report is
  `/tmp/KMIPKIT-0096-python-export-audit.json`.
- `cargo fmt --all --check` and `git diff --check`: exit 0.
- `python3 -B tools/normative_catalog/validate.py --structural-only`:
  exit 0; 4 sources, 1,411 clauses, 4,143 records.
- `/tmp/kmipkit-0095-py312/bin/python -m pytest -q tools/normative_catalog/tests/test_feature_traceability.py`:
  exit 0; 31 passed, 1,413 subtests passed.

## Cross-binding observation

`kmipkit_symmetric_key_builder_build_batch_item` records generic invalid-input
metadata through `record_ffi_error`, which sets delivery `NONE`. Direct C
consumers of the last-error metadata therefore still see `NONE` for these
pre-exchange builder failures. Java also delegates build to that ABI; its
typed builder surface is unchanged in this Python slice. The Python build
and insertion paths explicitly correct validation delivery to `NOT_SENT` and
are tested against a real listener accepting no connection. The shared
native metadata gap is recorded for the parent to track; it is not a Python
delivery limitation.

Hosted exact-head CI and stack integration remain the parent's responsibility;
T005/T006 are intentionally left open. No push, PR, or rebase was performed.

## Refactor and final local verification

Red: `e90957edc`. Green: `868e5a179`.

Consolidated the two identical insertion/build validation-error conversions
into a private fluent-only helper. Non-validation errors continue unchanged;
the global client status function is unchanged.

Final commands/results:

```sh
PYTHONPATH=bindings/python/src COVERAGE_FILE=/tmp/KMIPKIT-0096-python.coverage /tmp/kmipkit-0095-py312/bin/python -m coverage run --rcfile=bindings/python/pyproject.toml -m pytest -q bindings/python/tests --tb=short
COVERAGE_FILE=/tmp/KMIPKIT-0096-python.coverage /tmp/kmipkit-0095-py312/bin/python -m coverage report --rcfile=bindings/python/pyproject.toml
COVERAGE_FILE=/tmp/KMIPKIT-0096-python.coverage /tmp/kmipkit-0095-py312/bin/python -m coverage json --rcfile=bindings/python/pyproject.toml -o /tmp/KMIPKIT-0096-python-coverage.json
python3 -B tools/api_manifest/generate.py --check
/tmp/kmipkit-0095-py312/bin/python -m pytest -q tools/api_manifest/tests/test_0096_c_expert_attribute_positions.py tools/api_manifest/tests/test_0096_python_expert_attribute_positions.py --tb=short
/tmp/kmipkit-0095-py312/bin/python -m pytest -q tools/api_manifest/tests/test_t102_0052_route_total.py --tb=short
cargo fmt --all --check
git diff --check
```

All exited 0. Full Python: 1,327 passed, 119 subtests passed, 92.32 seconds.
Coverage: 96.62% aggregate (8,874/9,184 statements), 91.26% for `fluent.py`.
Intersecting added lines from `git diff 3c48b824e --unified=0` with executable
lines in the coverage JSON gives 15/16 changed executable lines covered
(93.75%, above the 90% changed-code gate). The uncovered line is re-raising
a non-validation native error; validation, input rejection, ownership, and
successful exchange paths are covered. Generated metadata has no changed
executable behavior.

Generator check verified all 12 outputs. 0096 C/Python manifest tests:
2 passed. T102 total snapshot: 2 passed, 5 subtests passed. The final Python
source export audit was rerun after extracting the private helper and still
reports complete, matching, 2,597 exact manifest matches and zero issues.

Rust workspace Clippy/tests/llvm-cov and complete C/Java suites were not rerun
in this Python-only slice; the parent owns stack integration and exact-head
CI. Rust's generated declaration metadata is the only changed Rust file.

The temporary local native-library symlink used for tests is removed after
verification; the parent-provided untracked task brief is preserved.
