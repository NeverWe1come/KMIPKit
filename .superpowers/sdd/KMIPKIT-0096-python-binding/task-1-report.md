# KMIPKIT-0096 Python binding delivery

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
