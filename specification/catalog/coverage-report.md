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
| Protocol elements | 62 |
| Client-to-server operations | 57 |
| Server-to-client operations | 5 |
| Tag ranges | 0 |
| Normative requirements | 0 |
| Profiles | 0 |
| Test cases | 0 |
| Open discrepancies | 0 |
| Project policies | 0 |

### Elements by kind

| Kind | Count |
| --- | --- |
| operation | 62 |

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
