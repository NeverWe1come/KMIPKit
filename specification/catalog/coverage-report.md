# KMIP 2.1 inventory coverage

This report records inventory coverage and evidence state. It does not claim profile conformance or certification.

## Source documents

| Source ID | Title | Authority | SHA-256 |
| --- | --- | --- | --- |
| KMIPKIT-SRC-profiles | Key Management Interoperability Protocol Profiles Version 2.1 | profile\_normative | f11144aa793335f38dbd1864ec4709ca83ac80f588bbb790ba37f6c53be207a9 |
| KMIPKIT-SRC-spec | Key Management Interoperability Protocol Specification Version 2.1 | primary\_normative | 8bf9d914c097e98a6509aa1ffcbf03406f738066e940597aee93d0a5e07addcf |
| KMIPKIT-SRC-testcases | Key Management Interoperability Protocol Test Cases Version 2.1 | test\_evidence | 305e0638af25aa8163a2b4a67a1a617a0207af67b3b3032fa499966b22d42b88 |
| KMIPKIT-SRC-usage-guide | Key Management Interoperability Protocol Usage Guide Version 2.1 | informative | f6997ac3ac34d588b2b43ace49835dceaa6af2086f9472a79c27bce43f5cbe38 |

## Count reconciliation

| Record class | Count |
| --- | --- |
| Sources | 4 |
| Source clauses | 0 |
| Protocol elements | 468 |
| Client-to-server operations | 57 |
| Server-to-client operations | 5 |
| Tag ranges | 5 |
| Normative requirements | 0 |
| Profiles | 0 |
| Test cases | 0 |
| Open discrepancies | 0 |
| Project policies | 0 |

### Elements by kind

| Kind | Count |
| --- | --- |
| data\_type | 11 |
| object\_structure | 12 |
| object\_type | 9 |
| operation | 62 |
| tag | 374 |

### Requirements by strength and scope

| Dimension | Value | Count |
| --- | --- | --- |
| — | — | — |

## Source clause dispositions

| Disposition | Count |
| --- | --- |
| — | — |

## Unassigned requirements

| Requirement | Strength | Scope | Source |
| --- | --- | --- | --- |
| — | — | — | — |

## Unassigned protocol elements

