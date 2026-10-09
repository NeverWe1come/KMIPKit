# OASIS source inventory

Downloaded: 2026-10-03
Publisher: OASIS Open

| Local file | Work product | Stage | Approval/publication | Canonical URL |
|---|---|---|---|---|
| `upstream/kmip-spec-v2.1-os.html` | Key Management Interoperability Protocol Specification Version 2.1 | OASIS Standard | 14 December 2020 | <https://docs.oasis-open.org/kmip/kmip-spec/v2.1/os/kmip-spec-v2.1-os.html> |
| `upstream/kmip-profiles-v2.1-os.html` | Key Management Interoperability Protocol Profiles Version 2.1 | OASIS Standard | 14 December 2020 | <https://docs.oasis-open.org/kmip/kmip-profiles/v2.1/os/kmip-profiles-v2.1-os.html> |
| `upstream/kmip-ug-v2.1-cn01.html` | Key Management Interoperability Protocol Usage Guide Version 2.1 | Committee Note 01 | 16 November 2020 | <https://docs.oasis-open.org/kmip/kmip-ug/v2.1/cn01/kmip-ug-v2.1-cn01.html> |
| `upstream/kmip-testcases-v2.1-cn01.html` | Key Management Interoperability Protocol Test Cases Version 2.1 | Committee Note 01 | 16 November 2020 | <https://docs.oasis-open.org/kmip/kmip-testcases/v2.1/cn01/kmip-testcases-v2.1-cn01.html> |

OASIS notices permit copying the documents and preparing material that comments
on, explains, or assists implementation when the copyright notice and notices
section are retained. Always consult the notice inside the exact work product.

The source inventory records what KMIPKit uses; it does not relicense OASIS
content or make a trademark, conformance, or endorsement claim.

## Supplemental official test-case fixtures

| Local file | Work product | Stage/date | Canonical URL | SHA-256 |
|---|---|---|---|---|
| `fixtures/TC-CREATE-SD-1-21.xml` | Key Management Interoperability Protocol Test Cases Version 2.1 | Committee Note 01, 07 May 2020 | <https://docs.oasis-open.org/kmip/kmip-testcases/v2.1/cn01/test-cases/kmip-v2.1/TC-CREATE-SD-1-21.xml> | `882e0f57ff2cc42b214e2ffb40bf9c80105489aab00f07a6ea5e2c1c395474ad` |
| `fixtures/TC-STREAM-ENC-1-21.xml` | Key Management Interoperability Protocol Test Cases Version 2.1 | Committee Note 01, 07 May 2020 | <https://docs.oasis-open.org/kmip/kmip-testcases/v2.1/cn01/test-cases/kmip-v2.1/TC-STREAM-ENC-1-21.xml> | `765b8b33b03ff2ae137ff273085e0196857bfdde38bcb078c8003dcfb06e07dc` |
| `fixtures/TC-STREAM-ENC-2-21.xml` | Key Management Interoperability Protocol Test Cases Version 2.1 | Committee Note 01, 07 May 2020 | <https://docs.oasis-open.org/kmip/kmip-testcases/v2.1/cn01/test-cases/kmip-v2.1/TC-STREAM-ENC-2-21.xml> | `8e106a1304899cbc975b283aec41101774aac127988962c34b5daa6a0facc41f` |
| `fixtures/TC-STREAM-ENCDEC-1-21.xml` | Key Management Interoperability Protocol Test Cases Version 2.1 | Committee Note 01, 07 May 2020 | <https://docs.oasis-open.org/kmip/kmip-testcases/v2.1/cn01/test-cases/kmip-v2.1/TC-STREAM-ENCDEC-1-21.xml> | `24ee1de35f5815c53996eacc1baafc780e9d7dea8852db546b03be6660d307f8` |

These fixtures are byte-identical linked XML work products. Feature tests may consume the in-scope operation items from a multi-operation fixture; partial item tests must not be reported as complete official-case passes. Fixture presence alone does not mean that a test passed.

The three streaming-operation fixtures were pinned on 2026-10-09 from the
canonical OASIS links above after confirming the case IDs and operation
sequences against the pinned Test Cases HTML.
