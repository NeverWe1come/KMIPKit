# Traceability Matrix: Standalone Python KMIP 2.1 Validator

| Requirement | Description | Implementation Artifact | Verification |
|-------------|-------------|-------------------------|--------------|
| REQ-VAL-PY-001 | Standalone Python project isolated from library | `interop/python-kmip-validator/pyproject.toml` | Isolated import & package |
| REQ-VAL-PY-002 | Load capabilities and canonical catalog without external deps | `src/kmipkit_validator/cli.py` (`load_*`) | `test_parity_cosmian` |
| REQ-VAL-PY-003 | Capability matcher for operation, object, attribute filtering | `src/kmipkit_validator/matcher.py` | `test_capability_matcher` |
| REQ-VAL-PY-004 | Attempt context with unique naming for collision avoidance | `src/kmipkit_validator/isolation.py` | `test_attempt_context_uniqueness` |
| REQ-VAL-PY-005 | ANSI, HTML, Markdown, JSON matrix reporting | `src/kmipkit_validator/reporter.py` | `test_cli_run_cosmian` |
| REQ-VAL-PY-006 | CLI interface supporting `plan` and `run` commands | `src/kmipkit_validator/cli.py` | CLI integration test |
| REQ-VAL-PY-007 | Lightweight GitHub Actions workflow | `.github/workflows/interop-python.yml` | Action syntax & path filter |
