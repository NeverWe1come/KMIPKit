# KMIPKIT-0087 implementation evidence

## Artifact under test

- Release ref: `release/1.0.0`
- Exact library commit: `52a2be180340e5da3166401a52f652dfd368ad37`
- Locally installed wheel SHA-256: `4bc56fc596d64cd8b7d63845e55831dad31d284c1fd260be5a1b0e0f7fa9cacf`
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
- All 25 consumer tests passed in 6.820 seconds, including six controlled
  mTLS exchange, version-gating and authentication tests.
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

## Final implementation-head CI

The published feature branch incorporated release commit
`2b01ab516e50035eee42fc206396c313dae8d9fb` with merge commit
`395a68ec7846fcdbabbdf0b382c65e9581bb1ae3`. The merge required no conflicts
and preserved the approved artifact pin and the locally installed wheel above.
No consumer production behavior changed in this closeout.

All five workflows for implementation head
`395a68ec7846fcdbabbdf0b382c65e9581bb1ae3` completed successfully:

- [CI, run 42909548152](https://github.com/NeverWe1come/KMIPKit/actions/runs/42909548152): Linux, Windows and macOS core jobs, language consumers, script contracts, sanitizers, normative inventory, documentation/traceability, dependencies, coverage and selected-scope coverage gate passed.
- [Interop Python Validator, pull-request run 42909547242](https://github.com/NeverWe1come/KMIPKit/actions/runs/42909547242): external wheel build/install and provenance, offline preflight and KMIP 2.0/2.1 plans, all 25 consumer tests and shared catalog contract checks passed. The unit suite took 8.330 seconds.
- [Interop Python Validator, push run 42909535759](https://github.com/NeverWe1come/KMIPKit/actions/runs/42909535759): passed independently for the same feature commit.
- [CodeQL Analysis, run 42909547772](https://github.com/NeverWe1come/KMIPKit/actions/runs/42909547772): C/C++, Java/Kotlin, Python and Rust analyses passed.
- [zizmor, run 42909548137](https://github.com/NeverWe1come/KMIPKit/actions/runs/42909548137): GitHub Actions workflow audit passed.

The PR rollup had 45 successful checks, including SonarCloud, and no failures
or pending jobs before this evidence update. Three jobs were skipped by their
workflow event conditions: `Dependency policy / active release schedule` and
`Informational branch coverage / nightly` run only for `schedule`;
`Profile evidence / protected release aggregate` runs only for a protected
`release/1.0.0` push. This was a feature pull-request run, so these skips do
not establish release aggregate evidence.

The pull-request CI wheel was built from the same exact library commit and
verified SHA-256
`0cdc157fc9017a1064f1fbbf4cc5e488e7eceb92afdfc4f97b147cdd8372d441`.
This CI artifact is distinct from the retained local wheel; byte-identical
wheel reproducibility is not claimed. Every artifact's report uses its own
verified digest.

The evidence-only follow-up commit is subject to a fresh CI cycle and must
pass before this PR is marked ready for review. The final follow-up SHA and
its check results are recorded in the closeout report and PR description to
avoid an endless sequence of self-referential evidence commits.

## Remaining limitations

Controlled local TLS results are not vendor interoperability evidence. No Cosmian, CipherTrust or ESKM endpoint was
available for a live run. Cross-language parity is not claimed by this
Python-only validator run.
