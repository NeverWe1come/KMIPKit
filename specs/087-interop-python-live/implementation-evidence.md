# KMIPKIT-0087 implementation evidence

## Artifact under test

- Release ref: `release/1.0.0`
- Exact library commit: `52a2be180340e5da3166401a52f652dfd368ad37`
- Installed wheel SHA-256: `4bc56fc596d64cd8b7d63845e55831dad31d284c1fd260be5a1b0e0f7fa9cacf`
- Runtime: Python 3.12; Rust 1.94 build toolchain

The validator imports the installed wheel. It does not import `bindings/python`
from the checkout.

## Local verification

From `interop/python-kmip-validator` after the pinned artifact is installed:

```sh
PYTHONPATH=src python tools/verify_artifact.py
PYTHONPATH=src python -B -m unittest discover -s tests -p 'test_*.py' -v
PYTHONPATH=src python -m kmipkit_validator plan \
  --connection ../shared/profiles/connection.example.json
```

Observed locally:

- `verify_artifact.py` accepted the pinned commit and wheel hash listed above.
- All 22 consumer tests passed, including six controlled mTLS exchange and
  safety tests.
- The catalog contains 218 rows across KMIP 2.0 and 2.1. Offline plans
  enumerate all 203 cases applicable to KMIP 2.0 and all 212 applicable to
  KMIP 2.1 without network access.
- Each version plan marks 130 (KMIP 2.0) or 139 (KMIP 2.1) cases as planned
  and 73 as documented public-API gaps. Those gaps are visible as pending and
  are not represented as server incompatibilities.
- The installed-package constructor sweep built and schema-validated every
  advertised typed request recipe; async routes with required correlations
  use dedicated runners. Local enum and bitmask catalog values round-tripped
  through the installed TTLV API.

The GitHub Actions result for this feature branch is recorded after the
workflow completes. Controlled local TLS results are not vendor
interoperability evidence. No Cosmian, CipherTrust or ESKM endpoint was
available for a live run. Cross-language parity is not claimed by this
Python-only validator run.
