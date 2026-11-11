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

### Green

- Added CFFI declarations for all seven existing operation builders, typed
  setters, batch conversion, execution and release.
- Added `Request.encrypt()`, `decrypt()`, `sign()`, `signature_verify()`,
  `mac()`, `mac_verify()` and `hash()` with operation-specific builder classes.
- The builders copy input into a bounded mutable staging buffer, wipe the
  binding-owned copy after the synchronous C call, and preserve Rust-owned
  typed request, batch and response lifetimes.
- `asynchronous_indicator()` now writes through the existing C ABI builder
  setter, so the Request Header carries the selected value on the wire.
- Added typed Hash parameters to the Hash exchange fixture and valid single
  or multipart indicator shapes to the per-operation TLS fixtures.
- Command: `/tmp/kmipkit-py105-venv/bin/python -m pytest -q tests/test_fluent_crypto.py`
  from `bindings/python`.
- Result: **12 passed**.
- Commands: `/tmp/kmipkit-py105-venv/bin/python -m py_compile
  bindings/python/src/kmipkit/fluent.py bindings/python/src/kmipkit/_client_ffi.py
  bindings/python/tests/test_fluent_crypto.py` and `git diff --check`.
- Results: passed.

### Refactor

Pending import ordering, typing/documentation cleanup, full binding checks,
manifest generation and exact-head CI.

## Red — public API manifest

- Command: `python3 -m unittest
  tools.api_manifest.tests.test_0105_fluent_python_crypto_api_manifest`.
- Result: **expected failure**; the manifest has no KMIPKIT-0105 declarations
  for the seven Python factories or their builder types and methods.
- The contract pins the Python public surface, four-language mapping, approved
  source, and requirement linkage before adding its manifest declarations.
