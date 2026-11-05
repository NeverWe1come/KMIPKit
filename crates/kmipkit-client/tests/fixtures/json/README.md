# Official JSON encoding test fixture

`MSGENC-JSON-M-1-21.xml` is an exact OASIS source copy downloaded on
2026-11-05 from [KMIP Profiles Version 2.1, OASIS Standard, 14 December
2020](https://docs.oasis-open.org/kmip/kmip-profiles/v2.1/os/test-cases/kmip-v2.1/mandatory/MSGENC-JSON-M-1-21.xml).
SHA-256: `681e7ccd518fc4d25b66077567868ab63652418a81e44625e4bfdc8c5ab70cfe`.
The directory's `.gitattributes` prevents line-ending conversion of this copy.

Profiles Â§5.5.4.1 defines two Query exchanges. The test description is XML,
not four literal JSON bodies. The codec test converts each described semantic
message to JSON and binds `$NOW` to 1700000000. It replays all four messages,
including the first ResponseTooLarge failure and subsequent successful Query
response. This is codec fixture evidence; live client/server sequencing,
Baseline Client, HTTPS, and complete JSON profile conformance remain separate
gates owned by KMIPKIT-0057 and the integration programme.


`request-time-0.json`, `response-time-0.json`, and `request-time-1.json`
contain the deterministic normative JSON wire examples from the pinned
Profiles HTML §5.5.4.1 table immediately following “The normative corresponding
wire encoding in JSON”. The offline `extract_profiles_json.py` utility strips
HTML markup/entities and converts indentation nonbreaking spaces to ordinary
spaces; all JSON tokens, versions, values and timestamps are retained exactly.
Pinned source HTML SHA-256: `f11144aa793335f38dbd1864ec4709ca83ac80f588bbb790ba37f6c53be207a9`.
Run it from any directory with Python to reproduce these three fixtures.
Their SHA-256 values are:

| Fixture | SHA-256 |
|---|---|
| request-time-0.json | 51a75d58c1df8e77a4b614a30cdf26fd2a5219bc7347463b38c5996d33810ae4 |
| response-time-0.json | 431bc2cd5b9689354e4ef6b32692d3a1c86f45e68ee72c599a7754f390a31368 |
| request-time-1.json | ed49a233d4ec805dad556c4a50b4424a13824275876971acdb45f6b84d3b8ece |

These HTML examples retain ProtocolVersion 1.4. They are exercised as literal
generic Item codec vectors, not as authorized KMIPKit client protocol versions.
The successful Query response is explicitly variable; its historical sample
also lists the old Template object type. The semantic descriptor replay checks
the successful response separately and does not promote Template to a KMIP
2.1 assigned symbolic name.

The normative-wire test compares emitted JSON to each source example with
only profile-permitted representation choices normalized: explicit Structure
types, decimal Integer values and a 64-bit hexadecimal DateTime. It does not
claim whitespace/representation byte identity where the profile permits
several forms. The descriptor-derived test remains separate semantic evidence.
