# Traceability Matrix: KMIPKIT-0067

| Requirement | Tasks | Verification |
| :--- | :--- | :--- |
| `REQ-067-001` (Unified Directory Layout) | `TASK-067-001`, `TASK-067-004` | Directory structure inspection in `docs/` |
| `REQ-067-002` (Two-Level Architecture & Strict Separation) | `TASK-067-002`, `TASK-067-005` | Inspection of `docs/site/` vs `docs/client-manuals/` |
| `REQ-067-003` (Bilingual Content Parity) | `TASK-067-005` | `docs/site/src/content/docs/{es,en}` parity check |
| `REQ-067-004` (Automated SSOT Generation) | `TASK-067-003` | Execution of `tools/docgen/generate_docs.py` |
| `REQ-067-005` (Astro Starlight Configuration) | `TASK-067-004` | `npm run build` in `docs/site` |
| `REQ-067-006` (CI Workflow & Link Integrity) | `TASK-067-006`, `TASK-067-007` | `.github/workflows/docs.yml` validation |
