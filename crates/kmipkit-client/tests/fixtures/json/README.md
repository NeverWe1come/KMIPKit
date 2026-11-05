# Official JSON encoding test fixture

`MSGENC-JSON-M-1-21.xml` is an exact OASIS source copy downloaded on
2026-11-05 from [KMIP Profiles Version 2.1, OASIS Standard, 14 December
2020](https://docs.oasis-open.org/kmip/kmip-profiles/v2.1/os/test-cases/kmip-v2.1/mandatory/MSGENC-JSON-M-1-21.xml).
SHA-256: `681e7ccd518fc4d25b66077567868ab63652418a81e44625e4bfdc8c5ab70cfe`.
The directory's `.gitattributes` prevents line-ending conversion of this copy.

Profiles §5.5.4.1 defines two Query exchanges. The test description is XML,
not four literal JSON bodies. The codec test converts each described semantic
message to JSON and binds `$NOW` to 1700000000. It replays all four messages,
including the first ResponseTooLarge failure and subsequent successful Query
response. This is codec fixture evidence; live client/server sequencing,
Baseline Client, HTTPS, and complete JSON profile conformance remain separate
gates owned by KMIPKIT-0057 and the integration programme.
