# KMIPKIT-0105 Implementation Evidence

## Branch and stack

- Branch: `feature/KMIPKIT-0105-python-fluent-crypto-api`.
- Base: `feature/KMIPKIT-0104-java-fluent-crypto-api` at
  `cfbf6267828ab5b8407eb380ecf782553f086df6`.
- Parent PR: #188, Java fluent cryptographic API.
- PR: not opened yet.

## Red

- Command: `/tmp/kmipkit-py105-venv/bin/python -m pytest -q tests/test_fluent_crypto.py`
  from `bindings/python`.
- Result: **expected failure** before production changes; 2 failed. The first
  reported missing `Request.encrypt`; the second reported missing
  `EncryptBuilder`. This confirms the requested consumer surface is absent.
- Commit: `e134c998b`.

## Red — full operation and transport contracts

- Syntax command: `/tmp/kmipkit-py105-venv/bin/python -m py_compile tests/test_fluent_crypto.py`.
- Result: passed.
- Test command: `/tmp/kmipkit-py105-venv/bin/python -m pytest -q tests/test_fluent_crypto.py`.
- Result: **expected failure** before production changes; 12 failed because
  all seven operation-specific request factories/builders and their typed
  setters are absent. The parametrized TLS tests stop at the absent API check,
  before opening sockets.
- The tests cover operation-only setter surfaces, local missing-input and
  closed-handle rejection, typed Cryptographic Parameters and TTLV Data,
  caller-buffer copying, request-header/options separation, one TLS 1.3 mTLS
  exchange per operation, encoded request fields, and typed KMIP failure
  metadata.
- Commit: recorded after this Red commit is created.

## Green and Refactor

Pending implementation and verification.