| Element | Kind | Direction | Scope | Source |
| --- | --- | --- | --- | --- |
| KMIPKIT-ELEM-DATA-TYPE-BIG-INTEGER | data\_type | both | client\_1\_0 | KMIPKIT-SRC-spec §1.5, KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-DATA-TYPE-BOOLEAN | data\_type | both | client\_1\_0 | KMIPKIT-SRC-spec §1.5, KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-DATA-TYPE-BYTE-STRING | data\_type | both | client\_1\_0 | KMIPKIT-SRC-spec §1.5, KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-DATA-TYPE-DATE-TIME | data\_type | both | client\_1\_0 | KMIPKIT-SRC-spec §1.5, KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-DATA-TYPE-DATE-TIME-EXTENDED | data\_type | both | client\_1\_0 | KMIPKIT-SRC-spec §1.5, KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-DATA-TYPE-ENUMERATION | data\_type | both | client\_1\_0 | KMIPKIT-SRC-spec §1.5, KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-DATA-TYPE-INTEGER | data\_type | both | client\_1\_0 | KMIPKIT-SRC-spec §1.5, KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-DATA-TYPE-INTERVAL | data\_type | both | client\_1\_0 | KMIPKIT-SRC-spec §1.5, KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-DATA-TYPE-LONG-INTEGER | data\_type | both | client\_1\_0 | KMIPKIT-SRC-spec §1.5, KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-DATA-TYPE-STRUCTURE | data\_type | both | client\_1\_0 | KMIPKIT-SRC-spec §1.5, KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-DATA-TYPE-TEXT-STRING | data\_type | both | client\_1\_0 | KMIPKIT-SRC-spec §1.5, KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-CERTIFICATE | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §2.1 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-CERTIFICATE-REQUEST | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §2.2 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-KEY-BLOCK | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-KEY-VALUE | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §3.2 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-KEY-WRAPPING-DATA | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-OPAQUE-OBJECT | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §2.3 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-PGP-KEY | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §2.4 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-PRIVATE-KEY | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §2.5 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-PUBLIC-KEY | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §2.6 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-SECRET-DATA | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §2.7 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-SPLIT-KEY | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §2.8 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-SYMMETRIC-KEY | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §2.9 |
| KMIPKIT-ELEM-OBJECT-TYPE-CERTIFICATE | object\_type | both | client\_1\_0 | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-OBJECT-TYPE-CERTIFICATE-REQUEST | object\_type | both | client\_1\_0 | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-OBJECT-TYPE-OPAQUE-OBJECT | object\_type | both | client\_1\_0 | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-OBJECT-TYPE-PGP-KEY | object\_type | both | client\_1\_0 | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-OBJECT-TYPE-PRIVATE-KEY | object\_type | both | client\_1\_0 | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-OBJECT-TYPE-PUBLIC-KEY | object\_type | both | client\_1\_0 | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-OBJECT-TYPE-SECRET-DATA | object\_type | both | client\_1\_0 | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-OBJECT-TYPE-SPLIT-KEY | object\_type | both | client\_1\_0 | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-OBJECT-TYPE-SYMMETRIC-KEY | object\_type | both | client\_1\_0 | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-OP-C2S-ACTIVATE | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.1 |
| KMIPKIT-ELEM-OP-C2S-ADD-ATTRIBUTE | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.2 |
| KMIPKIT-ELEM-OP-C2S-ADJUST-ATTRIBUTE | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.3 |
| KMIPKIT-ELEM-OP-C2S-ARCHIVE | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.4 |
| KMIPKIT-ELEM-OP-C2S-CANCEL | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.5 |
| KMIPKIT-ELEM-OP-C2S-CERTIFY | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.6 |
| KMIPKIT-ELEM-OP-C2S-CHECK | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.7 |
| KMIPKIT-ELEM-OP-C2S-CREATE | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.8 |
| KMIPKIT-ELEM-OP-C2S-CREATE-KEY-PAIR | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.9 |
| KMIPKIT-ELEM-OP-C2S-CREATE-SPLIT-KEY | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.10 |
| KMIPKIT-ELEM-OP-C2S-DECRYPT | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.11 |
| KMIPKIT-ELEM-OP-C2S-DELEGATED-LOGIN | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.12 |
| KMIPKIT-ELEM-OP-C2S-DELETE-ATTRIBUTE | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.13 |
| KMIPKIT-ELEM-OP-C2S-DERIVE-KEY | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.14 |
| KMIPKIT-ELEM-OP-C2S-DESTROY | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.15 |
| KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.16 |
| KMIPKIT-ELEM-OP-C2S-ENCRYPT | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.17 |
| KMIPKIT-ELEM-OP-C2S-EXPORT | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.18 |
| KMIPKIT-ELEM-OP-C2S-GET | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.19 |
| KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.21 |
| KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.20 |
| KMIPKIT-ELEM-OP-C2S-GET-CONSTRAINTS | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.22 |
| KMIPKIT-ELEM-OP-C2S-GET-USAGE-ALLOCATION | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.23 |
| KMIPKIT-ELEM-OP-C2S-HASH | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.24 |
| KMIPKIT-ELEM-OP-C2S-IMPORT | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.25 |
| KMIPKIT-ELEM-OP-C2S-INTEROP | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.26 |
| KMIPKIT-ELEM-OP-C2S-JOIN-SPLIT-KEY | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.27 |
| KMIPKIT-ELEM-OP-C2S-LOCATE | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-ELEM-OP-C2S-LOG | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.29 |
| KMIPKIT-ELEM-OP-C2S-LOGIN | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.30 |
| KMIPKIT-ELEM-OP-C2S-LOGOUT | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.31 |
| KMIPKIT-ELEM-OP-C2S-MAC | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.32 |
| KMIPKIT-ELEM-OP-C2S-MAC-VERIFY | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.33 |
| KMIPKIT-ELEM-OP-C2S-MODIFY-ATTRIBUTE | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.34 |
| KMIPKIT-ELEM-OP-C2S-OBTAIN-LEASE | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.35 |
| KMIPKIT-ELEM-OP-C2S-PING | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.36 |
| KMIPKIT-ELEM-OP-C2S-PKCS-11 | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.37 |
| KMIPKIT-ELEM-OP-C2S-POLL | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.38 |
| KMIPKIT-ELEM-OP-C2S-PROCESS | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.39 |
| KMIPKIT-ELEM-OP-C2S-QUERY | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.40 |
| KMIPKIT-ELEM-OP-C2S-QUERY-ASYNCHRONOUS-REQUESTS | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.41 |
| KMIPKIT-ELEM-OP-C2S-RE-CERTIFY | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.45 |
| KMIPKIT-ELEM-OP-C2S-RE-KEY | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.46 |
| KMIPKIT-ELEM-OP-C2S-RE-KEY-KEY-PAIR | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.47 |
| KMIPKIT-ELEM-OP-C2S-RE-PROVISION | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.48 |
| KMIPKIT-ELEM-OP-C2S-RECOVER | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.42 |
| KMIPKIT-ELEM-OP-C2S-REGISTER | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.43 |
| KMIPKIT-ELEM-OP-C2S-REVOKE | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.44 |
| KMIPKIT-ELEM-OP-C2S-RNG-RETRIEVE | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.49 |
| KMIPKIT-ELEM-OP-C2S-RNG-SEED | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.50 |
| KMIPKIT-ELEM-OP-C2S-SET-ATTRIBUTE | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.51 |
| KMIPKIT-ELEM-OP-C2S-SET-CONSTRAINTS | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.52 |
| KMIPKIT-ELEM-OP-C2S-SET-DEFAULTS | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.53 |
| KMIPKIT-ELEM-OP-C2S-SET-ENDPOINT-ROLE | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.54 |
| KMIPKIT-ELEM-OP-C2S-SIGN | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.55 |
| KMIPKIT-ELEM-OP-C2S-SIGNATURE-VERIFY | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.56 |
| KMIPKIT-ELEM-OP-C2S-VALIDATE | operation | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §6.1.57 |
| KMIPKIT-ELEM-OP-S2C-DISCOVER-VERSIONS | operation | server\_to\_client | client\_1\_1 | KMIPKIT-SRC-spec §6.2.1 |
| KMIPKIT-ELEM-OP-S2C-NOTIFY | operation | server\_to\_client | client\_1\_1 | KMIPKIT-SRC-spec §6.2.2 |
| KMIPKIT-ELEM-OP-S2C-PUT | operation | server\_to\_client | client\_1\_1 | KMIPKIT-SRC-spec §6.2.3 |
| KMIPKIT-ELEM-OP-S2C-QUERY | operation | server\_to\_client | client\_1\_1 | KMIPKIT-SRC-spec §6.2.4 |
| KMIPKIT-ELEM-OP-S2C-SET-ENDPOINT-ROLE | operation | server\_to\_client | client\_1\_1 | KMIPKIT-SRC-spec §6.2.5 |
| KMIPKIT-ELEM-TAG-420001 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420002 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420003 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420004 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420005 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420006 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420007 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420008 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420009 | tag | both | out\_of\_scope | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42000A | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42000B | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42000C | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42000D | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42000E | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42000F | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420010 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420011 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420012 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420013 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420014 | tag | both | out\_of\_scope | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420015 | tag | both | out\_of\_scope | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420016 | tag | both | out\_of\_scope | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420017 | tag | both | out\_of\_scope | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420018 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420019 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42001A | tag | both | out\_of\_scope | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42001B | tag | both | out\_of\_scope | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42001C | tag | both | out\_of\_scope | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42001D | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42001E | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42001F | tag | both | out\_of\_scope | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420020 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420021 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420022 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420023 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420024 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420025 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420026 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420027 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420028 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420029 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42002A | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42002B | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42002C | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42002D | tag | both | out\_of\_scope | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42002E | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42002F | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420030 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420031 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420032 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420033 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420034 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420035 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420036 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420037 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420038 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420039 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42003A | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42003B | tag | both | out\_of\_scope | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42003C | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42003D | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42003E | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42003F | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420040 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420041 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420042 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420043 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420044 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420045 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420046 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420047 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420048 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420049 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42004A | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42004B | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42004C | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42004D | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42004E | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42004F | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420050 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420051 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420052 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420053 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420054 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420055 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420056 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420057 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420058 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420059 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42005A | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42005B | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42005C | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42005D | tag | both | out\_of\_scope | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42005E | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42005F | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420060 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420061 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420062 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420063 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420064 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420065 | tag | both | out\_of\_scope | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420066 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420067 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420068 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420069 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42006A | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42006B | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42006C | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42006D | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42006E | tag | both | out\_of\_scope | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42006F | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420070 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420071 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420072 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420073 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420074 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420075 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420076 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420077 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420078 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420079 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42007A | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42007B | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42007C | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42007D | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42007E | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42007F | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420080 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420081 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420082 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420083 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420084 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420085 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420086 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420087 | tag | both | out\_of\_scope | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420088 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420089 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42008A | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42008B | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42008C | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42008D | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42008E | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42008F | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420090 | tag | both | out\_of\_scope | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420091 | tag | both | out\_of\_scope | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420092 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420093 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420094 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420095 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420096 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420097 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420098 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420099 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42009A | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42009B | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42009C | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42009D | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42009E | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42009F | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200A0 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200A1 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200A2 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200A3 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200A4 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200A5 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200A6 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200A7 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200A8 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200A9 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200AA | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200AB | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200AC | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200AD | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200AE | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200AF | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200B0 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200B1 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200B2 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200B3 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200B4 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200B5 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200B6 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200B7 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200B8 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200B9 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200BA | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200BB | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200BC | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200BD | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200BE | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200BF | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200C0 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200C1 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200C2 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200C3 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200C4 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200C5 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200C6 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200C7 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200C8 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200C9 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200CA | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200CB | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200CC | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200CD | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200CE | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200CF | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200D0 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200D1 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200D2 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200D3 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200D4 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200D5 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200D6 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200D7 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200D8 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200D9 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200DA | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200DB | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200DC | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200DD | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200DE | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200DF | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200E0 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200E1 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200E2 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200E3 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200E4 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200E5 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200E6 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200E7 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200E8 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200E9 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200EA | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200EB | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200EC | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200ED | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200EE | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200EF | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200F0 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200F1 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200F2 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200F3 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200F4 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200F5 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200F6 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200F7 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200F8 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200F9 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200FA | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200FB | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200FC | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200FD | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200FE | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-4200FF | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420100 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420101 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420102 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420103 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420104 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420105 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420106 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420107 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420108 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420109 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42010A | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42010B | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42010C | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42010D | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42010E | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42010F | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420110 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420111 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420112 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420113 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420114 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420115 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420116 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420117 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420118 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420119 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42011A | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42011B | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42011C | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42011D | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42011E | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42011F | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420120 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420121 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420122 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420123 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420124 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420125 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420126 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420127 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420128 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420129 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42012A | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42012B | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42012C | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42012D | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42012E | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42012F | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420130 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420131 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420132 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420133 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420134 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420135 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420136 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420137 | tag | both | out\_of\_scope | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420138 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420139 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42013A | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42013B | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42013C | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42013D | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42013E | tag | both | out\_of\_scope | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42013F | tag | both | out\_of\_scope | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420140 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420141 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420142 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420143 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420144 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420145 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420146 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420147 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420148 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420149 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42014A | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42014B | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42014C | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42014D | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42014E | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42014F | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420150 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420151 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420152 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420153 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420154 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420155 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420156 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420157 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420158 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420159 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42015A | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42015B | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42015C | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42015D | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42015E | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42015F | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420160 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420161 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420162 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420163 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420164 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420165 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420166 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420167 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420168 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420169 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42016A | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42016B | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42016C | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42016D | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42016E | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-42016F | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420170 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420171 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420172 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420173 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420174 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420175 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-TAG-420176 | tag | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |

## Profile states

| Profile | Name | Applicability | Claim state | Source |
| --- | --- | --- | --- | --- |
| — | — | — | — | — |

### Profiles by applicability and claim state

| Dimension | Value | Count |
| --- | --- | --- |
| — | — | — |

## Test fixture availability

| Test | Official ID | Fixture status | Local fixture |
| --- | --- | --- | --- |
| — | — | — | — |

### Test evidence by fixture availability

| Fixture state | Count |
| --- | --- |
| — | — |

## Open discrepancies

| Discrepancy | State | Summary | Source |
| --- | --- | --- | --- |
| — | — | — | — |

## Project policies

| Policy | Provenance | Summary |
| --- | --- | --- |
| — | — | — |
