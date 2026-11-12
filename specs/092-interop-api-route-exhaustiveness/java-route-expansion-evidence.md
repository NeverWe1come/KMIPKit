# Java local route expansion evidence

This implements local obligations of KMIPKIT-0092-FR-002, FR-003, FR-006,
and FR-008 against the installed artifact pinned at
`9a16cc90a12205101d22a38501a5e74c665df4dc`. The complete matrix remains
mandatory; unimplemented cases remain pending, never API-unavailable.

## Tag identity and allocation

Sources: OASIS KMIP 2.0 Specification §11.54 and KMIP 2.1 Specification
§11.56 (Tag enumeration), plus §10.1 (TTLV representation).

Red commit `3e8d2b469` adds per-case, per-version route assertions and fails
because the expected installed RawTag calls were absent. Integration with
the shared gate established that each selected case must complete every
applicable route. This phase therefore selects the 264 Tag cases whose
approved Java surface is entirely generic, including all 20 reserved tags.
The other 110 tags retain their generic and typed obligations as pending;
their absence does not change the manifest or full-matrix gate.

Each assigned tag is constructed with `RawTag.fromRaw`, allocation-checked,
used in `TtlvItem.create`, read through an item view, and decoded from a
deterministic TTLV fixture. The assertions compare the exact tag, type, and
sentinel value. Reserved tags retain their raw identity and exercise both
the actual checked-tag rejection and the decoder's `RESERVED_TAG` category.
These are tag-identity checks, not semantic payload or server-exchange claims.

Verification:

```sh
bash ../../bindings/java/gradlew --max-workers=2 test \
  --tests org.kmipkit.validator.ControlledTlsPeerTest.controlledQueryProducesPerRouteEvidenceForBothProtocolVersions
python3 ../../interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report build/route-execution-evidence.json \
  --artifact-sha256 37d7eeae9e64644fd7cdde4bfd64b7da5ccbb0bf2be53a949826eb62dc3aaf86 \
  --write-ledger
```

The focused Java test passes. The shared validator accepts 10,958 executed
cells, zero failed cells, and 60,497 pending out of 71,455 planned cells.
This adds 2,560 validated cells to the 8,398-cell baseline.

The local host's `/tmp` was full during development. Tests use
`JAVA_TOOL_OPTIONS=-Djava.io.tmpdir=<worktree>/interop/java-kmip-validator/build/java-tmp`
to retain private native extraction on the worktree filesystem. This changes
no package, test assertion, matrix obligation, or CI gate.
