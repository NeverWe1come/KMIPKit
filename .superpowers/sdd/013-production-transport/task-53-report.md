# Task 53 report — KMIPKIT-0013 requirement traceability

## Outcome

Added specification/compliance/requirements/KMIPKIT-0013.csv with 34 stable rows: all 18 feature requirements and 16 applicable OASIS catalog requirements. The OASIS set covers five TTLV requirements in §§10.1.2 and 10.1.5, three channel-security requirements in §10.4, and the eight selected HTTPS Client requirements in §5.3.1. FR-018 links the KMIPKIT-0012 ClientConfiguration prerequisite, ADR-0013, implementation ownership, and both T046a provenance tests.

Every implementation location and test function named in the CSV was checked against the current worktree. All project test references are derived tests; the CSV does not present them as official OASIS Test Cases. The pinned catalog has no requirement-specific Test Cases IDs for these records. The official HTTPS mandatory test case MSGENC-HTTPS-M-1-21 remains uncredited because its Query operation is not implemented. The mapping makes no formal profile, certification, or conformance claim.

## Applicability and open verification

- The eight §5.3.1 records use the catalog's profile_conditional scope and are marked scoped_verified only for the selected HTTP/1.1 + TTLV behavior. The implementation does not claim full HTTPS Client profile support.
- KMIPKIT-REQ-SPEC-10.1.2-001 remains deferred and is assigned to the Phase D typed operation-family specifications, with per-Structure assignments derived from the normative inventory. The catalog review note says generic ordered-child preservation does not establish the order of every KMIP Structure. Each applicable Structure needs an owner and structure-specific order tests before a global compliance claim.
- KMIPKIT-REQ-SPEC-10.4-001-002 remains deferred. Existing raw-TLS and HTTPS tests exercise valid protected exchanges, but the current tree has no test that corrupts a TLS record and asserts rejection. A successful TLS handshake is not direct negative integrity evidence. T063 was added to tasks.md as a corrective verification gate; the CSV does not mark this requirement verified.
- The catalog has no requirement records for the other TTLV sections cited by FR-007 (§§10.1.1, 10.1.3, 10.1.4, or 10.2). No IDs were invented; one-frame stream framing remains the KMIPKit project contract described by FR-007.

## Verification

- T053 CSV schema, exact FR/catalog ID set equality, implementation path, and Rust test-symbol audit: PASS — 34 unique rows, exact equality for 18 FR IDs and 16 normative catalog IDs, all referenced paths and symbols exist, and all eight §5.3.1 mappings are scoped_verified.
- python tools/normative_catalog/validate.py: PASS — catalog valid, 4 sources, 1,411 clauses, 4,024 records.
- python -m unittest tools.normative_catalog.tests.test_feature_traceability: PASS — 14 tests, 1 skipped. This existing suite validates other feature CSVs; the separate T053 audit above checked KMIPKIT-0013.
- git diff --check: PASS (exit 0; only a Windows working-copy line-ending notice).
- Runtime transport tests were not rerun for this documentation-only traceability task.

- T053 exact-set/path/symbol audit command (run in PowerShell):
  ```powershell
  @'
  import csv, json, re
  from pathlib import Path
  root = Path('.')
  rows = list(csv.DictReader((root / 'specification/compliance/requirements/KMIPKIT-0013.csv').open(encoding='utf-8-sig', newline='')))
  expected_fr = {f'KMIPKIT-0013-FR-{n:03}' for n in range(1, 19)}
  expected_oasis = {
      'KMIPKIT-REQ-SPEC-10.1.2-001', 'KMIPKIT-REQ-SPEC-10.1.2-002-001', 'KMIPKIT-REQ-SPEC-10.1.2-002-002',
      'KMIPKIT-REQ-SPEC-10.1.5-001-001', 'KMIPKIT-REQ-SPEC-10.1.5-001-002',
      'KMIPKIT-REQ-SPEC-10.4-001-001', 'KMIPKIT-REQ-SPEC-10.4-001-002', 'KMIPKIT-REQ-SPEC-10.4-001-003',
      'KMIPKIT-REQ-PROF-5.3.1-001', 'KMIPKIT-REQ-PROF-5.3.1-002', 'KMIPKIT-REQ-PROF-5.3.1-003', 'KMIPKIT-REQ-PROF-5.3.1-004',
      'KMIPKIT-REQ-PROF-5.3.1-005', 'KMIPKIT-REQ-PROF-5.3.1-008', 'KMIPKIT-REQ-PROF-5.3.1-009', 'KMIPKIT-REQ-PROF-5.3.1-010',
  }
  fr = {r['requirement_id'] for r in rows if r['requirement_kind'] == 'project functional'}
  oasis = {r['requirement_id'] for r in rows if r['requirement_kind'] == 'OASIS normative'}
  catalog = json.loads((root / 'specification/catalog/kmip-2.1.json').read_text(encoding='utf-8'))
  catalog_ids = {r['requirement_id'] for r in catalog['requirements']}
  assert len(rows) == 34 and fr == expected_fr and oasis == expected_oasis and oasis <= catalog_ids
  assert all((root / p).is_file() for r in rows for p in r['implementation_location'].split('; '))
  for r in rows:
      for ref in r['test_ids'].split('; '):
          path, symbol = ref.split('::', 1)
          source = (root / path).read_text(encoding='utf-8')
          assert re.search(r'(?m)^\s*(?:async\s+)?fn\s+' + re.escape(symbol) + r'\s*\(', source), ref
  profiles = [r for r in rows if r['requirement_id'].startswith('KMIPKIT-REQ-PROF-5.3.1-')]
  assert len(profiles) == 8 and all(r['status'] == 'scoped_verified' for r in profiles)
  print('PASS: 34 rows; exact 18 FR IDs and 16 catalog IDs; all implementation paths and 128 test symbols exist; 8/8 profile rows scoped')
  '@ | python -
  ```
- Result: PASS — 34 rows; exact 18 FR IDs and 16 catalog IDs; every implementation path and 128 test symbols exist; 8/8 selected profile mappings scoped.

## Review fix round 1

- Assigned deferred structure-ordering requirement KMIPKIT-REQ-SPEC-10.1.2-001 to Phase D typed operation-family specifications, with per-Structure allocation from the normative inventory and structure-specific tests required before a global claim.
- Replaced the count-and-membership audit with an explicit expected ID set and exact equality assertions for all 18 FRs and 16 OASIS records. The same command checks all 128 test symbols and implementation paths.
- Independent review approved the mapping and raised only these follow-up/evidence issues. Controller review after the fix confirmed the explicit Phase D ownership and exact-set audit result; this round changes documentation and traceability only.
## Changed files

- specification/compliance/requirements/KMIPKIT-0013.csv
- specs/013-production-transport/tasks.md
- .superpowers/sdd/013-production-transport/task-53-report.md
