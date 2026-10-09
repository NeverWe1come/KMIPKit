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
| Source clauses | 1411 |
| Protocol elements | 1750 |
| Client-to-server operations | 57 |
| Server-to-client operations | 5 |
| Tag ranges | 5 |
| Normative requirements | 566 |
| Profiles | 35 |
| Test cases | 203 |
| Open discrepancies | 42 |
| Project policies | 4 |

### Elements by kind

| Kind | Count |
| --- | --- |
| attribute | 63 |
| attribute\_structure | 7 |
| bitmask | 3 |
| bitmask\_value | 44 |
| credential | 7 |
| data\_type | 11 |
| enumeration | 64 |
| enumeration\_value | 723 |
| message\_field | 71 |
| object\_structure | 23 |
| object\_type | 9 |
| operation | 62 |
| operation\_structure | 41 |
| option | 3 |
| result | 4 |
| structure\_member | 241 |
| tag | 374 |

### Requirements by strength and scope

| Dimension | Value | Count |
| --- | --- | --- |
| Strength | discouraged | 2 |
| Strength | mandatory | 316 |
| Strength | permission\_or\_optional | 180 |
| Strength | prohibited | 47 |
| Strength | recommended | 21 |
| Scope | client\_1\_0 | 368 |
| Scope | out\_of\_scope | 1 |
| Scope | profile\_conditional | 194 |
| Scope | server\_only | 3 |
| Direction | both | 14 |
| Direction | client\_to\_server | 378 |
| Direction | server\_to\_client | 16 |
| Direction | unclear | 158 |

## Tag allocation and ranges

### Named tags by OASIS allocation

| Allocation | Count |
| --- | --- |
| assigned | 354 |
| reserved | 20 |

### Tag ranges

| Range | Value range | Allocation | Source |
| --- | --- | --- | --- |
| KMIPKIT-RANGE-001 | 000000 - 420000 | unused | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-RANGE-002 | 420XXX – 42FFFF | reserved | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-RANGE-003 | 430000 – 53FFFF | unused | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-RANGE-004 | 540000 – 54FFFF | extension | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-RANGE-005 | 550000 - FFFFFF | unused | KMIPKIT-SRC-spec §11.56 |

## Source clause dispositions

| Disposition | Count |
| --- | --- |
| informative\_context | 118 |
| later\_1\_1 | 38 |
| non\_applicable | 346 |
| profile\_conditional | 184 |
| requirement | 262 |
| server\_only | 436 |
| source\_discrepancy | 27 |

## Source clause review by section

Every row summarizes audited candidate locators by their pinned source section. The immutable-source audit separately requires exact candidate and clause-ledger locator equality.

| Source | Section | Candidates | Dispositions |
| --- | --- | --- | --- |
| KMIPKIT-SRC-profiles | 0 | 12 | informative\_context: 4, non\_applicable: 8 |
| KMIPKIT-SRC-profiles | 1 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-profiles | 1.1 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-profiles | 1.2 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-profiles | 1.3 | 2 | non\_applicable: 2 |
| KMIPKIT-SRC-profiles | 2 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-profiles | 2.1 | 9 | non\_applicable: 9 |
| KMIPKIT-SRC-profiles | 2.2 | 5 | non\_applicable: 5 |
| KMIPKIT-SRC-profiles | 3 | 6 | non\_applicable: 2, profile\_conditional: 4 |
| KMIPKIT-SRC-profiles | 3.1 | 1 | profile\_conditional: 1 |
| KMIPKIT-SRC-profiles | 3.1.1 | 5 | profile\_conditional: 3, server\_only: 2 |
| KMIPKIT-SRC-profiles | 3.1.2 | 5 | non\_applicable: 2, profile\_conditional: 2, server\_only: 1 |
| KMIPKIT-SRC-profiles | 3.1.3 | 3 | server\_only: 3 |
| KMIPKIT-SRC-profiles | 3.1.4 | 1 | server\_only: 1 |
| KMIPKIT-SRC-profiles | 3.2 | 1 | profile\_conditional: 1 |
| KMIPKIT-SRC-profiles | 3.2.1 | 1 | profile\_conditional: 1 |
| KMIPKIT-SRC-profiles | 3.2.2 | 1 | profile\_conditional: 1 |
| KMIPKIT-SRC-profiles | 3.2.3 | 1 | profile\_conditional: 1 |
| KMIPKIT-SRC-profiles | 3.2.4 | 2 | requirement: 1, server\_only: 1 |
| KMIPKIT-SRC-profiles | 4 | 4 | informative\_context: 4 |
| KMIPKIT-SRC-profiles | 4.1 | 3 | informative\_context: 1, profile\_conditional: 2 |
| KMIPKIT-SRC-profiles | 4.1.1 | 14 | server\_only: 14 |
| KMIPKIT-SRC-profiles | 4.1.2 | 10 | informative\_context: 4, profile\_conditional: 5, server\_only: 1 |
| KMIPKIT-SRC-profiles | 5.1.1 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-profiles | 5.1.2 | 1 | server\_only: 1 |
| KMIPKIT-SRC-profiles | 5.3.1 | 11 | later\_1\_1: 1, non\_applicable: 2, requirement: 8 |
| KMIPKIT-SRC-profiles | 5.3.2 | 10 | server\_only: 10 |
| KMIPKIT-SRC-profiles | 5.3.3.1 | 1 | server\_only: 1 |
| KMIPKIT-SRC-profiles | 5.4 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-profiles | 5.4.1.1 | 2 | non\_applicable: 2 |
| KMIPKIT-SRC-profiles | 5.4.1.2 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-profiles | 5.4.1.3 | 2 | non\_applicable: 2 |
| KMIPKIT-SRC-profiles | 5.4.1.4 | 2 | non\_applicable: 2 |
| KMIPKIT-SRC-profiles | 5.4.1.6 | 3 | non\_applicable: 3 |
| KMIPKIT-SRC-profiles | 5.4.1.6.1 | 2 | non\_applicable: 2 |
| KMIPKIT-SRC-profiles | 5.4.1.6.2 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-profiles | 5.4.1.6.4 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-profiles | 5.4.1.6.7 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-profiles | 5.4.1.6.11 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-profiles | 5.4.2 | 4 | non\_applicable: 2, source\_discrepancy: 2 |
| KMIPKIT-SRC-profiles | 5.4.3 | 4 | non\_applicable: 2, source\_discrepancy: 2 |
| KMIPKIT-SRC-profiles | 5.4.4.1 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-profiles | 5.5 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-profiles | 5.5.1.1 | 2 | non\_applicable: 2 |
| KMIPKIT-SRC-profiles | 5.5.1.2 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-profiles | 5.5.1.3 | 2 | non\_applicable: 2 |
| KMIPKIT-SRC-profiles | 5.5.1.4 | 2 | non\_applicable: 2 |
| KMIPKIT-SRC-profiles | 5.5.1.6 | 2 | non\_applicable: 2 |
| KMIPKIT-SRC-profiles | 5.5.1.6.1 | 2 | non\_applicable: 2 |
| KMIPKIT-SRC-profiles | 5.5.1.6.2 | 2 | non\_applicable: 2 |
| KMIPKIT-SRC-profiles | 5.5.1.6.4 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-profiles | 5.5.1.6.5 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-profiles | 5.5.1.6.6 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-profiles | 5.5.1.6.7 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-profiles | 5.5.1.6.8 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-profiles | 5.5.1.6.9 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-profiles | 5.5.1.6.10 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-profiles | 5.5.1.6.11 | 3 | non\_applicable: 3 |
| KMIPKIT-SRC-profiles | 5.5.1.6.13 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-profiles | 5.5.2 | 4 | non\_applicable: 4 |
| KMIPKIT-SRC-profiles | 5.5.3 | 4 | non\_applicable: 4 |
| KMIPKIT-SRC-profiles | 5.5.4.1 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-profiles | 5.6.1 | 3 | profile\_conditional: 3 |
| KMIPKIT-SRC-profiles | 5.6.2 | 7 | server\_only: 7 |
| KMIPKIT-SRC-profiles | 5.6.4.1 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-profiles | 5.7.1 | 3 | profile\_conditional: 3 |
| KMIPKIT-SRC-profiles | 5.7.2 | 3 | profile\_conditional: 3 |
| KMIPKIT-SRC-profiles | 5.7.3 | 3 | profile\_conditional: 3 |
| KMIPKIT-SRC-profiles | 5.7.4 | 7 | server\_only: 7 |
| KMIPKIT-SRC-profiles | 5.8.1 | 3 | profile\_conditional: 3 |
| KMIPKIT-SRC-profiles | 5.8.2 | 6 | server\_only: 6 |
| KMIPKIT-SRC-profiles | 5.8.4.1 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-profiles | 5.9.1 | 4 | profile\_conditional: 4 |
| KMIPKIT-SRC-profiles | 5.9.2 | 4 | profile\_conditional: 4 |
| KMIPKIT-SRC-profiles | 5.9.3 | 4 | profile\_conditional: 4 |
| KMIPKIT-SRC-profiles | 5.9.4 | 4 | server\_only: 4 |
| KMIPKIT-SRC-profiles | 5.9.5 | 4 | server\_only: 4 |
| KMIPKIT-SRC-profiles | 5.9.6 | 4 | server\_only: 4 |
| KMIPKIT-SRC-profiles | 5.9.10.1 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-profiles | 5.9.10.2 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-profiles | 5.9.10.3 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-profiles | 5.9.10.4 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-profiles | 5.10.1 | 3 | profile\_conditional: 3 |
| KMIPKIT-SRC-profiles | 5.10.2 | 7 | server\_only: 7 |
| KMIPKIT-SRC-profiles | 5.10.4.1 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-profiles | 5.11.1 | 4 | profile\_conditional: 4 |
| KMIPKIT-SRC-profiles | 5.11.2 | 11 | server\_only: 11 |
| KMIPKIT-SRC-profiles | 5.11.3.1 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-profiles | 5.11.3.2 | 1 | server\_only: 1 |
| KMIPKIT-SRC-profiles | 5.12.1 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-profiles | 5.12.2 | 7 | informative\_context: 1, profile\_conditional: 5, server\_only: 1 |
| KMIPKIT-SRC-profiles | 5.12.3 | 3 | profile\_conditional: 2, server\_only: 1 |
| KMIPKIT-SRC-profiles | 5.12.4 | 9 | profile\_conditional: 9 |
| KMIPKIT-SRC-profiles | 5.12.5 | 12 | server\_only: 12 |
| KMIPKIT-SRC-profiles | 5.12.6.1 | 1 | profile\_conditional: 1 |
| KMIPKIT-SRC-profiles | 5.12.6.2 | 4 | informative\_context: 1, profile\_conditional: 3 |
| KMIPKIT-SRC-profiles | 5.12.6.3 | 4 | profile\_conditional: 4 |
| KMIPKIT-SRC-profiles | 5.13.1 | 3 | profile\_conditional: 2, source\_discrepancy: 1 |
| KMIPKIT-SRC-profiles | 5.13.2 | 7 | server\_only: 7 |
| KMIPKIT-SRC-profiles | 5.15 | 4 | profile\_conditional: 4 |
| KMIPKIT-SRC-profiles | 5.16 | 10 | server\_only: 10 |
| KMIPKIT-SRC-profiles | 5.17 | 1 | profile\_conditional: 1 |
| KMIPKIT-SRC-profiles | 5.17.1 | 2 | profile\_conditional: 2 |
| KMIPKIT-SRC-profiles | 5.17.2 | 2 | informative\_context: 1, profile\_conditional: 1 |
| KMIPKIT-SRC-profiles | 5.18.1 | 5 | non\_applicable: 3, profile\_conditional: 2 |
| KMIPKIT-SRC-profiles | 5.18.2.1 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-profiles | 5.18.3 | 5 | profile\_conditional: 4, source\_discrepancy: 1 |
| KMIPKIT-SRC-profiles | 5.18.4 | 5 | server\_only: 4, source\_discrepancy: 1 |
| KMIPKIT-SRC-profiles | 6 | 1 | profile\_conditional: 1 |
| KMIPKIT-SRC-profiles | 6.1 | 4 | profile\_conditional: 4 |
| KMIPKIT-SRC-profiles | 6.2 | 4 | server\_only: 4 |
| KMIPKIT-SRC-profiles | 6.3 | 4 | server\_only: 4 |
| KMIPKIT-SRC-profiles | 6.4 | 5 | profile\_conditional: 5 |
| KMIPKIT-SRC-profiles | 6.5 | 5 | server\_only: 5 |
| KMIPKIT-SRC-profiles | 6.6 | 5 | non\_applicable: 5 |
| KMIPKIT-SRC-profiles | 6.7 | 6 | non\_applicable: 6 |
| KMIPKIT-SRC-profiles | 6.8 | 5 | non\_applicable: 5 |
| KMIPKIT-SRC-profiles | 6.9 | 6 | non\_applicable: 5, source\_discrepancy: 1 |
| KMIPKIT-SRC-profiles | 6.10 | 5 | profile\_conditional: 5 |
| KMIPKIT-SRC-profiles | 6.11 | 5 | server\_only: 5 |
| KMIPKIT-SRC-profiles | 6.12 | 5 | profile\_conditional: 5 |
| KMIPKIT-SRC-profiles | 6.13 | 5 | profile\_conditional: 4, source\_discrepancy: 1 |
| KMIPKIT-SRC-profiles | 6.14 | 5 | profile\_conditional: 4, source\_discrepancy: 1 |
| KMIPKIT-SRC-profiles | 6.15 | 7 | server\_only: 7 |
| KMIPKIT-SRC-profiles | 6.16 | 5 | profile\_conditional: 5 |
| KMIPKIT-SRC-profiles | 6.17 | 5 | server\_only: 5 |
| KMIPKIT-SRC-profiles | 6.18 | 5 | profile\_conditional: 5 |
| KMIPKIT-SRC-profiles | 6.19 | 5 | profile\_conditional: 5 |
| KMIPKIT-SRC-profiles | 6.20 | 5 | profile\_conditional: 5 |
| KMIPKIT-SRC-profiles | 6.21 | 5 | server\_only: 5 |
| KMIPKIT-SRC-profiles | 6.22 | 5 | server\_only: 5 |
| KMIPKIT-SRC-profiles | 6.23 | 5 | server\_only: 5 |
| KMIPKIT-SRC-profiles | 6.24 | 5 | profile\_conditional: 5 |
| KMIPKIT-SRC-profiles | 6.25 | 5 | server\_only: 5 |
| KMIPKIT-SRC-profiles | 6.26 | 5 | profile\_conditional: 5 |
| KMIPKIT-SRC-profiles | 6.27 | 5 | server\_only: 5 |
| KMIPKIT-SRC-profiles | 6.28 | 7 | profile\_conditional: 7 |
| KMIPKIT-SRC-profiles | 6.29 | 7 | server\_only: 7 |
| KMIPKIT-SRC-profiles | 6.30 | 5 | profile\_conditional: 5 |
| KMIPKIT-SRC-profiles | 6.31 | 5 | server\_only: 5 |
| KMIPKIT-SRC-profiles | 6.32 | 5 | profile\_conditional: 5 |
| KMIPKIT-SRC-profiles | 6.33 | 5 | server\_only: 5 |
| KMIPKIT-SRC-profiles | 6.34 | 5 | profile\_conditional: 5 |
| KMIPKIT-SRC-profiles | 6.35 | 5 | server\_only: 5 |
| KMIPKIT-SRC-spec | 0 | 8 | informative\_context: 6, non\_applicable: 2 |
| KMIPKIT-SRC-spec | 1.1 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-spec | 1.2 | 3 | informative\_context: 3 |
| KMIPKIT-SRC-spec | 1.3 | 7 | informative\_context: 7 |
| KMIPKIT-SRC-spec | 2.1 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 2.2 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 2.3 | 3 | requirement: 3 |
| KMIPKIT-SRC-spec | 2.4 | 3 | requirement: 3 |
| KMIPKIT-SRC-spec | 2.5 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 2.6 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 2.7 | 2 | requirement: 2 |
| KMIPKIT-SRC-spec | 2.8 | 3 | requirement: 3 |
| KMIPKIT-SRC-spec | 2.9 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 3.1 | 6 | requirement: 6 |
| KMIPKIT-SRC-spec | 3.2 | 2 | requirement: 2 |
| KMIPKIT-SRC-spec | 3.3 | 9 | requirement: 9 |
| KMIPKIT-SRC-spec | 3.4 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 3.5 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 3.6 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 3.7 | 2 | requirement: 2 |
| KMIPKIT-SRC-spec | 3.8 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 3.9 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 3.10 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 3.11 | 2 | informative\_context: 1, requirement: 1 |
| KMIPKIT-SRC-spec | 3.12 | 2 | informative\_context: 1, requirement: 1 |
| KMIPKIT-SRC-spec | 4 | 8 | informative\_context: 2, requirement: 1, server\_only: 5 |
| KMIPKIT-SRC-spec | 4.1 | 2 | informative\_context: 1, requirement: 1 |
| KMIPKIT-SRC-spec | 4.2 | 3 | informative\_context: 1, requirement: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 4.3 | 2 | server\_only: 2 |
| KMIPKIT-SRC-spec | 4.4 | 3 | informative\_context: 1, requirement: 2 |
| KMIPKIT-SRC-spec | 4.5 | 2 | informative\_context: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 4.6 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-spec | 4.7 | 2 | server\_only: 2 |
| KMIPKIT-SRC-spec | 4.8 | 2 | server\_only: 2 |
| KMIPKIT-SRC-spec | 4.9 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-spec | 4.10 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-spec | 4.11 | 2 | informative\_context: 1, requirement: 1 |
| KMIPKIT-SRC-spec | 4.12 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-spec | 4.13 | 2 | server\_only: 2 |
| KMIPKIT-SRC-spec | 4.14 | 5 | informative\_context: 4, requirement: 1 |
| KMIPKIT-SRC-spec | 4.15 | 2 | server\_only: 2 |
| KMIPKIT-SRC-spec | 4.16 | 5 | informative\_context: 1, non\_applicable: 1, requirement: 3 |
| KMIPKIT-SRC-spec | 4.17 | 2 | requirement: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 4.18 | 2 | informative\_context: 1, requirement: 1 |
| KMIPKIT-SRC-spec | 4.19 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-spec | 4.20 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-spec | 4.21 | 3 | requirement: 1, server\_only: 2 |
| KMIPKIT-SRC-spec | 4.22 | 2 | server\_only: 2 |
| KMIPKIT-SRC-spec | 4.23 | 2 | server\_only: 2 |
| KMIPKIT-SRC-spec | 4.24 | 2 | server\_only: 2 |
| KMIPKIT-SRC-spec | 4.25 | 2 | server\_only: 2 |
| KMIPKIT-SRC-spec | 4.26 | 3 | requirement: 1, server\_only: 2 |
| KMIPKIT-SRC-spec | 4.27 | 3 | informative\_context: 1, requirement: 2 |
| KMIPKIT-SRC-spec | 4.28 | 3 | informative\_context: 1, requirement: 2 |
| KMIPKIT-SRC-spec | 4.29 | 1 | server\_only: 1 |
| KMIPKIT-SRC-spec | 4.30 | 2 | informative\_context: 1, requirement: 1 |
| KMIPKIT-SRC-spec | 4.31 | 4 | informative\_context: 2, requirement: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 4.32 | 3 | informative\_context: 1, requirement: 2 |
| KMIPKIT-SRC-spec | 4.33 | 3 | server\_only: 3 |
| KMIPKIT-SRC-spec | 4.34 | 2 | informative\_context: 1, source\_discrepancy: 1 |
| KMIPKIT-SRC-spec | 4.35 | 2 | informative\_context: 1, requirement: 1 |
| KMIPKIT-SRC-spec | 4.36 | 2 | server\_only: 2 |
| KMIPKIT-SRC-spec | 4.37 | 2 | server\_only: 2 |
| KMIPKIT-SRC-spec | 4.38 | 4 | informative\_context: 1, requirement: 2, server\_only: 1 |
| KMIPKIT-SRC-spec | 4.39 | 3 | informative\_context: 1, server\_only: 2 |
| KMIPKIT-SRC-spec | 4.40 | 2 | informative\_context: 1, requirement: 1 |
| KMIPKIT-SRC-spec | 4.41 | 2 | informative\_context: 1, requirement: 1 |
| KMIPKIT-SRC-spec | 4.42 | 2 | informative\_context: 2 |
| KMIPKIT-SRC-spec | 4.43 | 2 | informative\_context: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 4.44 | 1 | server\_only: 1 |
| KMIPKIT-SRC-spec | 4.45 | 2 | informative\_context: 2 |
| KMIPKIT-SRC-spec | 4.46 | 6 | informative\_context: 1, requirement: 1, server\_only: 4 |
| KMIPKIT-SRC-spec | 4.47 | 3 | informative\_context: 1, requirement: 2 |
| KMIPKIT-SRC-spec | 4.48 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-spec | 4.49 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-spec | 4.50 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-spec | 4.51 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-spec | 4.52 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-spec | 4.53 | 2 | informative\_context: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 4.54 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-spec | 4.55 | 2 | server\_only: 2 |
| KMIPKIT-SRC-spec | 4.56 | 3 | server\_only: 3 |
| KMIPKIT-SRC-spec | 4.57 | 12 | requirement: 6, server\_only: 5, source\_discrepancy: 1 |
| KMIPKIT-SRC-spec | 4.58 | 2 | server\_only: 2 |
| KMIPKIT-SRC-spec | 4.59 | 4 | informative\_context: 1, requirement: 2, server\_only: 1 |
| KMIPKIT-SRC-spec | 4.60 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 4.61 | 3 | requirement: 1, server\_only: 2 |
| KMIPKIT-SRC-spec | 4.62 | 4 | requirement: 2, server\_only: 2 |
| KMIPKIT-SRC-spec | 4.63 | 6 | requirement: 3, server\_only: 3 |
| KMIPKIT-SRC-spec | 5.1 | 2 | requirement: 2 |
| KMIPKIT-SRC-spec | 5.2 | 2 | requirement: 2 |
| KMIPKIT-SRC-spec | 5.3 | 2 | requirement: 2 |
| KMIPKIT-SRC-spec | 5.4 | 2 | requirement: 2 |
| KMIPKIT-SRC-spec | 5.5 | 2 | requirement: 2 |
| KMIPKIT-SRC-spec | 5.6 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 5.7 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 6.1 | 10 | informative\_context: 4, requirement: 4, server\_only: 2 |
| KMIPKIT-SRC-spec | 6.1.1 | 3 | non\_applicable: 2, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.1.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.2 | 3 | non\_applicable: 2, requirement: 1 |
| KMIPKIT-SRC-spec | 6.1.2.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.3 | 3 | non\_applicable: 2, requirement: 1 |
| KMIPKIT-SRC-spec | 6.1.3.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.4 | 3 | non\_applicable: 2, requirement: 1 |
| KMIPKIT-SRC-spec | 6.1.4.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.5 | 5 | non\_applicable: 3, requirement: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.6 | 8 | non\_applicable: 2, requirement: 4, server\_only: 2 |
| KMIPKIT-SRC-spec | 6.1.6.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.7 | 8 | non\_applicable: 2, requirement: 5, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.7.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.8 | 3 | non\_applicable: 2, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.8.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.9 | 7 | non\_applicable: 2, requirement: 3, server\_only: 2 |
| KMIPKIT-SRC-spec | 6.1.9.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.10 | 5 | non\_applicable: 2, requirement: 1, server\_only: 2 |
| KMIPKIT-SRC-spec | 6.1.10.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.11 | 7 | non\_applicable: 2, requirement: 5 |
| KMIPKIT-SRC-spec | 6.1.11.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.12 | 4 | informative\_context: 1, non\_applicable: 2, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.12.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.13 | 3 | non\_applicable: 2, requirement: 1 |
| KMIPKIT-SRC-spec | 6.1.13.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.14 | 8 | non\_applicable: 2, requirement: 4, server\_only: 2 |
| KMIPKIT-SRC-spec | 6.1.14.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.15 | 3 | non\_applicable: 2, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.15.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.16 | 6 | non\_applicable: 2, requirement: 2, server\_only: 2 |
| KMIPKIT-SRC-spec | 6.1.16.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.17 | 7 | non\_applicable: 2, requirement: 4, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.17.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.18 | 4 | non\_applicable: 2, server\_only: 2 |
| KMIPKIT-SRC-spec | 6.1.18.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.19 | 7 | non\_applicable: 2, requirement: 2, server\_only: 2, source\_discrepancy: 1 |
| KMIPKIT-SRC-spec | 6.1.19.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.20 | 5 | non\_applicable: 2, requirement: 2, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.20.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.21 | 4 | non\_applicable: 2, server\_only: 2 |
| KMIPKIT-SRC-spec | 6.1.21.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.22 | 2 | non\_applicable: 2 |
| KMIPKIT-SRC-spec | 6.1.22.1 | 2 | non\_applicable: 1, source\_discrepancy: 1 |
| KMIPKIT-SRC-spec | 6.1.23 | 4 | non\_applicable: 2, requirement: 2 |
| KMIPKIT-SRC-spec | 6.1.23.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.24 | 2 | non\_applicable: 2 |
| KMIPKIT-SRC-spec | 6.1.24.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.25 | 7 | non\_applicable: 2, requirement: 3, server\_only: 2 |
| KMIPKIT-SRC-spec | 6.1.25.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.26 | 3 | non\_applicable: 2, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.26.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.27 | 6 | non\_applicable: 2, requirement: 3, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.27.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.28 | 16 | non\_applicable: 2, requirement: 9, server\_only: 5 |
| KMIPKIT-SRC-spec | 6.1.28.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.29 | 2 | non\_applicable: 2 |
| KMIPKIT-SRC-spec | 6.1.29.1 | 2 | non\_applicable: 1, source\_discrepancy: 1 |
| KMIPKIT-SRC-spec | 6.1.30 | 2 | non\_applicable: 2 |
| KMIPKIT-SRC-spec | 6.1.30.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.31 | 2 | non\_applicable: 2 |
| KMIPKIT-SRC-spec | 6.1.31.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.32 | 5 | non\_applicable: 2, requirement: 3 |
| KMIPKIT-SRC-spec | 6.1.32.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.33 | 6 | non\_applicable: 2, requirement: 3, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.33.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.34 | 3 | non\_applicable: 2, requirement: 1 |
| KMIPKIT-SRC-spec | 6.1.34.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.35 | 5 | non\_applicable: 2, requirement: 3 |
| KMIPKIT-SRC-spec | 6.1.35.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.36 | 3 | non\_applicable: 2, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.37 | 4 | non\_applicable: 2, source\_discrepancy: 2 |
| KMIPKIT-SRC-spec | 6.1.37.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.38 | 5 | non\_applicable: 1, requirement: 1, server\_only: 3 |
| KMIPKIT-SRC-spec | 6.1.38.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.39 | 3 | non\_applicable: 2, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.39.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.40 | 29 | non\_applicable: 2, requirement: 2, server\_only: 25 |
| KMIPKIT-SRC-spec | 6.1.40.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.41 | 4 | non\_applicable: 2, server\_only: 2 |
| KMIPKIT-SRC-spec | 6.1.41.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.42 | 3 | non\_applicable: 2, requirement: 1 |
| KMIPKIT-SRC-spec | 6.1.42.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.43 | 11 | non\_applicable: 3, requirement: 6, server\_only: 2 |
| KMIPKIT-SRC-spec | 6.1.43.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.44 | 4 | non\_applicable: 2, requirement: 2 |
| KMIPKIT-SRC-spec | 6.1.44.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.45 | 9 | non\_applicable: 2, requirement: 5, server\_only: 2 |
| KMIPKIT-SRC-spec | 6.1.45.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.46 | 6 | non\_applicable: 2, requirement: 2, server\_only: 2 |
| KMIPKIT-SRC-spec | 6.1.46.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.47 | 6 | non\_applicable: 2, requirement: 2, server\_only: 2 |
| KMIPKIT-SRC-spec | 6.1.47.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.48 | 11 | non\_applicable: 2, requirement: 3, server\_only: 6 |
| KMIPKIT-SRC-spec | 6.1.48.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.49 | 2 | non\_applicable: 2 |
| KMIPKIT-SRC-spec | 6.1.49.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.50 | 3 | non\_applicable: 2, requirement: 1 |
| KMIPKIT-SRC-spec | 6.1.50.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.51 | 3 | non\_applicable: 2, requirement: 1 |
| KMIPKIT-SRC-spec | 6.1.51.1 | 2 | non\_applicable: 1, source\_discrepancy: 1 |
| KMIPKIT-SRC-spec | 6.1.52 | 2 | non\_applicable: 2 |
| KMIPKIT-SRC-spec | 6.1.52.1 | 2 | non\_applicable: 1, source\_discrepancy: 1 |
| KMIPKIT-SRC-spec | 6.1.53 | 2 | non\_applicable: 2 |
| KMIPKIT-SRC-spec | 6.1.53.1 | 2 | non\_applicable: 1, source\_discrepancy: 1 |
| KMIPKIT-SRC-spec | 6.1.54 | 2 | non\_applicable: 2 |
| KMIPKIT-SRC-spec | 6.1.54.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.55 | 6 | non\_applicable: 2, requirement: 3, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.55.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.56 | 9 | non\_applicable: 2, requirement: 4, server\_only: 3 |
| KMIPKIT-SRC-spec | 6.1.56.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.1.57 | 7 | non\_applicable: 2, requirement: 5 |
| KMIPKIT-SRC-spec | 6.1.57.1 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 6.2.1 | 6 | later\_1\_1: 4, non\_applicable: 2 |
| KMIPKIT-SRC-spec | 6.2.1.1 | 2 | later\_1\_1: 1, non\_applicable: 1 |
| KMIPKIT-SRC-spec | 6.2.2 | 3 | later\_1\_1: 2, non\_applicable: 1 |
| KMIPKIT-SRC-spec | 6.2.2.1 | 2 | later\_1\_1: 1, non\_applicable: 1 |
| KMIPKIT-SRC-spec | 6.2.3 | 6 | later\_1\_1: 5, non\_applicable: 1 |
| KMIPKIT-SRC-spec | 6.2.3.1 | 2 | later\_1\_1: 1, non\_applicable: 1 |
| KMIPKIT-SRC-spec | 6.2.4 | 23 | later\_1\_1: 21, non\_applicable: 2 |
| KMIPKIT-SRC-spec | 6.2.4.1 | 2 | later\_1\_1: 1, non\_applicable: 1 |
| KMIPKIT-SRC-spec | 6.2.5 | 2 | non\_applicable: 2 |
| KMIPKIT-SRC-spec | 6.2.5.1 | 2 | later\_1\_1: 1, non\_applicable: 1 |
| KMIPKIT-SRC-spec | 7.1 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-spec | 7.2 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-spec | 7.3 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-spec | 7.4 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-spec | 7.5 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-spec | 7.6 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-spec | 7.7 | 2 | non\_applicable: 1, requirement: 1 |
| KMIPKIT-SRC-spec | 7.8 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 7.11 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 7.12 | 6 | informative\_context: 1, non\_applicable: 1, requirement: 3, source\_discrepancy: 1 |
| KMIPKIT-SRC-spec | 7.13 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-spec | 7.18 | 7 | non\_applicable: 1, requirement: 5, server\_only: 1 |
| KMIPKIT-SRC-spec | 7.21 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 7.22 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-spec | 7.23 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 7.24 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 7.25 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 7.26 | 2 | non\_applicable: 1, profile\_conditional: 1 |
| KMIPKIT-SRC-spec | 7.27 | 2 | non\_applicable: 1, profile\_conditional: 1 |
| KMIPKIT-SRC-spec | 7.28 | 2 | non\_applicable: 1, profile\_conditional: 1 |
| KMIPKIT-SRC-spec | 7.29 | 2 | non\_applicable: 1, profile\_conditional: 1 |
| KMIPKIT-SRC-spec | 7.30 | 2 | non\_applicable: 1, profile\_conditional: 1 |
| KMIPKIT-SRC-spec | 7.31 | 2 | non\_applicable: 1, profile\_conditional: 1 |
| KMIPKIT-SRC-spec | 7.32 | 2 | non\_applicable: 1, profile\_conditional: 1 |
| KMIPKIT-SRC-spec | 7.33 | 4 | informative\_context: 1, non\_applicable: 1, requirement: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 7.34 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-spec | 7.35 | 2 | non\_applicable: 1, requirement: 1 |
| KMIPKIT-SRC-spec | 7.36 | 5 | non\_applicable: 2, requirement: 3 |
| KMIPKIT-SRC-spec | 7.37 | 4 | non\_applicable: 1, server\_only: 3 |
| KMIPKIT-SRC-spec | 7.39 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-spec | 7.40 | 2 | informative\_context: 1, non\_applicable: 1 |
| KMIPKIT-SRC-spec | 7.41 | 4 | non\_applicable: 1, server\_only: 3 |
| KMIPKIT-SRC-spec | 8 | 3 | requirement: 3 |
| KMIPKIT-SRC-spec | 8.1 | 2 | non\_applicable: 1, requirement: 1 |
| KMIPKIT-SRC-spec | 8.2 | 2 | non\_applicable: 1, requirement: 1 |
| KMIPKIT-SRC-spec | 8.3 | 4 | informative\_context: 1, non\_applicable: 1, requirement: 2 |
| KMIPKIT-SRC-spec | 8.4 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 8.5 | 2 | non\_applicable: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 8.6 | 5 | non\_applicable: 1, server\_only: 4 |
| KMIPKIT-SRC-spec | 9.1 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 9.2 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 9.3 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 9.4 | 2 | requirement: 1, source\_discrepancy: 1 |
| KMIPKIT-SRC-spec | 9.5 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 9.6 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 9.7 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 9.8 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 9.9 | 2 | requirement: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 9.10 | 1 | server\_only: 1 |
| KMIPKIT-SRC-spec | 9.11 | 12 | informative\_context: 1, non\_applicable: 7, profile\_conditional: 1, requirement: 3 |
| KMIPKIT-SRC-spec | 9.12 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 9.13 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 9.14 | 1 | non\_applicable: 1 |
| KMIPKIT-SRC-spec | 9.16 | 1 | source\_discrepancy: 1 |
| KMIPKIT-SRC-spec | 9.17 | 1 | server\_only: 1 |
| KMIPKIT-SRC-spec | 9.18 | 1 | server\_only: 1 |
| KMIPKIT-SRC-spec | 9.19 | 2 | requirement: 1, server\_only: 1 |
| KMIPKIT-SRC-spec | 9.20 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 9.21 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 10.1.2 | 2 | requirement: 2 |
| KMIPKIT-SRC-spec | 10.1.5 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 10.2 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-spec | 10.3 | 1 | server\_only: 1 |
| KMIPKIT-SRC-spec | 10.4 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 11 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 11.3 | 4 | informative\_context: 1, server\_only: 3 |
| KMIPKIT-SRC-spec | 11.5 | 2 | server\_only: 2 |
| KMIPKIT-SRC-spec | 11.20 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-spec | 11.28 | 1 | source\_discrepancy: 1 |
| KMIPKIT-SRC-spec | 11.45 | 2 | informative\_context: 2 |
| KMIPKIT-SRC-spec | 11.46 | 7 | informative\_context: 5, server\_only: 1, source\_discrepancy: 1 |
| KMIPKIT-SRC-spec | 11.49 | 1 | informative\_context: 1 |
| KMIPKIT-SRC-spec | 11.56 | 2 | informative\_context: 1, requirement: 1 |
| KMIPKIT-SRC-spec | 11.58 | 1 | source\_discrepancy: 1 |
| KMIPKIT-SRC-spec | 12 | 1 | requirement: 1 |
| KMIPKIT-SRC-spec | 12.1 | 3 | informative\_context: 2, source\_discrepancy: 1 |
| KMIPKIT-SRC-spec | 14.1 | 2 | profile\_conditional: 1, requirement: 1 |
| KMIPKIT-SRC-spec | 14.2 | 4 | informative\_context: 2, server\_only: 2 |

## Unassigned requirements

| Requirement | Strength | Scope | Source |
| --- | --- | --- | --- |
| KMIPKIT-REQ-PROF-3-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §3 |
| KMIPKIT-REQ-PROF-3-003 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §3 |
| KMIPKIT-REQ-PROF-3-004 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §3 |
| KMIPKIT-REQ-PROF-3-006 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §3 |
| KMIPKIT-REQ-PROF-3.1-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §3.1 |
| KMIPKIT-REQ-PROF-3.1.1-002 | recommended | profile\_conditional | KMIPKIT-SRC-profiles §3.1.1 |
| KMIPKIT-REQ-PROF-3.1.1-004 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §3.1.1 |
| KMIPKIT-REQ-PROF-3.1.1-005 | prohibited | profile\_conditional | KMIPKIT-SRC-profiles §3.1.1 |
| KMIPKIT-REQ-PROF-3.1.2-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §3.1.2 |
| KMIPKIT-REQ-PROF-3.1.2-005 | prohibited | profile\_conditional | KMIPKIT-SRC-profiles §3.1.2 |
| KMIPKIT-REQ-PROF-3.2-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §3.2 |
| KMIPKIT-REQ-PROF-3.2.1-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §3.2.1 |
| KMIPKIT-REQ-PROF-3.2.2-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §3.2.2 |
| KMIPKIT-REQ-PROF-3.2.3-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §3.2.3 |
| KMIPKIT-REQ-PROF-3.2.4-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §3.2.4 |
| KMIPKIT-REQ-PROF-4.1-001 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §4.1 |
| KMIPKIT-REQ-PROF-4.1-003 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §4.1 |
| KMIPKIT-REQ-PROF-4.1.2-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §4.1.2 |
| KMIPKIT-REQ-PROF-4.1.2-005 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §4.1.2 |
| KMIPKIT-REQ-PROF-4.1.2-007 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §4.1.2 |
| KMIPKIT-REQ-PROF-4.1.2-008 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §4.1.2 |
| KMIPKIT-REQ-PROF-4.1.2-009 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §4.1.2 |
| KMIPKIT-REQ-PROF-5.10.1-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.10.1 |
| KMIPKIT-REQ-PROF-5.10.1-002 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.10.1 |
| KMIPKIT-REQ-PROF-5.10.1-003 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.10.1 |
| KMIPKIT-REQ-PROF-5.11.1-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.11.1 |
| KMIPKIT-REQ-PROF-5.11.1-002 | discouraged | profile\_conditional | KMIPKIT-SRC-profiles §5.11.1 |
| KMIPKIT-REQ-PROF-5.11.1-003 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.11.1 |
| KMIPKIT-REQ-PROF-5.11.1-004 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.11.1 |
| KMIPKIT-REQ-PROF-5.12.2-002 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.12.2 |
| KMIPKIT-REQ-PROF-5.12.2-003 | recommended | profile\_conditional | KMIPKIT-SRC-profiles §5.12.2 |
| KMIPKIT-REQ-PROF-5.12.2-004-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.12.2 |
| KMIPKIT-REQ-PROF-5.12.2-004-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.12.2 |
| KMIPKIT-REQ-PROF-5.12.2-005 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.12.2 |
| KMIPKIT-REQ-PROF-5.12.2-007-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.12.2 |
| KMIPKIT-REQ-PROF-5.12.2-007-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.12.2 |
| KMIPKIT-REQ-PROF-5.12.3-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.12.3 |
| KMIPKIT-REQ-PROF-5.12.3-003 | prohibited | profile\_conditional | KMIPKIT-SRC-profiles §5.12.3 |
| KMIPKIT-REQ-PROF-5.12.4-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.12.4 |
| KMIPKIT-REQ-PROF-5.12.4-002 | recommended | profile\_conditional | KMIPKIT-SRC-profiles §5.12.4 |
| KMIPKIT-REQ-PROF-5.12.4-003 | discouraged | profile\_conditional | KMIPKIT-SRC-profiles §5.12.4 |
| KMIPKIT-REQ-PROF-5.12.4-004 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.12.4 |
| KMIPKIT-REQ-PROF-5.12.4-005 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.12.4 |
| KMIPKIT-REQ-PROF-5.12.4-006 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.12.4 |
| KMIPKIT-REQ-PROF-5.12.4-007 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.12.4 |
| KMIPKIT-REQ-PROF-5.12.4-008 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.12.4 |
| KMIPKIT-REQ-PROF-5.12.4-009 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.12.4 |
| KMIPKIT-REQ-PROF-5.12.6.1-001 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.12.6.1 |
| KMIPKIT-REQ-PROF-5.12.6.2-002 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.12.6.2 |
| KMIPKIT-REQ-PROF-5.12.6.2-003 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.12.6.2 |
| KMIPKIT-REQ-PROF-5.12.6.2-004 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.12.6.2 |
| KMIPKIT-REQ-PROF-5.12.6.3-001 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.12.6.3 |
| KMIPKIT-REQ-PROF-5.12.6.3-002 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.12.6.3 |
| KMIPKIT-REQ-PROF-5.12.6.3-003 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.12.6.3 |
| KMIPKIT-REQ-PROF-5.12.6.3-004 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.12.6.3 |
| KMIPKIT-REQ-PROF-5.13.1-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.13.1 |
| KMIPKIT-REQ-PROF-5.13.1-003 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.13.1 |
| KMIPKIT-REQ-PROF-5.15-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.15 |
| KMIPKIT-REQ-PROF-5.15-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.15 |
| KMIPKIT-REQ-PROF-5.15-003 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.15 |
| KMIPKIT-REQ-PROF-5.15-004 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.15 |
| KMIPKIT-REQ-PROF-5.17-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.17 |
| KMIPKIT-REQ-PROF-5.17.1-001 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.17.1 |
| KMIPKIT-REQ-PROF-5.17.1-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.17.1 |
| KMIPKIT-REQ-PROF-5.17.2-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.17.2 |
| KMIPKIT-REQ-PROF-5.18.1-004 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.18.1 |
| KMIPKIT-REQ-PROF-5.18.1-005 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.18.1 |
| KMIPKIT-REQ-PROF-5.18.3-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.18.3 |
| KMIPKIT-REQ-PROF-5.18.3-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.18.3 |
| KMIPKIT-REQ-PROF-5.18.3-003 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.18.3 |
| KMIPKIT-REQ-PROF-5.18.3-005 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.18.3 |
| KMIPKIT-REQ-PROF-5.3.1-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.3.1 |
| KMIPKIT-REQ-PROF-5.3.1-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.3.1 |
| KMIPKIT-REQ-PROF-5.3.1-003 | recommended | profile\_conditional | KMIPKIT-SRC-profiles §5.3.1 |
| KMIPKIT-REQ-PROF-5.3.1-004 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.3.1 |
| KMIPKIT-REQ-PROF-5.3.1-005 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.3.1 |
| KMIPKIT-REQ-PROF-5.3.1-008 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.3.1 |
| KMIPKIT-REQ-PROF-5.3.1-009 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.3.1 |
| KMIPKIT-REQ-PROF-5.3.1-010 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.3.1 |
| KMIPKIT-REQ-PROF-5.6.1-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.6.1 |
| KMIPKIT-REQ-PROF-5.6.1-002 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.6.1 |
| KMIPKIT-REQ-PROF-5.6.1-003 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.6.1 |
| KMIPKIT-REQ-PROF-5.7.1-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.7.1 |
| KMIPKIT-REQ-PROF-5.7.1-002 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.7.1 |
| KMIPKIT-REQ-PROF-5.7.1-003 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.7.1 |
| KMIPKIT-REQ-PROF-5.7.2-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.7.2 |
| KMIPKIT-REQ-PROF-5.7.2-002 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.7.2 |
| KMIPKIT-REQ-PROF-5.7.2-003 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.7.2 |
| KMIPKIT-REQ-PROF-5.7.3-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.7.3 |
| KMIPKIT-REQ-PROF-5.7.3-002 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.7.3 |
| KMIPKIT-REQ-PROF-5.7.3-003 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.7.3 |
| KMIPKIT-REQ-PROF-5.8.1-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.8.1 |
| KMIPKIT-REQ-PROF-5.8.1-002 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.8.1 |
| KMIPKIT-REQ-PROF-5.8.1-003 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.8.1 |
| KMIPKIT-REQ-PROF-5.9.1-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.9.1 |
| KMIPKIT-REQ-PROF-5.9.1-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.9.1 |
| KMIPKIT-REQ-PROF-5.9.1-003 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.9.1 |
| KMIPKIT-REQ-PROF-5.9.1-004 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.9.1 |
| KMIPKIT-REQ-PROF-5.9.2-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.9.2 |
| KMIPKIT-REQ-PROF-5.9.2-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.9.2 |
| KMIPKIT-REQ-PROF-5.9.2-003 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.9.2 |
| KMIPKIT-REQ-PROF-5.9.2-004 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.9.2 |
| KMIPKIT-REQ-PROF-5.9.3-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.9.3 |
| KMIPKIT-REQ-PROF-5.9.3-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §5.9.3 |
| KMIPKIT-REQ-PROF-5.9.3-003 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.9.3 |
| KMIPKIT-REQ-PROF-5.9.3-004 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-profiles §5.9.3 |
| KMIPKIT-REQ-PROF-6-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6 |
| KMIPKIT-REQ-PROF-6.1-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.1 |
| KMIPKIT-REQ-PROF-6.1-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.1 |
| KMIPKIT-REQ-PROF-6.1-003 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.1 |
| KMIPKIT-REQ-PROF-6.1-004 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.1 |
| KMIPKIT-REQ-PROF-6.10-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.10 |
| KMIPKIT-REQ-PROF-6.10-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.10 |
| KMIPKIT-REQ-PROF-6.10-003 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.10 |
| KMIPKIT-REQ-PROF-6.10-004 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.10 |
| KMIPKIT-REQ-PROF-6.10-005 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.10 |
| KMIPKIT-REQ-PROF-6.12-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.12 |
| KMIPKIT-REQ-PROF-6.12-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.12 |
| KMIPKIT-REQ-PROF-6.12-003 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.12 |
| KMIPKIT-REQ-PROF-6.12-004 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.12 |
| KMIPKIT-REQ-PROF-6.12-005 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.12 |
| KMIPKIT-REQ-PROF-6.13-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.13 |
| KMIPKIT-REQ-PROF-6.13-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.13 |
| KMIPKIT-REQ-PROF-6.13-004 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.13 |
| KMIPKIT-REQ-PROF-6.13-005 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.13 |
| KMIPKIT-REQ-PROF-6.14-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.14 |
| KMIPKIT-REQ-PROF-6.14-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.14 |
| KMIPKIT-REQ-PROF-6.14-004 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.14 |
| KMIPKIT-REQ-PROF-6.14-005 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.14 |
| KMIPKIT-REQ-PROF-6.16-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.16 |
| KMIPKIT-REQ-PROF-6.16-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.16 |
| KMIPKIT-REQ-PROF-6.16-003 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.16 |
| KMIPKIT-REQ-PROF-6.16-004 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.16 |
| KMIPKIT-REQ-PROF-6.16-005 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.16 |
| KMIPKIT-REQ-PROF-6.18-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.18 |
| KMIPKIT-REQ-PROF-6.18-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.18 |
| KMIPKIT-REQ-PROF-6.18-003 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.18 |
| KMIPKIT-REQ-PROF-6.18-004 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.18 |
| KMIPKIT-REQ-PROF-6.18-005 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.18 |
| KMIPKIT-REQ-PROF-6.19-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.19 |
| KMIPKIT-REQ-PROF-6.19-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.19 |
| KMIPKIT-REQ-PROF-6.19-003 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.19 |
| KMIPKIT-REQ-PROF-6.19-004 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.19 |
| KMIPKIT-REQ-PROF-6.19-005 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.19 |
| KMIPKIT-REQ-PROF-6.20-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.20 |
| KMIPKIT-REQ-PROF-6.20-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.20 |
| KMIPKIT-REQ-PROF-6.20-003 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.20 |
| KMIPKIT-REQ-PROF-6.20-004 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.20 |
| KMIPKIT-REQ-PROF-6.20-005 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.20 |
| KMIPKIT-REQ-PROF-6.24-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.24 |
| KMIPKIT-REQ-PROF-6.24-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.24 |
| KMIPKIT-REQ-PROF-6.24-003 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.24 |
| KMIPKIT-REQ-PROF-6.24-004 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.24 |
| KMIPKIT-REQ-PROF-6.24-005 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.24 |
| KMIPKIT-REQ-PROF-6.26-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.26 |
| KMIPKIT-REQ-PROF-6.26-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.26 |
| KMIPKIT-REQ-PROF-6.26-003 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.26 |
| KMIPKIT-REQ-PROF-6.26-004 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.26 |
| KMIPKIT-REQ-PROF-6.26-005 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.26 |
| KMIPKIT-REQ-PROF-6.28-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.28 |
| KMIPKIT-REQ-PROF-6.28-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.28 |
| KMIPKIT-REQ-PROF-6.28-003 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.28 |
| KMIPKIT-REQ-PROF-6.28-004 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.28 |
| KMIPKIT-REQ-PROF-6.28-005 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.28 |
| KMIPKIT-REQ-PROF-6.28-006 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.28 |
| KMIPKIT-REQ-PROF-6.28-007 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.28 |
| KMIPKIT-REQ-PROF-6.30-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.30 |
| KMIPKIT-REQ-PROF-6.30-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.30 |
| KMIPKIT-REQ-PROF-6.30-003 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.30 |
| KMIPKIT-REQ-PROF-6.30-004 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.30 |
| KMIPKIT-REQ-PROF-6.30-005 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.30 |
| KMIPKIT-REQ-PROF-6.32-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.32 |
| KMIPKIT-REQ-PROF-6.32-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.32 |
| KMIPKIT-REQ-PROF-6.32-003 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.32 |
| KMIPKIT-REQ-PROF-6.32-004 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.32 |
| KMIPKIT-REQ-PROF-6.32-005 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.32 |
| KMIPKIT-REQ-PROF-6.34-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.34 |
| KMIPKIT-REQ-PROF-6.34-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.34 |
| KMIPKIT-REQ-PROF-6.34-003 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.34 |
| KMIPKIT-REQ-PROF-6.34-004 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.34 |
| KMIPKIT-REQ-PROF-6.34-005 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.34 |
| KMIPKIT-REQ-PROF-6.4-001 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.4 |
| KMIPKIT-REQ-PROF-6.4-002 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.4 |
| KMIPKIT-REQ-PROF-6.4-003 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.4 |
| KMIPKIT-REQ-PROF-6.4-004 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.4 |
| KMIPKIT-REQ-PROF-6.4-005 | mandatory | profile\_conditional | KMIPKIT-SRC-profiles §6.4 |
| KMIPKIT-REQ-SPEC-10.1.2-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §10.1.2 |
| KMIPKIT-REQ-SPEC-10.4-001-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §10.4 |
| KMIPKIT-REQ-SPEC-10.4-001-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §10.4 |
| KMIPKIT-REQ-SPEC-10.4-001-003 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §10.4 |
| KMIPKIT-REQ-SPEC-12-001-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §12 |
| KMIPKIT-REQ-SPEC-12-001-002 | mandatory | out\_of\_scope | KMIPKIT-SRC-spec §12 |
| KMIPKIT-REQ-SPEC-14.1-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §14.1 |
| KMIPKIT-REQ-SPEC-14.1-002 | mandatory | profile\_conditional | KMIPKIT-SRC-spec §14.1 |
| KMIPKIT-REQ-SPEC-2.1-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §2.1 |
| KMIPKIT-REQ-SPEC-2.2-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §2.2 |
| KMIPKIT-REQ-SPEC-2.3-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §2.3 |
| KMIPKIT-REQ-SPEC-2.3-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §2.3 |
| KMIPKIT-REQ-SPEC-2.3-003 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §2.3 |
| KMIPKIT-REQ-SPEC-2.4-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §2.4 |
| KMIPKIT-REQ-SPEC-2.4-001-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §2.4 |
| KMIPKIT-REQ-SPEC-2.4-002-001 | recommended | client\_1\_0 | KMIPKIT-SRC-spec §2.4 |
| KMIPKIT-REQ-SPEC-2.4-002-002 | recommended | client\_1\_0 | KMIPKIT-SRC-spec §2.4 |
| KMIPKIT-REQ-SPEC-2.4-003 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §2.4 |
| KMIPKIT-REQ-SPEC-2.5-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §2.5 |
| KMIPKIT-REQ-SPEC-2.6-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §2.6 |
| KMIPKIT-REQ-SPEC-2.7-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §2.7 |
| KMIPKIT-REQ-SPEC-2.7-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §2.7 |
| KMIPKIT-REQ-SPEC-2.8-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §2.8 |
| KMIPKIT-REQ-SPEC-2.8-001-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §2.8 |
| KMIPKIT-REQ-SPEC-2.8-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §2.8 |
| KMIPKIT-REQ-SPEC-2.8-003 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §2.8 |
| KMIPKIT-REQ-SPEC-2.9-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §2.9 |
| KMIPKIT-REQ-SPEC-3.1-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-REQ-SPEC-3.1-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-REQ-SPEC-3.1-003 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-REQ-SPEC-3.1-004-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-REQ-SPEC-3.1-004-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-REQ-SPEC-3.1-005-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-REQ-SPEC-3.1-005-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-REQ-SPEC-3.1-006 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-REQ-SPEC-3.10-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §3.10 |
| KMIPKIT-REQ-SPEC-3.11-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §3.11 |
| KMIPKIT-REQ-SPEC-3.12-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §3.12 |
| KMIPKIT-REQ-SPEC-3.2-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §3.2 |
| KMIPKIT-REQ-SPEC-3.2-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §3.2 |
| KMIPKIT-REQ-SPEC-3.3-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-REQ-SPEC-3.3-001-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-REQ-SPEC-3.3-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-REQ-SPEC-3.3-003 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-REQ-SPEC-3.3-004 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-REQ-SPEC-3.3-005 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-REQ-SPEC-3.3-006 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-REQ-SPEC-3.3-007 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-REQ-SPEC-3.3-008 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-REQ-SPEC-3.3-009 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-REQ-SPEC-3.4-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §3.4 |
| KMIPKIT-REQ-SPEC-3.5-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §3.5 |
| KMIPKIT-REQ-SPEC-3.6-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §3.6 |
| KMIPKIT-REQ-SPEC-3.7-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §3.7 |
| KMIPKIT-REQ-SPEC-3.7-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §3.7 |
| KMIPKIT-REQ-SPEC-3.8-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §3.8 |
| KMIPKIT-REQ-SPEC-3.9-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §3.9 |
| KMIPKIT-REQ-SPEC-4-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4 |
| KMIPKIT-REQ-SPEC-4-001-002 | recommended | client\_1\_0 | KMIPKIT-SRC-spec §4 |
| KMIPKIT-REQ-SPEC-4-001-003 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §4 |
| KMIPKIT-REQ-SPEC-4-001-004 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4 |
| KMIPKIT-REQ-SPEC-4-001-005 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4 |
| KMIPKIT-REQ-SPEC-4.1-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4.1 |
| KMIPKIT-REQ-SPEC-4.1-001-002 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4.1 |
| KMIPKIT-REQ-SPEC-4.1-001-003 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4.1 |
| KMIPKIT-REQ-SPEC-4.11-001 | recommended | client\_1\_0 | KMIPKIT-SRC-spec §4.11 |
| KMIPKIT-REQ-SPEC-4.14-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4.14 |
| KMIPKIT-REQ-SPEC-4.14-001-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4.14 |
| KMIPKIT-REQ-SPEC-4.16-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-REQ-SPEC-4.16-001-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-REQ-SPEC-4.16-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-REQ-SPEC-4.16-003 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-REQ-SPEC-4.17-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4.17 |
| KMIPKIT-REQ-SPEC-4.17-001-002 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4.17 |
| KMIPKIT-REQ-SPEC-4.18-001-001 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4.18 |
| KMIPKIT-REQ-SPEC-4.18-001-002 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4.18 |
| KMIPKIT-REQ-SPEC-4.2-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §4.2 |
| KMIPKIT-REQ-SPEC-4.21-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §4.21 |
| KMIPKIT-REQ-SPEC-4.26-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4.26 |
| KMIPKIT-REQ-SPEC-4.27-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4.27 |
| KMIPKIT-REQ-SPEC-4.27-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §4.27 |
| KMIPKIT-REQ-SPEC-4.28-001-001 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4.28 |
| KMIPKIT-REQ-SPEC-4.28-001-002 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4.28 |
| KMIPKIT-REQ-SPEC-4.28-001-003 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4.28 |
| KMIPKIT-REQ-SPEC-4.28-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §4.28 |
| KMIPKIT-REQ-SPEC-4.30-001 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4.30 |
| KMIPKIT-REQ-SPEC-4.31-003 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §4.31 |
| KMIPKIT-REQ-SPEC-4.32-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §4.32 |
| KMIPKIT-REQ-SPEC-4.32-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §4.32 |
| KMIPKIT-REQ-SPEC-4.34-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4.34 |
| KMIPKIT-REQ-SPEC-4.34-001-002 | recommended | client\_1\_0 | KMIPKIT-SRC-spec §4.34 |
| KMIPKIT-REQ-SPEC-4.35-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4.35 |
| KMIPKIT-REQ-SPEC-4.35-001-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4.35 |
| KMIPKIT-REQ-SPEC-4.35-001-003 | recommended | client\_1\_0 | KMIPKIT-SRC-spec §4.35 |
| KMIPKIT-REQ-SPEC-4.38-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4.38 |
| KMIPKIT-REQ-SPEC-4.38-001-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4.38 |
| KMIPKIT-REQ-SPEC-4.38-001-003 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4.38 |
| KMIPKIT-REQ-SPEC-4.38-003 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4.38 |
| KMIPKIT-REQ-SPEC-4.4-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4.4 |
| KMIPKIT-REQ-SPEC-4.4-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §4.4 |
| KMIPKIT-REQ-SPEC-4.40-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4.40 |
| KMIPKIT-REQ-SPEC-4.40-001-002 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4.40 |
| KMIPKIT-REQ-SPEC-4.40-001-003 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4.40 |
| KMIPKIT-REQ-SPEC-4.40-001-004 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4.40 |
| KMIPKIT-REQ-SPEC-4.40-001-005 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4.40 |
| KMIPKIT-REQ-SPEC-4.41-001-001 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4.41 |
| KMIPKIT-REQ-SPEC-4.41-001-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4.41 |
| KMIPKIT-REQ-SPEC-4.41-001-003 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4.41 |
| KMIPKIT-REQ-SPEC-4.41-001-004 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4.41 |
| KMIPKIT-REQ-SPEC-4.46-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4.46 |
| KMIPKIT-REQ-SPEC-4.46-001-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4.46 |
| KMIPKIT-REQ-SPEC-4.47-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4.47 |
| KMIPKIT-REQ-SPEC-4.47-001-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4.47 |
| KMIPKIT-REQ-SPEC-4.47-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §4.47 |
| KMIPKIT-REQ-SPEC-4.57-001 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-002 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-003-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-003-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-003-003 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-004-001 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-004-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-004-003 | recommended | client\_1\_0 | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-005-001 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-005-002 | recommended | client\_1\_0 | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-005-003 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-006 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-007 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.59-002 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4.59 |
| KMIPKIT-REQ-SPEC-4.59-003-001 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4.59 |
| KMIPKIT-REQ-SPEC-4.59-003-002 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §4.59 |
| KMIPKIT-REQ-SPEC-4.60-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §4.60 |
| KMIPKIT-REQ-SPEC-4.61-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §4.61 |
| KMIPKIT-REQ-SPEC-4.62-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §4.62 |
| KMIPKIT-REQ-SPEC-4.62-003 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4.62 |
| KMIPKIT-REQ-SPEC-4.63-003 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §4.63 |
| KMIPKIT-REQ-SPEC-4.63-004 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4.63 |
| KMIPKIT-REQ-SPEC-4.63-005 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §4.63 |
| KMIPKIT-REQ-SPEC-5.1-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §5.1 |
| KMIPKIT-REQ-SPEC-5.1-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §5.1 |
| KMIPKIT-REQ-SPEC-5.2-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §5.2 |
| KMIPKIT-REQ-SPEC-5.2-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §5.2 |
| KMIPKIT-REQ-SPEC-5.3-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §5.3 |
| KMIPKIT-REQ-SPEC-5.3-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §5.3 |
| KMIPKIT-REQ-SPEC-5.4-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §5.4 |
| KMIPKIT-REQ-SPEC-5.4-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §5.4 |
| KMIPKIT-REQ-SPEC-5.5-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §5.5 |
| KMIPKIT-REQ-SPEC-5.5-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §5.5 |
| KMIPKIT-REQ-SPEC-5.6-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §5.6 |
| KMIPKIT-REQ-SPEC-5.7-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §5.7 |
| KMIPKIT-REQ-SPEC-6.1-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1 |
| KMIPKIT-REQ-SPEC-6.1-001-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1 |
| KMIPKIT-REQ-SPEC-6.1-001-003 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1 |
| KMIPKIT-REQ-SPEC-6.1-003-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1 |
| KMIPKIT-REQ-SPEC-6.1-003-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1 |
| KMIPKIT-REQ-SPEC-6.1-004 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1 |
| KMIPKIT-REQ-SPEC-6.1-005 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1 |
| KMIPKIT-REQ-SPEC-6.1.10-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.10 |
| KMIPKIT-REQ-SPEC-6.1.11-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.11 |
| KMIPKIT-REQ-SPEC-6.1.11-001-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.11 |
| KMIPKIT-REQ-SPEC-6.1.11-003 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.11 |
| KMIPKIT-REQ-SPEC-6.1.11-004-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.11 |
| KMIPKIT-REQ-SPEC-6.1.11-004-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.11 |
| KMIPKIT-REQ-SPEC-6.1.11-005 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.11 |
| KMIPKIT-REQ-SPEC-6.1.11-006 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.11 |
| KMIPKIT-REQ-SPEC-6.1.13-001-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.13 |
| KMIPKIT-REQ-SPEC-6.1.13-001-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.13 |
| KMIPKIT-REQ-SPEC-6.1.14-001-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.14 |
| KMIPKIT-REQ-SPEC-6.1.14-001-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.14 |
| KMIPKIT-REQ-SPEC-6.1.14-001-003 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.14 |
| KMIPKIT-REQ-SPEC-6.1.14-001-004 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.14 |
| KMIPKIT-REQ-SPEC-6.1.14-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.14 |
| KMIPKIT-REQ-SPEC-6.1.14-006-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.14 |
| KMIPKIT-REQ-SPEC-6.1.14-006-002 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §6.1.14 |
| KMIPKIT-REQ-SPEC-6.1.14-007-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.14 |
| KMIPKIT-REQ-SPEC-6.1.14-007-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.14 |
| KMIPKIT-REQ-SPEC-6.1.16-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.16 |
| KMIPKIT-REQ-SPEC-6.1.16-001-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.16 |
| KMIPKIT-REQ-SPEC-6.1.16-004 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.16 |
| KMIPKIT-REQ-SPEC-6.1.17-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.17 |
| KMIPKIT-REQ-SPEC-6.1.17-001-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.17 |
| KMIPKIT-REQ-SPEC-6.1.17-004 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.17 |
| KMIPKIT-REQ-SPEC-6.1.17-005-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.17 |
| KMIPKIT-REQ-SPEC-6.1.17-005-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.17 |
| KMIPKIT-REQ-SPEC-6.1.17-006 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.17 |
| KMIPKIT-REQ-SPEC-6.1.19-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.19 |
| KMIPKIT-REQ-SPEC-6.1.19-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.19 |
| KMIPKIT-REQ-SPEC-6.1.2-001-001 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §6.1.2 |
| KMIPKIT-REQ-SPEC-6.1.2-001-002 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §6.1.2 |
| KMIPKIT-REQ-SPEC-6.1.20-001-001 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §6.1.20 |
| KMIPKIT-REQ-SPEC-6.1.20-001-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.20 |
| KMIPKIT-REQ-SPEC-6.1.20-004 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.20 |
| KMIPKIT-REQ-SPEC-6.1.23-001-001 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §6.1.23 |
| KMIPKIT-REQ-SPEC-6.1.23-001-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.23 |
| KMIPKIT-REQ-SPEC-6.1.23-002-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.23 |
| KMIPKIT-REQ-SPEC-6.1.23-002-002 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §6.1.23 |
| KMIPKIT-REQ-SPEC-6.1.25-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.25 |
| KMIPKIT-REQ-SPEC-6.1.25-004 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.25 |
| KMIPKIT-REQ-SPEC-6.1.25-006 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.25 |
| KMIPKIT-REQ-SPEC-6.1.27-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.27 |
| KMIPKIT-REQ-SPEC-6.1.27-002-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.27 |
| KMIPKIT-REQ-SPEC-6.1.27-002-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.27 |
| KMIPKIT-REQ-SPEC-6.1.27-005 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.27 |
| KMIPKIT-REQ-SPEC-6.1.28-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.28-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.28-004-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.28-004-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.28-007 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.28-008-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.28-008-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.28-009-001 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.28-009-002 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.28-011 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.28-012 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.28-013-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.28-013-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.3-001 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §6.1.3 |
| KMIPKIT-REQ-SPEC-6.1.32-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.32 |
| KMIPKIT-REQ-SPEC-6.1.32-003 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.32 |
| KMIPKIT-REQ-SPEC-6.1.32-004-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.32 |
| KMIPKIT-REQ-SPEC-6.1.32-004-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.32 |
| KMIPKIT-REQ-SPEC-6.1.33-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.33 |
| KMIPKIT-REQ-SPEC-6.1.33-001-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.33 |
| KMIPKIT-REQ-SPEC-6.1.33-004 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.33 |
| KMIPKIT-REQ-SPEC-6.1.33-005-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.33 |
| KMIPKIT-REQ-SPEC-6.1.33-005-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.33 |
| KMIPKIT-REQ-SPEC-6.1.34-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.34 |
| KMIPKIT-REQ-SPEC-6.1.34-001-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.34 |
| KMIPKIT-REQ-SPEC-6.1.34-001-003 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.34 |
| KMIPKIT-REQ-SPEC-6.1.34-001-004 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.34 |
| KMIPKIT-REQ-SPEC-6.1.35-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.35 |
| KMIPKIT-REQ-SPEC-6.1.35-001-002 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §6.1.35 |
| KMIPKIT-REQ-SPEC-6.1.35-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.35 |
| KMIPKIT-REQ-SPEC-6.1.35-005 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.35 |
| KMIPKIT-REQ-SPEC-6.1.38-001-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.38 |
| KMIPKIT-REQ-SPEC-6.1.38-001-002 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §6.1.38 |
| KMIPKIT-REQ-SPEC-6.1.4-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.4 |
| KMIPKIT-REQ-SPEC-6.1.40-014 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.40 |
| KMIPKIT-REQ-SPEC-6.1.40-016 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.40 |
| KMIPKIT-REQ-SPEC-6.1.42-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.42 |
| KMIPKIT-REQ-SPEC-6.1.42-001-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.42 |
| KMIPKIT-REQ-SPEC-6.1.43-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.43 |
| KMIPKIT-REQ-SPEC-6.1.43-005 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.43 |
| KMIPKIT-REQ-SPEC-6.1.43-007 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.43 |
| KMIPKIT-REQ-SPEC-6.1.43-009-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.43 |
| KMIPKIT-REQ-SPEC-6.1.43-009-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.43 |
| KMIPKIT-REQ-SPEC-6.1.43-010-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.43 |
| KMIPKIT-REQ-SPEC-6.1.43-010-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.43 |
| KMIPKIT-REQ-SPEC-6.1.43-011 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.43 |
| KMIPKIT-REQ-SPEC-6.1.44-001 | recommended | client\_1\_0 | KMIPKIT-SRC-spec §6.1.44 |
| KMIPKIT-REQ-SPEC-6.1.44-003-001 | recommended | client\_1\_0 | KMIPKIT-SRC-spec §6.1.44 |
| KMIPKIT-REQ-SPEC-6.1.44-003-002 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §6.1.44 |
| KMIPKIT-REQ-SPEC-6.1.45-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.45 |
| KMIPKIT-REQ-SPEC-6.1.45-002-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.45 |
| KMIPKIT-REQ-SPEC-6.1.45-002-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.45 |
| KMIPKIT-REQ-SPEC-6.1.45-004 | recommended | client\_1\_0 | KMIPKIT-SRC-spec §6.1.45 |
| KMIPKIT-REQ-SPEC-6.1.45-006-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.45 |
| KMIPKIT-REQ-SPEC-6.1.45-006-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.45 |
| KMIPKIT-REQ-SPEC-6.1.45-008 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.45 |
| KMIPKIT-REQ-SPEC-6.1.46-001 | recommended | client\_1\_0 | KMIPKIT-SRC-spec §6.1.46 |
| KMIPKIT-REQ-SPEC-6.1.46-004-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.46 |
| KMIPKIT-REQ-SPEC-6.1.46-004-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.46 |
| KMIPKIT-REQ-SPEC-6.1.47-001 | recommended | client\_1\_0 | KMIPKIT-SRC-spec §6.1.47 |
| KMIPKIT-REQ-SPEC-6.1.47-004-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.47 |
| KMIPKIT-REQ-SPEC-6.1.47-004-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.47 |
| KMIPKIT-REQ-SPEC-6.1.48-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.48 |
| KMIPKIT-REQ-SPEC-6.1.48-004 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.48 |
| KMIPKIT-REQ-SPEC-6.1.48-006 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.48 |
| KMIPKIT-REQ-SPEC-6.1.5-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.5 |
| KMIPKIT-REQ-SPEC-6.1.50-001 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §6.1.50 |
| KMIPKIT-REQ-SPEC-6.1.51-001 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §6.1.51 |
| KMIPKIT-REQ-SPEC-6.1.55-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.55 |
| KMIPKIT-REQ-SPEC-6.1.55-004 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.55 |
| KMIPKIT-REQ-SPEC-6.1.55-005-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.55 |
| KMIPKIT-REQ-SPEC-6.1.55-005-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.55 |
| KMIPKIT-REQ-SPEC-6.1.56-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.56 |
| KMIPKIT-REQ-SPEC-6.1.56-001-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.56 |
| KMIPKIT-REQ-SPEC-6.1.56-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.56 |
| KMIPKIT-REQ-SPEC-6.1.56-006 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.56 |
| KMIPKIT-REQ-SPEC-6.1.56-007-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.56 |
| KMIPKIT-REQ-SPEC-6.1.56-007-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.56 |
| KMIPKIT-REQ-SPEC-6.1.57-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.57 |
| KMIPKIT-REQ-SPEC-6.1.57-002-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.57 |
| KMIPKIT-REQ-SPEC-6.1.57-002-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.57 |
| KMIPKIT-REQ-SPEC-6.1.57-004 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.57 |
| KMIPKIT-REQ-SPEC-6.1.57-005 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.57 |
| KMIPKIT-REQ-SPEC-6.1.57-006 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.57 |
| KMIPKIT-REQ-SPEC-6.1.6-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.6 |
| KMIPKIT-REQ-SPEC-6.1.6-002-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.6 |
| KMIPKIT-REQ-SPEC-6.1.6-002-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.6 |
| KMIPKIT-REQ-SPEC-6.1.6-002-003 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.6 |
| KMIPKIT-REQ-SPEC-6.1.6-003 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.6 |
| KMIPKIT-REQ-SPEC-6.1.6-007 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.6 |
| KMIPKIT-REQ-SPEC-6.1.7-001 | recommended | client\_1\_0 | KMIPKIT-SRC-spec §6.1.7 |
| KMIPKIT-REQ-SPEC-6.1.7-002-001 | recommended | client\_1\_0 | KMIPKIT-SRC-spec §6.1.7 |
| KMIPKIT-REQ-SPEC-6.1.7-002-002 | recommended | client\_1\_0 | KMIPKIT-SRC-spec §6.1.7 |
| KMIPKIT-REQ-SPEC-6.1.7-003 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.7 |
| KMIPKIT-REQ-SPEC-6.1.7-004-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.7 |
| KMIPKIT-REQ-SPEC-6.1.7-004-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.7 |
| KMIPKIT-REQ-SPEC-6.1.7-005 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.7 |
| KMIPKIT-REQ-SPEC-6.1.9-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.9 |
| KMIPKIT-REQ-SPEC-6.1.9-001-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §6.1.9 |
| KMIPKIT-REQ-SPEC-6.1.9-006 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.9 |
| KMIPKIT-REQ-SPEC-6.1.9-007 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §6.1.9 |
| KMIPKIT-REQ-SPEC-7.12-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §7.12 |
| KMIPKIT-REQ-SPEC-7.12-003 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §7.12 |
| KMIPKIT-REQ-SPEC-7.12-005-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §7.12 |
| KMIPKIT-REQ-SPEC-7.12-005-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §7.12 |
| KMIPKIT-REQ-SPEC-7.18-001-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §7.18 |
| KMIPKIT-REQ-SPEC-7.18-001-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §7.18 |
| KMIPKIT-REQ-SPEC-7.18-002 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §7.18 |
| KMIPKIT-REQ-SPEC-7.18-004 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §7.18 |
| KMIPKIT-REQ-SPEC-7.18-005 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §7.18 |
| KMIPKIT-REQ-SPEC-7.18-006 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §7.18 |
| KMIPKIT-REQ-SPEC-7.26-001 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-spec §7.26 |
| KMIPKIT-REQ-SPEC-7.27-001 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-spec §7.27 |
| KMIPKIT-REQ-SPEC-7.28-001 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-spec §7.28 |
| KMIPKIT-REQ-SPEC-7.29-001 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-spec §7.29 |
| KMIPKIT-REQ-SPEC-7.30-001 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-spec §7.30 |
| KMIPKIT-REQ-SPEC-7.31-001 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-spec §7.31 |
| KMIPKIT-REQ-SPEC-7.32-001 | mandatory | profile\_conditional | KMIPKIT-SRC-spec §7.32 |
| KMIPKIT-REQ-SPEC-7.33-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §7.33 |
| KMIPKIT-REQ-SPEC-7.35-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §7.35 |
| KMIPKIT-REQ-SPEC-7.36-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §7.36 |
| KMIPKIT-REQ-SPEC-7.36-002-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §7.36 |
| KMIPKIT-REQ-SPEC-7.36-002-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §7.36 |
| KMIPKIT-REQ-SPEC-7.36-003 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §7.36 |
| KMIPKIT-REQ-SPEC-7.7-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §7.7 |
| KMIPKIT-REQ-SPEC-7.8-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §7.8 |
| KMIPKIT-REQ-SPEC-9.1-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.1 |
| KMIPKIT-REQ-SPEC-9.11-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-REQ-SPEC-9.11-004-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-REQ-SPEC-9.11-004-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-REQ-SPEC-9.11-004-003 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-REQ-SPEC-9.11-006 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-REQ-SPEC-9.11-010-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-REQ-SPEC-9.11-010-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-REQ-SPEC-9.19-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.19 |
| KMIPKIT-REQ-SPEC-9.20-001-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §9.20 |
| KMIPKIT-REQ-SPEC-9.3-001-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.3 |
| KMIPKIT-REQ-SPEC-9.4-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §9.4 |
| KMIPKIT-REQ-SPEC-9.4-001-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.4 |
| KMIPKIT-REQ-SPEC-9.4-001-003 | mandatory | server\_only | KMIPKIT-SRC-spec §9.4 |
| KMIPKIT-REQ-SPEC-9.4-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §9.4 |
| KMIPKIT-REQ-SPEC-9.8-001-002 | mandatory | server\_only | KMIPKIT-SRC-spec §9.8 |
| KMIPKIT-REQ-SPEC-9.8-001-003 | permission\_or\_optional | server\_only | KMIPKIT-SRC-spec §9.8 |

## Requirements without official test-case links

| Requirement | Evidence gap | Source |
| --- | --- | --- |
| KMIPKIT-REQ-PROF-3-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §3 |
| KMIPKIT-REQ-PROF-3-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §3 |
| KMIPKIT-REQ-PROF-3-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §3 |
| KMIPKIT-REQ-PROF-3-006 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §3 |
| KMIPKIT-REQ-PROF-3.1-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §3.1 |
| KMIPKIT-REQ-PROF-3.1.1-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §3.1.1 |
| KMIPKIT-REQ-PROF-3.1.1-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §3.1.1 |
| KMIPKIT-REQ-PROF-3.1.1-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §3.1.1 |
| KMIPKIT-REQ-PROF-3.1.2-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §3.1.2 |
| KMIPKIT-REQ-PROF-3.1.2-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §3.1.2 |
| KMIPKIT-REQ-PROF-3.2-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §3.2 |
| KMIPKIT-REQ-PROF-3.2.1-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §3.2.1 |
| KMIPKIT-REQ-PROF-3.2.2-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §3.2.2 |
| KMIPKIT-REQ-PROF-3.2.3-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §3.2.3 |
| KMIPKIT-REQ-PROF-3.2.4-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §3.2.4 |
| KMIPKIT-REQ-PROF-4.1-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §4.1 |
| KMIPKIT-REQ-PROF-4.1-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §4.1 |
| KMIPKIT-REQ-PROF-4.1.2-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §4.1.2 |
| KMIPKIT-REQ-PROF-4.1.2-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §4.1.2 |
| KMIPKIT-REQ-PROF-4.1.2-007 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §4.1.2 |
| KMIPKIT-REQ-PROF-4.1.2-008 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §4.1.2 |
| KMIPKIT-REQ-PROF-4.1.2-009 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §4.1.2 |
| KMIPKIT-REQ-PROF-5.10.1-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.10.1 |
| KMIPKIT-REQ-PROF-5.10.1-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.10.1 |
| KMIPKIT-REQ-PROF-5.10.1-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.10.1 |
| KMIPKIT-REQ-PROF-5.11.1-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.11.1 |
| KMIPKIT-REQ-PROF-5.11.1-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.11.1 |
| KMIPKIT-REQ-PROF-5.11.1-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.11.1 |
| KMIPKIT-REQ-PROF-5.11.1-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.11.1 |
| KMIPKIT-REQ-PROF-5.12.2-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.12.2 |
| KMIPKIT-REQ-PROF-5.12.2-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.12.2 |
| KMIPKIT-REQ-PROF-5.12.2-004-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.12.2 |
| KMIPKIT-REQ-PROF-5.12.2-004-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.12.2 |
| KMIPKIT-REQ-PROF-5.12.2-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.12.2 |
| KMIPKIT-REQ-PROF-5.12.2-007-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.12.2 |
| KMIPKIT-REQ-PROF-5.12.2-007-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.12.2 |
| KMIPKIT-REQ-PROF-5.12.3-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.12.3 |
| KMIPKIT-REQ-PROF-5.12.3-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.12.3 |
| KMIPKIT-REQ-PROF-5.12.4-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.12.4 |
| KMIPKIT-REQ-PROF-5.12.4-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.12.4 |
| KMIPKIT-REQ-PROF-5.12.4-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.12.4 |
| KMIPKIT-REQ-PROF-5.12.4-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.12.4 |
| KMIPKIT-REQ-PROF-5.12.4-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.12.4 |
| KMIPKIT-REQ-PROF-5.12.4-006 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.12.4 |
| KMIPKIT-REQ-PROF-5.12.4-007 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.12.4 |
| KMIPKIT-REQ-PROF-5.12.4-008 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.12.4 |
| KMIPKIT-REQ-PROF-5.12.4-009 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.12.4 |
| KMIPKIT-REQ-PROF-5.12.6.1-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.12.6.1 |
| KMIPKIT-REQ-PROF-5.12.6.2-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.12.6.2 |
| KMIPKIT-REQ-PROF-5.12.6.2-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.12.6.2 |
| KMIPKIT-REQ-PROF-5.12.6.2-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.12.6.2 |
| KMIPKIT-REQ-PROF-5.12.6.3-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.12.6.3 |
| KMIPKIT-REQ-PROF-5.12.6.3-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.12.6.3 |
| KMIPKIT-REQ-PROF-5.12.6.3-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.12.6.3 |
| KMIPKIT-REQ-PROF-5.12.6.3-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.12.6.3 |
| KMIPKIT-REQ-PROF-5.13.1-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.13.1 |
| KMIPKIT-REQ-PROF-5.13.1-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.13.1 |
| KMIPKIT-REQ-PROF-5.15-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.15 |
| KMIPKIT-REQ-PROF-5.15-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.15 |
| KMIPKIT-REQ-PROF-5.15-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.15 |
| KMIPKIT-REQ-PROF-5.15-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.15 |
| KMIPKIT-REQ-PROF-5.17-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.17 |
| KMIPKIT-REQ-PROF-5.17.1-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.17.1 |
| KMIPKIT-REQ-PROF-5.17.1-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.17.1 |
| KMIPKIT-REQ-PROF-5.17.2-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.17.2 |
| KMIPKIT-REQ-PROF-5.18.1-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.18.1 |
| KMIPKIT-REQ-PROF-5.18.1-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.18.1 |
| KMIPKIT-REQ-PROF-5.18.3-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.18.3 |
| KMIPKIT-REQ-PROF-5.18.3-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.18.3 |
| KMIPKIT-REQ-PROF-5.18.3-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.18.3 |
| KMIPKIT-REQ-PROF-5.18.3-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.18.3 |
| KMIPKIT-REQ-PROF-5.3.1-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.3.1 |
| KMIPKIT-REQ-PROF-5.3.1-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.3.1 |
| KMIPKIT-REQ-PROF-5.3.1-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.3.1 |
| KMIPKIT-REQ-PROF-5.3.1-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.3.1 |
| KMIPKIT-REQ-PROF-5.3.1-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.3.1 |
| KMIPKIT-REQ-PROF-5.3.1-008 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.3.1 |
| KMIPKIT-REQ-PROF-5.3.1-009 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.3.1 |
| KMIPKIT-REQ-PROF-5.3.1-010 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.3.1 |
| KMIPKIT-REQ-PROF-5.6.1-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.6.1 |
| KMIPKIT-REQ-PROF-5.6.1-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.6.1 |
| KMIPKIT-REQ-PROF-5.6.1-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.6.1 |
| KMIPKIT-REQ-PROF-5.7.1-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.7.1 |
| KMIPKIT-REQ-PROF-5.7.1-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.7.1 |
| KMIPKIT-REQ-PROF-5.7.1-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.7.1 |
| KMIPKIT-REQ-PROF-5.7.2-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.7.2 |
| KMIPKIT-REQ-PROF-5.7.2-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.7.2 |
| KMIPKIT-REQ-PROF-5.7.2-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.7.2 |
| KMIPKIT-REQ-PROF-5.7.3-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.7.3 |
| KMIPKIT-REQ-PROF-5.7.3-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.7.3 |
| KMIPKIT-REQ-PROF-5.7.3-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.7.3 |
| KMIPKIT-REQ-PROF-5.8.1-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.8.1 |
| KMIPKIT-REQ-PROF-5.8.1-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.8.1 |
| KMIPKIT-REQ-PROF-5.8.1-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.8.1 |
| KMIPKIT-REQ-PROF-5.9.1-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.9.1 |
| KMIPKIT-REQ-PROF-5.9.1-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.9.1 |
| KMIPKIT-REQ-PROF-5.9.1-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.9.1 |
| KMIPKIT-REQ-PROF-5.9.1-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.9.1 |
| KMIPKIT-REQ-PROF-5.9.2-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.9.2 |
| KMIPKIT-REQ-PROF-5.9.2-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.9.2 |
| KMIPKIT-REQ-PROF-5.9.2-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.9.2 |
| KMIPKIT-REQ-PROF-5.9.2-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.9.2 |
| KMIPKIT-REQ-PROF-5.9.3-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.9.3 |
| KMIPKIT-REQ-PROF-5.9.3-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.9.3 |
| KMIPKIT-REQ-PROF-5.9.3-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.9.3 |
| KMIPKIT-REQ-PROF-5.9.3-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §5.9.3 |
| KMIPKIT-REQ-PROF-6-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6 |
| KMIPKIT-REQ-PROF-6.1-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.1 |
| KMIPKIT-REQ-PROF-6.1-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.1 |
| KMIPKIT-REQ-PROF-6.1-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.1 |
| KMIPKIT-REQ-PROF-6.10-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.10 |
| KMIPKIT-REQ-PROF-6.10-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.10 |
| KMIPKIT-REQ-PROF-6.10-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.10 |
| KMIPKIT-REQ-PROF-6.10-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.10 |
| KMIPKIT-REQ-PROF-6.12-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.12 |
| KMIPKIT-REQ-PROF-6.12-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.12 |
| KMIPKIT-REQ-PROF-6.12-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.12 |
| KMIPKIT-REQ-PROF-6.12-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.12 |
| KMIPKIT-REQ-PROF-6.13-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.13 |
| KMIPKIT-REQ-PROF-6.13-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.13 |
| KMIPKIT-REQ-PROF-6.13-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.13 |
| KMIPKIT-REQ-PROF-6.14-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.14 |
| KMIPKIT-REQ-PROF-6.14-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.14 |
| KMIPKIT-REQ-PROF-6.14-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.14 |
| KMIPKIT-REQ-PROF-6.16-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.16 |
| KMIPKIT-REQ-PROF-6.16-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.16 |
| KMIPKIT-REQ-PROF-6.16-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.16 |
| KMIPKIT-REQ-PROF-6.16-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.16 |
| KMIPKIT-REQ-PROF-6.18-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.18 |
| KMIPKIT-REQ-PROF-6.18-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.18 |
| KMIPKIT-REQ-PROF-6.18-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.18 |
| KMIPKIT-REQ-PROF-6.18-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.18 |
| KMIPKIT-REQ-PROF-6.19-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.19 |
| KMIPKIT-REQ-PROF-6.19-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.19 |
| KMIPKIT-REQ-PROF-6.19-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.19 |
| KMIPKIT-REQ-PROF-6.19-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.19 |
| KMIPKIT-REQ-PROF-6.20-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.20 |
| KMIPKIT-REQ-PROF-6.20-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.20 |
| KMIPKIT-REQ-PROF-6.20-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.20 |
| KMIPKIT-REQ-PROF-6.20-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.20 |
| KMIPKIT-REQ-PROF-6.24-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.24 |
| KMIPKIT-REQ-PROF-6.24-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.24 |
| KMIPKIT-REQ-PROF-6.24-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.24 |
| KMIPKIT-REQ-PROF-6.24-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.24 |
| KMIPKIT-REQ-PROF-6.26-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.26 |
| KMIPKIT-REQ-PROF-6.26-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.26 |
| KMIPKIT-REQ-PROF-6.26-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.26 |
| KMIPKIT-REQ-PROF-6.26-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.26 |
| KMIPKIT-REQ-PROF-6.28-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.28 |
| KMIPKIT-REQ-PROF-6.28-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.28 |
| KMIPKIT-REQ-PROF-6.28-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.28 |
| KMIPKIT-REQ-PROF-6.28-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.28 |
| KMIPKIT-REQ-PROF-6.28-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.28 |
| KMIPKIT-REQ-PROF-6.28-007 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.28 |
| KMIPKIT-REQ-PROF-6.30-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.30 |
| KMIPKIT-REQ-PROF-6.30-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.30 |
| KMIPKIT-REQ-PROF-6.30-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.30 |
| KMIPKIT-REQ-PROF-6.30-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.30 |
| KMIPKIT-REQ-PROF-6.32-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.32 |
| KMIPKIT-REQ-PROF-6.32-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.32 |
| KMIPKIT-REQ-PROF-6.32-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.32 |
| KMIPKIT-REQ-PROF-6.32-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.32 |
| KMIPKIT-REQ-PROF-6.32-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.32 |
| KMIPKIT-REQ-PROF-6.34-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.34 |
| KMIPKIT-REQ-PROF-6.34-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.34 |
| KMIPKIT-REQ-PROF-6.34-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.34 |
| KMIPKIT-REQ-PROF-6.34-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.34 |
| KMIPKIT-REQ-PROF-6.4-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.4 |
| KMIPKIT-REQ-PROF-6.4-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.4 |
| KMIPKIT-REQ-PROF-6.4-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.4 |
| KMIPKIT-REQ-PROF-6.4-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-profiles §6.4 |
| KMIPKIT-REQ-SPEC-10.1.2-001 | Generic TTLV child-order preservation does not establish ordering for every KMIP Structure. Keep this requirement unassigned until each applicable Structure has approved typed-spec ownership and executable order-verification references. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources. | KMIPKIT-SRC-spec §10.1.2 |
| KMIPKIT-REQ-SPEC-10.1.2-002-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. The verification\_refs link project executable tests and do not imply official test-case evidence. | KMIPKIT-SRC-spec §10.1.2 |
| KMIPKIT-REQ-SPEC-10.1.2-002-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. The verification\_refs link project executable tests and do not imply official test-case evidence. | KMIPKIT-SRC-spec §10.1.2 |
| KMIPKIT-REQ-SPEC-10.1.5-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. The verification\_refs link project executable tests and do not imply official test-case evidence. | KMIPKIT-SRC-spec §10.1.5 |
| KMIPKIT-REQ-SPEC-10.1.5-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. The verification\_refs link project executable tests and do not imply official test-case evidence. | KMIPKIT-SRC-spec §10.1.5 |
| KMIPKIT-REQ-SPEC-10.4-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §10.4 |
| KMIPKIT-REQ-SPEC-10.4-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §10.4 |
| KMIPKIT-REQ-SPEC-10.4-001-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §10.4 |
| KMIPKIT-REQ-SPEC-11-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §11 |
| KMIPKIT-REQ-SPEC-11.56-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. The verification\_refs link project executable tests and do not imply official test-case evidence. | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-REQ-SPEC-12-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §12 |
| KMIPKIT-REQ-SPEC-12-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §12 |
| KMIPKIT-REQ-SPEC-14.1-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §14.1 |
| KMIPKIT-REQ-SPEC-14.1-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §14.1 |
| KMIPKIT-REQ-SPEC-2.1-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §2.1 |
| KMIPKIT-REQ-SPEC-2.2-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §2.2 |
| KMIPKIT-REQ-SPEC-2.3-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §2.3 |
| KMIPKIT-REQ-SPEC-2.3-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §2.3 |
| KMIPKIT-REQ-SPEC-2.3-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §2.3 |
| KMIPKIT-REQ-SPEC-2.4-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §2.4 |
| KMIPKIT-REQ-SPEC-2.4-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §2.4 |
| KMIPKIT-REQ-SPEC-2.4-002-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §2.4 |
| KMIPKIT-REQ-SPEC-2.4-002-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §2.4 |
| KMIPKIT-REQ-SPEC-2.4-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §2.4 |
| KMIPKIT-REQ-SPEC-2.5-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §2.5 |
| KMIPKIT-REQ-SPEC-2.6-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §2.6 |
| KMIPKIT-REQ-SPEC-2.7-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §2.7 |
| KMIPKIT-REQ-SPEC-2.7-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §2.7 |
| KMIPKIT-REQ-SPEC-2.8-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §2.8 |
| KMIPKIT-REQ-SPEC-2.8-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §2.8 |
| KMIPKIT-REQ-SPEC-2.8-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §2.8 |
| KMIPKIT-REQ-SPEC-2.8-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §2.8 |
| KMIPKIT-REQ-SPEC-2.9-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §2.9 |
| KMIPKIT-REQ-SPEC-3.1-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-REQ-SPEC-3.1-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-REQ-SPEC-3.1-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-REQ-SPEC-3.1-004-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-REQ-SPEC-3.1-004-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-REQ-SPEC-3.1-005-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-REQ-SPEC-3.1-005-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-REQ-SPEC-3.1-006 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-REQ-SPEC-3.10-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.10 |
| KMIPKIT-REQ-SPEC-3.11-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.11 |
| KMIPKIT-REQ-SPEC-3.12-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.12 |
| KMIPKIT-REQ-SPEC-3.2-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.2 |
| KMIPKIT-REQ-SPEC-3.2-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.2 |
| KMIPKIT-REQ-SPEC-3.3-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-REQ-SPEC-3.3-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-REQ-SPEC-3.3-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-REQ-SPEC-3.3-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-REQ-SPEC-3.3-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-REQ-SPEC-3.3-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-REQ-SPEC-3.3-006 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-REQ-SPEC-3.3-007 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-REQ-SPEC-3.3-008 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-REQ-SPEC-3.3-009 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-REQ-SPEC-3.4-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.4 |
| KMIPKIT-REQ-SPEC-3.5-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.5 |
| KMIPKIT-REQ-SPEC-3.6-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.6 |
| KMIPKIT-REQ-SPEC-3.7-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.7 |
| KMIPKIT-REQ-SPEC-3.7-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.7 |
| KMIPKIT-REQ-SPEC-3.8-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.8 |
| KMIPKIT-REQ-SPEC-3.9-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §3.9 |
| KMIPKIT-REQ-SPEC-4-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4 |
| KMIPKIT-REQ-SPEC-4-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4 |
| KMIPKIT-REQ-SPEC-4-001-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4 |
| KMIPKIT-REQ-SPEC-4-001-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4 |
| KMIPKIT-REQ-SPEC-4-001-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4 |
| KMIPKIT-REQ-SPEC-4.1-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.1 |
| KMIPKIT-REQ-SPEC-4.1-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.1 |
| KMIPKIT-REQ-SPEC-4.1-001-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.1 |
| KMIPKIT-REQ-SPEC-4.11-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.11 |
| KMIPKIT-REQ-SPEC-4.14-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.14 |
| KMIPKIT-REQ-SPEC-4.14-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.14 |
| KMIPKIT-REQ-SPEC-4.16-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-REQ-SPEC-4.16-001-002 | The source states a MAY permission; it does not require local type gating. Preserve caller values and do not infer unavailable server object types. | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-REQ-SPEC-4.16-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-REQ-SPEC-4.16-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-REQ-SPEC-4.17-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.17 |
| KMIPKIT-REQ-SPEC-4.17-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.17 |
| KMIPKIT-REQ-SPEC-4.18-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.18 |
| KMIPKIT-REQ-SPEC-4.18-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.18 |
| KMIPKIT-REQ-SPEC-4.2-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.2 |
| KMIPKIT-REQ-SPEC-4.21-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.21 |
| KMIPKIT-REQ-SPEC-4.26-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.26 |
| KMIPKIT-REQ-SPEC-4.27-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.27 |
| KMIPKIT-REQ-SPEC-4.27-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.27 |
| KMIPKIT-REQ-SPEC-4.28-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.28 |
| KMIPKIT-REQ-SPEC-4.28-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.28 |
| KMIPKIT-REQ-SPEC-4.28-001-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.28 |
| KMIPKIT-REQ-SPEC-4.28-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.28 |
| KMIPKIT-REQ-SPEC-4.30-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.30 |
| KMIPKIT-REQ-SPEC-4.31-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.31 |
| KMIPKIT-REQ-SPEC-4.32-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.32 |
| KMIPKIT-REQ-SPEC-4.32-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.32 |
| KMIPKIT-REQ-SPEC-4.34-001-001 | MAY NOT update/delete is not a defined §1.2 keyword. Keep clear client MAY/SHOULD portions, but leave prohibition strength unresolved. Clear client sub-obligations are retained as separate records; disputed wording remains unresolved. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.34 |
| KMIPKIT-REQ-SPEC-4.34-001-002 | MAY NOT update/delete is not a defined §1.2 keyword. Keep clear client MAY/SHOULD portions, but leave prohibition strength unresolved. Clear client sub-obligations are retained as separate records; disputed wording remains unresolved. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.34 |
| KMIPKIT-REQ-SPEC-4.35-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.35 |
| KMIPKIT-REQ-SPEC-4.35-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.35 |
| KMIPKIT-REQ-SPEC-4.35-001-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.35 |
| KMIPKIT-REQ-SPEC-4.38-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.38 |
| KMIPKIT-REQ-SPEC-4.38-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.38 |
| KMIPKIT-REQ-SPEC-4.38-001-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.38 |
| KMIPKIT-REQ-SPEC-4.38-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.38 |
| KMIPKIT-REQ-SPEC-4.4-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.4 |
| KMIPKIT-REQ-SPEC-4.4-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.4 |
| KMIPKIT-REQ-SPEC-4.40-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.40 |
| KMIPKIT-REQ-SPEC-4.40-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.40 |
| KMIPKIT-REQ-SPEC-4.40-001-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.40 |
| KMIPKIT-REQ-SPEC-4.40-001-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.40 |
| KMIPKIT-REQ-SPEC-4.40-001-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.40 |
| KMIPKIT-REQ-SPEC-4.41-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.41 |
| KMIPKIT-REQ-SPEC-4.41-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.41 |
| KMIPKIT-REQ-SPEC-4.41-001-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.41 |
| KMIPKIT-REQ-SPEC-4.41-001-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.41 |
| KMIPKIT-REQ-SPEC-4.46-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.46 |
| KMIPKIT-REQ-SPEC-4.46-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.46 |
| KMIPKIT-REQ-SPEC-4.47-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.47 |
| KMIPKIT-REQ-SPEC-4.47-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.47 |
| KMIPKIT-REQ-SPEC-4.47-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.47 |
| KMIPKIT-REQ-SPEC-4.57-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-003-001 | §4.57 says “Protect Stop” then “Process Stop Date”; §4.41 defines Protect Stop Date. Preserve the state rules, leave this date reference unresolved. Clear client sub-obligations are retained as separate records; disputed wording remains unresolved. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-003-002 | §4.57 says “Protect Stop” then “Process Stop Date”; §4.41 defines Protect Stop Date. Preserve the state rules, leave this date reference unresolved. Clear client sub-obligations are retained as separate records; disputed wording remains unresolved. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-003-003 | §4.57 says “Protect Stop” then “Process Stop Date”; §4.41 defines Protect Stop Date. Preserve the state rules, leave this date reference unresolved. Clear client sub-obligations are retained as separate records; disputed wording remains unresolved. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-004-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-004-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-004-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-005-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-005-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-005-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-006 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-007 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.59-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.59 |
| KMIPKIT-REQ-SPEC-4.59-003-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.59 |
| KMIPKIT-REQ-SPEC-4.59-003-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.59 |
| KMIPKIT-REQ-SPEC-4.60-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.60 |
| KMIPKIT-REQ-SPEC-4.61-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.61 |
| KMIPKIT-REQ-SPEC-4.62-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.62 |
| KMIPKIT-REQ-SPEC-4.62-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.62 |
| KMIPKIT-REQ-SPEC-4.63-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.63 |
| KMIPKIT-REQ-SPEC-4.63-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.63 |
| KMIPKIT-REQ-SPEC-4.63-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §4.63 |
| KMIPKIT-REQ-SPEC-5.1-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §5.1 |
| KMIPKIT-REQ-SPEC-5.1-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §5.1 |
| KMIPKIT-REQ-SPEC-5.2-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §5.2 |
| KMIPKIT-REQ-SPEC-5.2-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §5.2 |
| KMIPKIT-REQ-SPEC-5.3-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §5.3 |
| KMIPKIT-REQ-SPEC-5.3-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §5.3 |
| KMIPKIT-REQ-SPEC-5.4-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §5.4 |
| KMIPKIT-REQ-SPEC-5.4-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §5.4 |
| KMIPKIT-REQ-SPEC-5.5-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §5.5 |
| KMIPKIT-REQ-SPEC-5.5-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §5.5 |
| KMIPKIT-REQ-SPEC-5.6-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §5.6 |
| KMIPKIT-REQ-SPEC-5.7-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §5.7 |
| KMIPKIT-REQ-SPEC-6.1-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1 |
| KMIPKIT-REQ-SPEC-6.1-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1 |
| KMIPKIT-REQ-SPEC-6.1-001-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1 |
| KMIPKIT-REQ-SPEC-6.1-003-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1 |
| KMIPKIT-REQ-SPEC-6.1-003-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1 |
| KMIPKIT-REQ-SPEC-6.1-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1 |
| KMIPKIT-REQ-SPEC-6.1-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1 |
| KMIPKIT-REQ-SPEC-6.1.10-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.10 |
| KMIPKIT-REQ-SPEC-6.1.11-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.11 |
| KMIPKIT-REQ-SPEC-6.1.11-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.11 |
| KMIPKIT-REQ-SPEC-6.1.11-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.11 |
| KMIPKIT-REQ-SPEC-6.1.11-004-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.11 |
| KMIPKIT-REQ-SPEC-6.1.11-004-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.11 |
| KMIPKIT-REQ-SPEC-6.1.11-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.11 |
| KMIPKIT-REQ-SPEC-6.1.11-006 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.11 |
| KMIPKIT-REQ-SPEC-6.1.13-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.13 |
| KMIPKIT-REQ-SPEC-6.1.13-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.13 |
| KMIPKIT-REQ-SPEC-6.1.14-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.14 |
| KMIPKIT-REQ-SPEC-6.1.14-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.14 |
| KMIPKIT-REQ-SPEC-6.1.14-001-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.14 |
| KMIPKIT-REQ-SPEC-6.1.14-001-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.14 |
| KMIPKIT-REQ-SPEC-6.1.14-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.14 |
| KMIPKIT-REQ-SPEC-6.1.14-006-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.14 |
| KMIPKIT-REQ-SPEC-6.1.14-006-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.14 |
| KMIPKIT-REQ-SPEC-6.1.14-007-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.14 |
| KMIPKIT-REQ-SPEC-6.1.14-007-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.14 |
| KMIPKIT-REQ-SPEC-6.1.16-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.16 |
| KMIPKIT-REQ-SPEC-6.1.16-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.16 |
| KMIPKIT-REQ-SPEC-6.1.16-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.16 |
| KMIPKIT-REQ-SPEC-6.1.17-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.17 |
| KMIPKIT-REQ-SPEC-6.1.17-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.17 |
| KMIPKIT-REQ-SPEC-6.1.17-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.17 |
| KMIPKIT-REQ-SPEC-6.1.17-005-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.17 |
| KMIPKIT-REQ-SPEC-6.1.17-005-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.17 |
| KMIPKIT-REQ-SPEC-6.1.17-006 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.17 |
| KMIPKIT-REQ-SPEC-6.1.19-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.19 |
| KMIPKIT-REQ-SPEC-6.1.19-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.19 |
| KMIPKIT-REQ-SPEC-6.1.2-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.2 |
| KMIPKIT-REQ-SPEC-6.1.2-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.2 |
| KMIPKIT-REQ-SPEC-6.1.20-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.20 |
| KMIPKIT-REQ-SPEC-6.1.20-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.20 |
| KMIPKIT-REQ-SPEC-6.1.20-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.20 |
| KMIPKIT-REQ-SPEC-6.1.23-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.23 |
| KMIPKIT-REQ-SPEC-6.1.23-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.23 |
| KMIPKIT-REQ-SPEC-6.1.23-002-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.23 |
| KMIPKIT-REQ-SPEC-6.1.23-002-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.23 |
| KMIPKIT-REQ-SPEC-6.1.25-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.25 |
| KMIPKIT-REQ-SPEC-6.1.25-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.25 |
| KMIPKIT-REQ-SPEC-6.1.25-006 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.25 |
| KMIPKIT-REQ-SPEC-6.1.27-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.27 |
| KMIPKIT-REQ-SPEC-6.1.27-002-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.27 |
| KMIPKIT-REQ-SPEC-6.1.27-002-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.27 |
| KMIPKIT-REQ-SPEC-6.1.27-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.27 |
| KMIPKIT-REQ-SPEC-6.1.28-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.28-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.28-004-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.28-004-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.28-007 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.28-008-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.28-008-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.28-009-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.28-009-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.28-011 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.28-012 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.28-013-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.28-013-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.3-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.3 |
| KMIPKIT-REQ-SPEC-6.1.32-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.32 |
| KMIPKIT-REQ-SPEC-6.1.32-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.32 |
| KMIPKIT-REQ-SPEC-6.1.32-004-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.32 |
| KMIPKIT-REQ-SPEC-6.1.32-004-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.32 |
| KMIPKIT-REQ-SPEC-6.1.33-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.33 |
| KMIPKIT-REQ-SPEC-6.1.33-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.33 |
| KMIPKIT-REQ-SPEC-6.1.33-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.33 |
| KMIPKIT-REQ-SPEC-6.1.33-005-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.33 |
| KMIPKIT-REQ-SPEC-6.1.33-005-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.33 |
| KMIPKIT-REQ-SPEC-6.1.34-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.34 |
| KMIPKIT-REQ-SPEC-6.1.34-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.34 |
| KMIPKIT-REQ-SPEC-6.1.34-001-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.34 |
| KMIPKIT-REQ-SPEC-6.1.34-001-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.34 |
| KMIPKIT-REQ-SPEC-6.1.35-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.35 |
| KMIPKIT-REQ-SPEC-6.1.35-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.35 |
| KMIPKIT-REQ-SPEC-6.1.35-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.35 |
| KMIPKIT-REQ-SPEC-6.1.35-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.35 |
| KMIPKIT-REQ-SPEC-6.1.38-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.38 |
| KMIPKIT-REQ-SPEC-6.1.38-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.38 |
| KMIPKIT-REQ-SPEC-6.1.4-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.4 |
| KMIPKIT-REQ-SPEC-6.1.40-014 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.40 |
| KMIPKIT-REQ-SPEC-6.1.40-016 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.40 |
| KMIPKIT-REQ-SPEC-6.1.42-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.42 |
| KMIPKIT-REQ-SPEC-6.1.42-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.42 |
| KMIPKIT-REQ-SPEC-6.1.43-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.43 |
| KMIPKIT-REQ-SPEC-6.1.43-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.43 |
| KMIPKIT-REQ-SPEC-6.1.43-007 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.43 |
| KMIPKIT-REQ-SPEC-6.1.43-009-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.43 |
| KMIPKIT-REQ-SPEC-6.1.43-009-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.43 |
| KMIPKIT-REQ-SPEC-6.1.43-010-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.43 |
| KMIPKIT-REQ-SPEC-6.1.43-010-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.43 |
| KMIPKIT-REQ-SPEC-6.1.43-011 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.43 |
| KMIPKIT-REQ-SPEC-6.1.44-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.44 |
| KMIPKIT-REQ-SPEC-6.1.44-003-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.44 |
| KMIPKIT-REQ-SPEC-6.1.44-003-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.44 |
| KMIPKIT-REQ-SPEC-6.1.45-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.45 |
| KMIPKIT-REQ-SPEC-6.1.45-002-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.45 |
| KMIPKIT-REQ-SPEC-6.1.45-002-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.45 |
| KMIPKIT-REQ-SPEC-6.1.45-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.45 |
| KMIPKIT-REQ-SPEC-6.1.45-006-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.45 |
| KMIPKIT-REQ-SPEC-6.1.45-006-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.45 |
| KMIPKIT-REQ-SPEC-6.1.45-008 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.45 |
| KMIPKIT-REQ-SPEC-6.1.46-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.46 |
| KMIPKIT-REQ-SPEC-6.1.46-004-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.46 |
| KMIPKIT-REQ-SPEC-6.1.46-004-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.46 |
| KMIPKIT-REQ-SPEC-6.1.47-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.47 |
| KMIPKIT-REQ-SPEC-6.1.47-004-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.47 |
| KMIPKIT-REQ-SPEC-6.1.47-004-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.47 |
| KMIPKIT-REQ-SPEC-6.1.48-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.48 |
| KMIPKIT-REQ-SPEC-6.1.48-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.48 |
| KMIPKIT-REQ-SPEC-6.1.48-006 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.48 |
| KMIPKIT-REQ-SPEC-6.1.5-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.5 |
| KMIPKIT-REQ-SPEC-6.1.50-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.50 |
| KMIPKIT-REQ-SPEC-6.1.51-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.51 |
| KMIPKIT-REQ-SPEC-6.1.55-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.55 |
| KMIPKIT-REQ-SPEC-6.1.55-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.55 |
| KMIPKIT-REQ-SPEC-6.1.55-005-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.55 |
| KMIPKIT-REQ-SPEC-6.1.55-005-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.55 |
| KMIPKIT-REQ-SPEC-6.1.56-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.56 |
| KMIPKIT-REQ-SPEC-6.1.56-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.56 |
| KMIPKIT-REQ-SPEC-6.1.56-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.56 |
| KMIPKIT-REQ-SPEC-6.1.56-006 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.56 |
| KMIPKIT-REQ-SPEC-6.1.56-007-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.56 |
| KMIPKIT-REQ-SPEC-6.1.56-007-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.56 |
| KMIPKIT-REQ-SPEC-6.1.57-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.57 |
| KMIPKIT-REQ-SPEC-6.1.57-002-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.57 |
| KMIPKIT-REQ-SPEC-6.1.57-002-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.57 |
| KMIPKIT-REQ-SPEC-6.1.57-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.57 |
| KMIPKIT-REQ-SPEC-6.1.57-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.57 |
| KMIPKIT-REQ-SPEC-6.1.57-006 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.57 |
| KMIPKIT-REQ-SPEC-6.1.6-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.6 |
| KMIPKIT-REQ-SPEC-6.1.6-002-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.6 |
| KMIPKIT-REQ-SPEC-6.1.6-002-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.6 |
| KMIPKIT-REQ-SPEC-6.1.6-002-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.6 |
| KMIPKIT-REQ-SPEC-6.1.6-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.6 |
| KMIPKIT-REQ-SPEC-6.1.6-007 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.6 |
| KMIPKIT-REQ-SPEC-6.1.7-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.7 |
| KMIPKIT-REQ-SPEC-6.1.7-002-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.7 |
| KMIPKIT-REQ-SPEC-6.1.7-002-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.7 |
| KMIPKIT-REQ-SPEC-6.1.7-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.7 |
| KMIPKIT-REQ-SPEC-6.1.7-004-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.7 |
| KMIPKIT-REQ-SPEC-6.1.7-004-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.7 |
| KMIPKIT-REQ-SPEC-6.1.7-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.7 |
| KMIPKIT-REQ-SPEC-6.1.9-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.9 |
| KMIPKIT-REQ-SPEC-6.1.9-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.9 |
| KMIPKIT-REQ-SPEC-6.1.9-006 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.9 |
| KMIPKIT-REQ-SPEC-6.1.9-007 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §6.1.9 |
| KMIPKIT-REQ-SPEC-7.12-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §7.12 |
| KMIPKIT-REQ-SPEC-7.12-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §7.12 |
| KMIPKIT-REQ-SPEC-7.12-005-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §7.12 |
| KMIPKIT-REQ-SPEC-7.12-005-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §7.12 |
| KMIPKIT-REQ-SPEC-7.18-001-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §7.18 |
| KMIPKIT-REQ-SPEC-7.18-001-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §7.18 |
| KMIPKIT-REQ-SPEC-7.18-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §7.18 |
| KMIPKIT-REQ-SPEC-7.18-004 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §7.18 |
| KMIPKIT-REQ-SPEC-7.18-005 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §7.18 |
| KMIPKIT-REQ-SPEC-7.18-006 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §7.18 |
| KMIPKIT-REQ-SPEC-7.26-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §7.26 |
| KMIPKIT-REQ-SPEC-7.27-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §7.27 |
| KMIPKIT-REQ-SPEC-7.28-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §7.28 |
| KMIPKIT-REQ-SPEC-7.29-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §7.29 |
| KMIPKIT-REQ-SPEC-7.30-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §7.30 |
| KMIPKIT-REQ-SPEC-7.31-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §7.31 |
| KMIPKIT-REQ-SPEC-7.32-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §7.32 |
| KMIPKIT-REQ-SPEC-7.33-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §7.33 |
| KMIPKIT-REQ-SPEC-7.35-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §7.35 |
| KMIPKIT-REQ-SPEC-7.36-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §7.36 |
| KMIPKIT-REQ-SPEC-7.36-002-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §7.36 |
| KMIPKIT-REQ-SPEC-7.36-002-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §7.36 |
| KMIPKIT-REQ-SPEC-7.36-003 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §7.36 |
| KMIPKIT-REQ-SPEC-7.7-002 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §7.7 |
| KMIPKIT-REQ-SPEC-7.8-001 | No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §7.8 |
| KMIPKIT-REQ-SPEC-8-001 | KMIPKIT-0006 owns the in-memory client protocol model for this clause; outbound execution behavior remains assigned to its follow-on owner where specified. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §8 |
| KMIPKIT-REQ-SPEC-8-002 | KMIPKIT-0006 owns the in-memory client protocol model for this clause; outbound execution behavior remains assigned to its follow-on owner where specified. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §8 |
| KMIPKIT-REQ-SPEC-8-003-001 | Verified for the typed client's supported request options. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §8 |
| KMIPKIT-REQ-SPEC-8-003-002 | KMIPKIT-0006 owns the in-memory client protocol model for this clause; outbound execution behavior remains assigned to its follow-on owner where specified. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §8 |
| KMIPKIT-REQ-SPEC-8.1-002 | KMIPKIT-0006 owns the in-memory client protocol model for this clause; outbound execution behavior remains assigned to its follow-on owner where specified. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §8.1 |
| KMIPKIT-REQ-SPEC-8.2-002 | KMIPKIT-0006 owns the in-memory client protocol model for this clause; outbound execution behavior remains assigned to its follow-on owner where specified. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §8.2 |
| KMIPKIT-REQ-SPEC-8.3-003 | KMIPKIT-0006 owns the in-memory client protocol model for this clause; outbound execution behavior remains assigned to its follow-on owner where specified. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §8.3 |
| KMIPKIT-REQ-SPEC-8.3-004 | KMIPKIT-0006 owns the in-memory client protocol model for this clause; outbound execution behavior remains assigned to its follow-on owner where specified. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §8.3 |
| KMIPKIT-REQ-SPEC-9.1-001 | Implementation and executable verification are assigned to KMIPKIT-0009; KMIPKIT-0006 records this ownership transfer and does not claim the deferred client behavior. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.1 |
| KMIPKIT-REQ-SPEC-9.11-001 | Implementation and executable verification are assigned to KMIPKIT-0008; KMIPKIT-0006 records this ownership transfer and does not claim the deferred client behavior. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-REQ-SPEC-9.11-004-001 | Implementation and executable verification are assigned to KMIPKIT-0008; KMIPKIT-0006 does not choose a meaning for this requirement. The section 9.11 phrase “at least one field” has two plausible scopes: all six Table 412 fields or the four uniqueness fields named in the preceding sentence; see KMIPKIT-DISC-042. No interpretation is selected. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-REQ-SPEC-9.11-004-002 | Implementation and executable verification are assigned to KMIPKIT-0008; KMIPKIT-0006 records this ownership transfer and does not claim the deferred client behavior. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-REQ-SPEC-9.11-004-003 | Implementation and executable verification are assigned to KMIPKIT-0008; KMIPKIT-0006 records this ownership transfer and does not claim the deferred client behavior. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-REQ-SPEC-9.11-006 | Implementation and executable verification are assigned to KMIPKIT-0008; KMIPKIT-0006 records this ownership transfer and does not claim the deferred client behavior. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-REQ-SPEC-9.11-010-001 | Implementation and executable verification are assigned to KMIPKIT-0008; KMIPKIT-0006 records this ownership transfer and does not claim the deferred client behavior. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-REQ-SPEC-9.11-010-002 | Implementation and executable verification are assigned to KMIPKIT-0008; KMIPKIT-0006 records this ownership transfer and does not claim the deferred client behavior. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-REQ-SPEC-9.12-001-001 | KMIPKIT-0006 owns the in-memory client protocol model for this clause; outbound execution behavior remains assigned to its follow-on owner where specified. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.12 |
| KMIPKIT-REQ-SPEC-9.12-001-002 | The configured local response cap is enforced. Discover Versions declares no peer-visible Maximum Response Size, so the client does not claim handling a peer-declared value; an operation that declares one must verify that exact declaration. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.12 |
| KMIPKIT-REQ-SPEC-9.12-001-003 | Discover Versions is not classified as likely-large; the peer-visible field is omitted after assessing the recommendation. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.12 |
| KMIPKIT-REQ-SPEC-9.13-001-001 | KMIPKIT-0006 owns the in-memory client protocol model for this clause; outbound execution behavior remains assigned to its follow-on owner where specified. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.13 |
| KMIPKIT-REQ-SPEC-9.13-001-002 | KMIPKIT-0006 owns the in-memory client protocol model for this clause; outbound execution behavior remains assigned to its follow-on owner where specified. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.13 |
| KMIPKIT-REQ-SPEC-9.13-001-003 | KMIPKIT-0006 owns the in-memory client protocol model for this clause; outbound execution behavior remains assigned to its follow-on owner where specified. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.13 |
| KMIPKIT-REQ-SPEC-9.13-001-004 | The complete response is rejected for an unrecognized critical extension; no vendor critical-extension support is claimed. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.13 |
| KMIPKIT-REQ-SPEC-9.13-001-005 | OASIS permits treating an unknown non-critical extension as absent; KMIPKit's separate stricter policy preserves and exposes its opaque value. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.13 |
| KMIPKIT-REQ-SPEC-9.19-002 | Implementation and executable verification are assigned to KMIPKIT-0009; KMIPKIT-0006 records this ownership transfer and does not claim the deferred client behavior. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.19 |
| KMIPKIT-REQ-SPEC-9.2-001 | KMIPKIT-0006 owns the in-memory client protocol model for this clause; outbound execution behavior remains assigned to its follow-on owner where specified. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.2 |
| KMIPKIT-REQ-SPEC-9.20-001-001 | KMIPKIT-0006 owns the in-memory client protocol model for this clause; outbound execution behavior remains assigned to its follow-on owner where specified. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.20 |
| KMIPKIT-REQ-SPEC-9.20-001-002 | OD-004 explicitly defers countdown-derived outgoing Time Stamp generation. The section's countdown note discusses relative Date-attribute values and does not establish that a monotonic countdown may be serialized as the outgoing request Time Stamp. No implementation or verification is claimed. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.20 |
| KMIPKIT-REQ-SPEC-9.21-001-001 | KMIPKIT-0006 owns the in-memory client protocol model for this clause; outbound execution behavior remains assigned to its follow-on owner where specified. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.21 |
| KMIPKIT-REQ-SPEC-9.21-001-002 | Response items are associated by the sent Unique Batch Item ID independent of response order. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.21 |
| KMIPKIT-REQ-SPEC-9.3-001-001 | Implementation and executable verification are assigned to KMIPKIT-0008; KMIPKIT-0006 records this ownership transfer and does not claim the deferred client behavior. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.3 |
| KMIPKIT-REQ-SPEC-9.3-001-002 | KMIPKIT-0006 owns the in-memory client protocol model for this clause; outbound execution behavior remains assigned to its follow-on owner where specified. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.3 |
| KMIPKIT-REQ-SPEC-9.4-001-001 | Implementation and executable verification are assigned to KMIPKIT-0008; KMIPKIT-0006 only represents the optional Authentication header structure. The pinned KMIP Specification 2.1 section 9.4 uses lowercase “must” in the statement that multiple Credential structures must all be satisfied; section 1.2 defines uppercase key words by RFC 2119, so this classification remains unresolved in KMIPKIT-DISC-041. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §9.4 |
| KMIPKIT-REQ-SPEC-9.4-001-002 | Implementation and executable verification are assigned to KMIPKIT-0008; KMIPKIT-0006 records this ownership transfer and does not claim the deferred client behavior. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.4 |
| KMIPKIT-REQ-SPEC-9.4-001-003 | The pinned KMIP Specification 2.1 section 9.4 uses lowercase “must” in the statement that multiple Credential structures must all be satisfied; section 1.2 defines uppercase key words by RFC 2119, so this classification remains unresolved in KMIPKIT-DISC-041. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. No verification link is inferred; fixture-level evidence is unavailable where cited fixtures are missing. | KMIPKIT-SRC-spec §9.4 |
| KMIPKIT-REQ-SPEC-9.4-002 | Implementation and executable verification are assigned to KMIPKIT-0008; KMIPKIT-0006 records this ownership transfer and does not claim the deferred client behavior. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.4 |
| KMIPKIT-REQ-SPEC-9.5-001-001 | KMIPKIT-0006 owns the in-memory client protocol model for this clause; outbound execution behavior remains assigned to its follow-on owner where specified. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.5 |
| KMIPKIT-REQ-SPEC-9.5-001-002 | KMIPKIT-0006 owns the in-memory client protocol model for this clause; outbound execution behavior remains assigned to its follow-on owner where specified. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.5 |
| KMIPKIT-REQ-SPEC-9.6-001-001 | KMIPKIT-0006 owns the in-memory client protocol model for this clause; outbound execution behavior remains assigned to its follow-on owner where specified. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.6 |
| KMIPKIT-REQ-SPEC-9.6-001-002 | KMIPKIT-DEC-002 accepts assigned outbound values and preserves raw decoded values without claiming the Table 435 extension allocation invalid. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.6 |
| KMIPKIT-REQ-SPEC-9.6-001-003 | KMIPKIT-0006 owns the in-memory client protocol model for this clause; outbound execution behavior remains assigned to its follow-on owner where specified. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.6 |
| KMIPKIT-REQ-SPEC-9.7-001 | KMIPKIT-0006 owns the in-memory client protocol model for this clause; outbound execution behavior remains assigned to its follow-on owner where specified. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.7 |
| KMIPKIT-REQ-SPEC-9.8-001-001 | KMIPKIT-0006 owns the in-memory client protocol model for this clause; outbound execution behavior remains assigned to its follow-on owner where specified. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.8 |
| KMIPKIT-REQ-SPEC-9.8-001-002 | Classified by KMIPKIT-0006 as server\_only. This server duty is not implemented or claimed by the client library. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.8 |
| KMIPKIT-REQ-SPEC-9.8-001-003 | Classified by KMIPKIT-0006 as server\_only. This server duty is not implemented or claimed by the client library. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.8 |
| KMIPKIT-REQ-SPEC-9.9-001 | KMIPKIT-0006 owns the in-memory client protocol model for this clause; outbound execution behavior remains assigned to its follow-on owner where specified. No requirement-specific official Test Cases ID is explicitly linked by the pinned OASIS sources in KMIPKIT-0002. | KMIPKIT-SRC-spec §9.9 |

## Requirements needing negative verification

| Requirement | Strength | Negative verification | Status | Source |
| --- | --- | --- | --- | --- |
| KMIPKIT-REQ-PROF-3.1.1-005 | prohibited | required | unassigned | KMIPKIT-SRC-profiles §3.1.1 |
| KMIPKIT-REQ-PROF-3.1.2-005 | prohibited | required | unassigned | KMIPKIT-SRC-profiles §3.1.2 |
| KMIPKIT-REQ-PROF-5.12.3-003 | prohibited | required | unassigned | KMIPKIT-SRC-profiles §5.12.3 |
| KMIPKIT-REQ-SPEC-11-001 | prohibited | required | unassigned | KMIPKIT-SRC-spec §11 |
| KMIPKIT-REQ-SPEC-3.3-003 | prohibited | required | unassigned | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-REQ-SPEC-4-001-004 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4 |
| KMIPKIT-REQ-SPEC-4-001-005 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4 |
| KMIPKIT-REQ-SPEC-4.1-001-002 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4.1 |
| KMIPKIT-REQ-SPEC-4.1-001-003 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4.1 |
| KMIPKIT-REQ-SPEC-4.17-001-002 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4.17 |
| KMIPKIT-REQ-SPEC-4.18-001-001 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4.18 |
| KMIPKIT-REQ-SPEC-4.18-001-002 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4.18 |
| KMIPKIT-REQ-SPEC-4.28-001-001 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4.28 |
| KMIPKIT-REQ-SPEC-4.28-001-002 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4.28 |
| KMIPKIT-REQ-SPEC-4.30-001 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4.30 |
| KMIPKIT-REQ-SPEC-4.38-003 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4.38 |
| KMIPKIT-REQ-SPEC-4.40-001-002 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4.40 |
| KMIPKIT-REQ-SPEC-4.40-001-004 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4.40 |
| KMIPKIT-REQ-SPEC-4.40-001-005 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4.40 |
| KMIPKIT-REQ-SPEC-4.41-001-001 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4.41 |
| KMIPKIT-REQ-SPEC-4.41-001-003 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4.41 |
| KMIPKIT-REQ-SPEC-4.41-001-004 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4.41 |
| KMIPKIT-REQ-SPEC-4.57-001 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-002 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-003-003 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-004-001 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-005-001 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-006 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.57-007 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-REQ-SPEC-4.59-002 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4.59 |
| KMIPKIT-REQ-SPEC-4.59-003-001 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4.59 |
| KMIPKIT-REQ-SPEC-4.59-003-002 | prohibited | required | unassigned | KMIPKIT-SRC-spec §4.59 |
| KMIPKIT-REQ-SPEC-6.1.14-006-002 | prohibited | required | unassigned | KMIPKIT-SRC-spec §6.1.14 |
| KMIPKIT-REQ-SPEC-6.1.2-001-001 | prohibited | required | unassigned | KMIPKIT-SRC-spec §6.1.2 |
| KMIPKIT-REQ-SPEC-6.1.2-001-002 | prohibited | required | unassigned | KMIPKIT-SRC-spec §6.1.2 |
| KMIPKIT-REQ-SPEC-6.1.20-001-001 | prohibited | required | unassigned | KMIPKIT-SRC-spec §6.1.20 |
| KMIPKIT-REQ-SPEC-6.1.23-001-001 | prohibited | required | unassigned | KMIPKIT-SRC-spec §6.1.23 |
| KMIPKIT-REQ-SPEC-6.1.23-002-002 | prohibited | required | unassigned | KMIPKIT-SRC-spec §6.1.23 |
| KMIPKIT-REQ-SPEC-6.1.28-009-001 | prohibited | required | unassigned | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.28-009-002 | prohibited | required | unassigned | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-REQ-SPEC-6.1.3-001 | prohibited | required | unassigned | KMIPKIT-SRC-spec §6.1.3 |
| KMIPKIT-REQ-SPEC-6.1.35-001-002 | prohibited | required | unassigned | KMIPKIT-SRC-spec §6.1.35 |
| KMIPKIT-REQ-SPEC-6.1.38-001-002 | prohibited | required | unassigned | KMIPKIT-SRC-spec §6.1.38 |
| KMIPKIT-REQ-SPEC-6.1.44-003-002 | prohibited | required | unassigned | KMIPKIT-SRC-spec §6.1.44 |
| KMIPKIT-REQ-SPEC-6.1.50-001 | prohibited | required | unassigned | KMIPKIT-SRC-spec §6.1.50 |
| KMIPKIT-REQ-SPEC-6.1.51-001 | prohibited | required | unassigned | KMIPKIT-SRC-spec §6.1.51 |
| KMIPKIT-REQ-SPEC-7.18-002 | prohibited | required | unassigned | KMIPKIT-SRC-spec §7.18 |

## Unassigned protocol elements

| Element | Name | Kind | Direction | Scope | Wire value | Allocation | Source |
| --- | --- | --- | --- | --- | --- | --- | --- |
| KMIPKIT-ELEM-ATTRIBUTE-ACTIVATION-DATE | Activation Date | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.1, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-ALTERNATIVE-NAME | Alternative Name | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.2, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-ALWAYS-SENSITIVE | Always Sensitive | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.3, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-APPLICATION-SPECIFIC-INFORMATION | Application Specific Information | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.4, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-ARCHIVE-DATE | Archive Date | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.5, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-CERTIFICATE-ATTRIBUTES | Certificate Attributes | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.6 |
| KMIPKIT-ELEM-ATTRIBUTE-CERTIFICATE-LENGTH | Certificate Length | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.8, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-CERTIFICATE-TYPE | Certificate Type | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.7, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-COMMENT | Comment | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.9, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-COMPROMISE-DATE | Compromise Date | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.10, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-COMPROMISE-OCCURRENCE-DATE | Compromise Occurrence Date | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.11, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-CONTACT-INFORMATION | Contact Information | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.12, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-CRYPTOGRAPHIC-ALGORITHM | Cryptographic Algorithm | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.13, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-CRYPTOGRAPHIC-DOMAIN-PARAMETERS | Cryptographic Domain Parameters | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.14, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-CRYPTOGRAPHIC-LENGTH | Cryptographic Length | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.15, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-CRYPTOGRAPHIC-PARAMETERS | Cryptographic Parameters | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.16, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-CRYPTOGRAPHIC-USAGE-MASK | Cryptographic Usage Mask | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.17, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-DEACTIVATION-DATE | Deactivation Date | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.18, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-DESCRIPTION | Description | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.19, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-DESTROY-DATE | Destroy Date | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.20, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-DIGEST | Digest | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.21, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-DIGITAL-SIGNATURE-ALGORITHM | Digital Signature Algorithm | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.22, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-EXTRACTABLE | Extractable | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.23, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-FRESH | Fresh | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.24, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-INITIAL-DATE | Initial Date | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.25, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-KEY-FORMAT-TYPE | Key Format Type | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.26, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-KEY-VALUE-LOCATION | Key Value Location | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.27, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-KEY-VALUE-PRESENT | Key Value Present | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.28, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-LAST-CHANGE-DATE | Last Change Date | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.29, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-LEASE-TIME | Lease Time | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.30, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-LINK | Link | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.31, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-NAME | Name | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.32, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-NEVER-EXTRACTABLE | Never Extractable | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.33, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-NIST-KEY-TYPE | NIST Key Type | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.34, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-OBJECT-GROUP | Object Group | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.35, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-OBJECT-TYPE | Object Type | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.36, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-OPAQUE-DATA-TYPE | Opaque Data Type | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.37, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-ORIGINAL-CREATION-DATE | Original Creation Date | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.38, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-PKCS-12-FRIENDLY-NAME | PKCS#12 Friendly Name | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.39, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-PROCESS-START-DATE | Process Start Date | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.40, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-PROTECT-STOP-DATE | Protect Stop Date | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.41, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-PROTECTION-LEVEL | Protection Level | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.42, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-PROTECTION-PERIOD | Protection Period | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.43, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-PROTECTION-STORAGE-MASK | Protection Storage Mask | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.44, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-QUANTUM-SAFE | Quantum Safe | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.45, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-RANDOM-NUMBER-GENERATOR | Random Number Generator | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.46, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-REVOCATION-REASON | Revocation Reason | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.47, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-ROTATE-AUTOMATIC | Rotate Automatic | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.48, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-ROTATE-DATE | Rotate Date | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.49, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-ROTATE-GENERATION | Rotate Generation | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.50, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-ROTATE-INTERVAL | Rotate Interval | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.51, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-ROTATE-LATEST | Rotate Latest | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.52, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-ROTATE-NAME | Rotate Name | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.53, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-ROTATE-OFFSET | Rotate Offset | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.54, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-SENSITIVE | Sensitive | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.55, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-SHORT-UNIQUE-IDENTIFIER | Short Unique Identifier | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.56, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-STATE | State | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.57, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-1-ATTRIBUTES | Attributes | attribute\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §5.1 |
| KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-2-COMMON-ATTRIBUTES | Common Attributes | attribute\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §5.2 |
| KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-3-PRIVATE-KEY-ATTRIBUTES | Private Key Attributes | attribute\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §5.3 |
| KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-4-PUBLIC-KEY-ATTRIBUTES | Public Key Attributes | attribute\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §5.4 |
| KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-5-ATTRIBUTE-REFERENCE | Attribute Reference | attribute\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §5.5 |
| KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-6-CURRENT-ATTRIBUTE | Current Attribute | attribute\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §5.6 |
| KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-7-NEW-ATTRIBUTE | New Attribute | attribute\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §5.7 |
| KMIPKIT-ELEM-ATTRIBUTE-UNIQUE-IDENTIFIER | Unique Identifier | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.58, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-USAGE-LIMITS | Usage Limits | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.59, KMIPKIT-SRC-spec §7.40, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-VENDOR-ATTRIBUTE | Vendor Attribute | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.60 |
| KMIPKIT-ELEM-ATTRIBUTE-X-509-CERTIFICATE-IDENTIFIER | X.509 Certificate Identifier | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.61, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-X-509-CERTIFICATE-ISSUER | X.509 Certificate Issuer | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.62, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-X-509-CERTIFICATE-SUBJECT | X.509 Certificate Subject | attribute | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.63, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-BITMASK-CRYPTOGRAPHIC-USAGE-MASK | Cryptographic Usage Mask | bitmask | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-PROTECTION-STORAGE-MASK | Protection Storage Mask | bitmask | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-STORAGE-STATUS-MASK | Storage Status Mask | bitmask | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §12.3 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-AUTHENTICATE-00100000 | Authenticate | bitmask\_value | both | client\_1\_0 | 00100000 | assigned | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-CERTIFICATE-SIGN-00001000 | Certificate Sign | bitmask\_value | both | client\_1\_0 | 00001000 | assigned | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-CRL-SIGN-00002000 | CRL Sign | bitmask\_value | both | client\_1\_0 | 00002000 | assigned | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-DECRYPT-00000008 | Decrypt | bitmask\_value | both | client\_1\_0 | 00000008 | assigned | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-DERIVE-KEY-00000200 | Derive Key | bitmask\_value | both | client\_1\_0 | 00000200 | assigned | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-ENCRYPT-00000004 | Encrypt | bitmask\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-EXTENSIONS-XXX00000 | Extensions | bitmask\_value | both | client\_1\_0 | XXX00000 | extension | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-FPE-DECRYPT-00800000 | FPE Decrypt | bitmask\_value | both | client\_1\_0 | 00800000 | assigned | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-FPE-ENCRYPT-00400000 | FPE Encrypt | bitmask\_value | both | client\_1\_0 | 00400000 | assigned | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-KEY-AGREEMENT-00000800 | Key Agreement | bitmask\_value | both | client\_1\_0 | 00000800 | assigned | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-MAC-GENERATE-00000080 | MAC Generate | bitmask\_value | both | client\_1\_0 | 00000080 | assigned | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-MAC-VERIFY-00000100 | MAC Verify | bitmask\_value | both | client\_1\_0 | 00000100 | assigned | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-RESERVED-00000040 | \(Reserved\) | bitmask\_value | both | client\_1\_0 | 00000040 | reserved | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-RESERVED-00000400 | \(Reserved\) | bitmask\_value | both | client\_1\_0 | 00000400 | reserved | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-RESERVED-00004000 | \(Reserved\) | bitmask\_value | both | client\_1\_0 | 00004000 | reserved | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-RESERVED-00008000 | \(Reserved\) | bitmask\_value | both | client\_1\_0 | 00008000 | reserved | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-RESERVED-00010000 | \(Reserved\) | bitmask\_value | both | client\_1\_0 | 00010000 | reserved | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-RESERVED-00020000 | \(Reserved\) | bitmask\_value | both | client\_1\_0 | 00020000 | reserved | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-RESERVED-00040000 | \(Reserved\) | bitmask\_value | both | client\_1\_0 | 00040000 | reserved | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-RESERVED-00080000 | \(Reserved\) | bitmask\_value | both | client\_1\_0 | 00080000 | reserved | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-SIGN-00000001 | Sign | bitmask\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-UNRESTRICTED-00200000 | Unrestricted | bitmask\_value | both | client\_1\_0 | 00200000 | assigned | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-UNWRAP-KEY-00000020 | Unwrap Key | bitmask\_value | both | client\_1\_0 | 00000020 | assigned | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-VERIFY-00000002 | Verify | bitmask\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-WRAP-KEY-00000010 | Wrap Key | bitmask\_value | both | client\_1\_0 | 00000010 | assigned | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-CONTAINER-00000080 | Container | bitmask\_value | both | client\_1\_0 | 00000080 | assigned | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-EXTENSIONS-XXXXXXX0 | Extensions | bitmask\_value | both | client\_1\_0 | XXXXXXX0 | extension | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-HARDWARE-00000002 | Hardware | bitmask\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-HYPERVISOR-00000020 | Hypervisor | bitmask\_value | both | client\_1\_0 | 00000020 | assigned | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-OFF-PREMISES-00000200 | Off Premises | bitmask\_value | both | client\_1\_0 | 00000200 | assigned | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-OFF-SYSTEM-00000010 | Off System | bitmask\_value | both | client\_1\_0 | 00000010 | assigned | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-ON-PREMISES-00000100 | On Premises | bitmask\_value | both | client\_1\_0 | 00000100 | assigned | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-ON-PROCESSOR-00000004 | On Processor | bitmask\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-ON-SYSTEM-00000008 | On System | bitmask\_value | both | client\_1\_0 | 00000008 | assigned | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-OPERATING-SYSTEM-00000040 | Operating System | bitmask\_value | both | client\_1\_0 | 00000040 | assigned | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-OUTSOURCED-00000800 | Outsourced | bitmask\_value | both | client\_1\_0 | 00000800 | assigned | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-SAME-JURISDICTION-00002000 | Same Jurisdiction | bitmask\_value | both | client\_1\_0 | 00002000 | assigned | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-SELF-MANAGED-00000400 | Self Managed | bitmask\_value | both | client\_1\_0 | 00000400 | assigned | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-SOFTWARE-00000001 | Software | bitmask\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-VALIDATED-00001000 | Validated | bitmask\_value | both | client\_1\_0 | 00001000 | assigned | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-STORAGE-STATUS-MASK-ARCHIVAL-STORAGE-00000002 | Archival storage | bitmask\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §12.3 |
| KMIPKIT-ELEM-BITMASK-VALUE-STORAGE-STATUS-MASK-DESTROYED-STORAGE-00000004 | Destroyed storage | bitmask\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §12.3 |
| KMIPKIT-ELEM-BITMASK-VALUE-STORAGE-STATUS-MASK-EXTENSIONS-XXXXXXX0 | Extensions | bitmask\_value | both | client\_1\_0 | XXXXXXX0 | extension | KMIPKIT-SRC-spec §12.3 |
| KMIPKIT-ELEM-BITMASK-VALUE-STORAGE-STATUS-MASK-ON-LINE-STORAGE-00000001 | On-line storage | bitmask\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §12.3 |
| KMIPKIT-ELEM-CREDENTIAL-ATTESTATION | Attestation | credential | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-CREDENTIAL-CREDENTIAL | Credential | credential | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-CREDENTIAL-DEVICE | Device | credential | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-CREDENTIAL-HASHED-PASSWORD | Hashed Password | credential | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-CREDENTIAL-ONE-TIME-PASSWORD | One Time Password | credential | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-CREDENTIAL-TICKET | Ticket | credential | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-CREDENTIAL-USERNAME-AND-PASSWORD | Username and Password | credential | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-DATA-TYPE-BIG-INTEGER | Big Integer | data\_type | both | client\_1\_0 | 00000004 |  | KMIPKIT-SRC-spec §1.5, KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-DATA-TYPE-BOOLEAN | Boolean | data\_type | both | client\_1\_0 | 00000006 |  | KMIPKIT-SRC-spec §1.5, KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-DATA-TYPE-BYTE-STRING | Byte String | data\_type | both | client\_1\_0 | 00000008 |  | KMIPKIT-SRC-spec §1.5, KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-DATA-TYPE-DATE-TIME | Date Time | data\_type | both | client\_1\_0 | 00000009 |  | KMIPKIT-SRC-spec §1.5, KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-DATA-TYPE-DATE-TIME-EXTENDED | Date Time Extended | data\_type | both | client\_1\_0 | 0000000B |  | KMIPKIT-SRC-spec §1.5, KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-DATA-TYPE-ENUMERATION | Enumeration | data\_type | both | client\_1\_0 | 00000005 |  | KMIPKIT-SRC-spec §1.5, KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-DATA-TYPE-INTEGER | Integer | data\_type | both | client\_1\_0 | 00000002 |  | KMIPKIT-SRC-spec §1.5, KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-DATA-TYPE-INTERVAL | Interval | data\_type | both | client\_1\_0 | 0000000A |  | KMIPKIT-SRC-spec §1.5, KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-DATA-TYPE-LONG-INTEGER | Long Integer | data\_type | both | client\_1\_0 | 00000003 |  | KMIPKIT-SRC-spec §1.5, KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-DATA-TYPE-STRUCTURE | Structure | data\_type | both | client\_1\_0 | 00000001 |  | KMIPKIT-SRC-spec §1.5, KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-DATA-TYPE-TEXT-STRING | Text String | data\_type | both | client\_1\_0 | 00000007 |  | KMIPKIT-SRC-spec §1.5, KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-ENUM-VALUE-ADJUSTMENT-TYPE-DECREMENT-00000002 | Decrement | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.1 |
| KMIPKIT-ELEM-ENUM-VALUE-ADJUSTMENT-TYPE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.1 |
| KMIPKIT-ELEM-ENUM-VALUE-ADJUSTMENT-TYPE-INCREMENT-00000001 | Increment | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.1 |
| KMIPKIT-ELEM-ENUM-VALUE-ADJUSTMENT-TYPE-NEGATE-00000003 | Negate | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.1 |
| KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-DNS-NAME-00000005 | DNS Name | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.2 |
| KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-EMAIL-ADDRESS-00000004 | Email Address | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.2 |
| KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.2 |
| KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-IP-ADDRESS-00000007 | IP Address | enumeration\_value | both | client\_1\_0 | 00000007 | assigned | KMIPKIT-SRC-spec §11.2 |
| KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-OBJECT-SERIAL-NUMBER-00000003 | Object Serial Number | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.2 |
| KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-UNINTERPRETED-TEXT-STRING-00000001 | Uninterpreted Text String | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.2 |
| KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-URI-00000002 | URI | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.2 |
| KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-X-500-DISTINGUISHED-NAME-00000006 | X.500 Distinguished Name | enumeration\_value | both | client\_1\_0 | 00000006 | assigned | KMIPKIT-SRC-spec §11.2 |
| KMIPKIT-ELEM-ENUM-VALUE-ASYNCHRONOUS-INDICATOR-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.3 |
| KMIPKIT-ELEM-ENUM-VALUE-ASYNCHRONOUS-INDICATOR-MANDATORY-00000001 | Mandatory | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.3 |
| KMIPKIT-ELEM-ENUM-VALUE-ASYNCHRONOUS-INDICATOR-OPTIONAL-00000002 | Optional | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.3 |
| KMIPKIT-ELEM-ENUM-VALUE-ASYNCHRONOUS-INDICATOR-PROHIBITED-00000003 | Prohibited | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.3 |
| KMIPKIT-ELEM-ENUM-VALUE-ATTESTATION-TYPE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.4 |
| KMIPKIT-ELEM-ENUM-VALUE-ATTESTATION-TYPE-SAML-ASSERTION-00000003 | SAML Assertion | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.4 |
| KMIPKIT-ELEM-ENUM-VALUE-ATTESTATION-TYPE-TCG-INTEGRITY-REPORT-00000002 | TCG Integrity Report | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.4 |
| KMIPKIT-ELEM-ENUM-VALUE-ATTESTATION-TYPE-TPM-QUOTE-00000001 | TPM Quote | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.4 |
| KMIPKIT-ELEM-ENUM-VALUE-BATCH-ERROR-CONTINUATION-OPTION-CONTINUE-00000001 | Continue | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.5 |
| KMIPKIT-ELEM-ENUM-VALUE-BATCH-ERROR-CONTINUATION-OPTION-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.5 |
| KMIPKIT-ELEM-ENUM-VALUE-BATCH-ERROR-CONTINUATION-OPTION-STOP-00000002 | Stop | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.5 |
| KMIPKIT-ELEM-ENUM-VALUE-BATCH-ERROR-CONTINUATION-OPTION-UNDO-00000003 | Undo | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.5 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-AEAD-00000012 | AEAD | enumeration\_value | both | client\_1\_0 | 00000012 | assigned | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-AESKEYWRAPPADDING-0000000C | AESKeyWrapPadding | enumeration\_value | both | client\_1\_0 | 0000000C | assigned | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-CBC-00000001 | CBC | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-CBC-MAC-0000000A | CBC-MAC | enumeration\_value | both | client\_1\_0 | 0000000A | assigned | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-CCM-00000008 | CCM | enumeration\_value | both | client\_1\_0 | 00000008 | assigned | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-CFB-00000004 | CFB | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-CMAC-00000007 | CMAC | enumeration\_value | both | client\_1\_0 | 00000007 | assigned | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-CTR-00000006 | CTR | enumeration\_value | both | client\_1\_0 | 00000006 | assigned | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-ECB-00000002 | ECB | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-GCM-00000009 | GCM | enumeration\_value | both | client\_1\_0 | 00000009 | assigned | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-NISTKEYWRAP-0000000D | NISTKeyWrap | enumeration\_value | both | client\_1\_0 | 0000000D | assigned | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-OFB-00000005 | OFB | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-PCBC-00000003 | PCBC | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-X9-102-AESKW-0000000E | X9.102 AESKW | enumeration\_value | both | client\_1\_0 | 0000000E | assigned | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-X9-102-AKW1-00000010 | X9.102 AKW1 | enumeration\_value | both | client\_1\_0 | 00000010 | assigned | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-X9-102-AKW2-00000011 | X9.102 AKW2 | enumeration\_value | both | client\_1\_0 | 00000011 | assigned | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-X9-102-TDKW-0000000F | X9.102 TDKW | enumeration\_value | both | client\_1\_0 | 0000000F | assigned | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-XTS-0000000B | XTS | enumeration\_value | both | client\_1\_0 | 0000000B | assigned | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-CANCELLATION-RESULT-CANCELED-00000001 | Canceled | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.7 |
| KMIPKIT-ELEM-ENUM-VALUE-CANCELLATION-RESULT-COMPLETED-00000003 | Completed | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.7 |
| KMIPKIT-ELEM-ENUM-VALUE-CANCELLATION-RESULT-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.7 |
| KMIPKIT-ELEM-ENUM-VALUE-CANCELLATION-RESULT-FAILED-00000004 | Failed | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.7 |
| KMIPKIT-ELEM-ENUM-VALUE-CANCELLATION-RESULT-UNABLE-TO-CANCEL-00000002 | Unable to Cancel | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.7 |
| KMIPKIT-ELEM-ENUM-VALUE-CANCELLATION-RESULT-UNAVAILABLE-00000005 | Unavailable | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.7 |
| KMIPKIT-ELEM-ENUM-VALUE-CERTIFICATE-REQUEST-TYPE-CRMF-00000001 | CRMF | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.8 |
| KMIPKIT-ELEM-ENUM-VALUE-CERTIFICATE-REQUEST-TYPE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.8 |
| KMIPKIT-ELEM-ENUM-VALUE-CERTIFICATE-REQUEST-TYPE-PEM-00000003 | PEM | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.8 |
| KMIPKIT-ELEM-ENUM-VALUE-CERTIFICATE-REQUEST-TYPE-PKCS-10-00000002 | PKCS#10 | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.8 |
| KMIPKIT-ELEM-ENUM-VALUE-CERTIFICATE-REQUEST-TYPE-RESERVED-00000004 | \(Reserved\) | enumeration\_value | both | client\_1\_0 | 00000004 | reserved | KMIPKIT-SRC-spec §11.8 |
| KMIPKIT-ELEM-ENUM-VALUE-CERTIFICATE-TYPE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.9 |
| KMIPKIT-ELEM-ENUM-VALUE-CERTIFICATE-TYPE-PGP-00000002 | PGP | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.9 |
| KMIPKIT-ELEM-ENUM-VALUE-CERTIFICATE-TYPE-X-509-00000001 | X.509 | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.9 |
| KMIPKIT-ELEM-ENUM-VALUE-CLIENT-REGISTRATION-METHOD-CLIENT-GENERATED-00000004 | Client Generated | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.10 |
| KMIPKIT-ELEM-ENUM-VALUE-CLIENT-REGISTRATION-METHOD-CLIENT-REGISTERED-00000005 | Client Registered | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.10 |
| KMIPKIT-ELEM-ENUM-VALUE-CLIENT-REGISTRATION-METHOD-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.10 |
| KMIPKIT-ELEM-ENUM-VALUE-CLIENT-REGISTRATION-METHOD-SERVER-ON-DEMAND-00000003 | Server On-Demand | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.10 |
| KMIPKIT-ELEM-ENUM-VALUE-CLIENT-REGISTRATION-METHOD-SERVER-PRE-GENERATED-00000002 | Server Pre-Generated | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.10 |
| KMIPKIT-ELEM-ENUM-VALUE-CLIENT-REGISTRATION-METHOD-UNSPECIFIED-00000001 | Unspecified | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.10 |
| KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-ATTESTATION-00000003 | Attestation | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.11 |
| KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-DEVICE-00000002 | Device | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.11 |
| KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.11 |
| KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-HASHED-PASSWORD-00000005 | Hashed Password | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.11 |
| KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-ONE-TIME-PASSWORD-00000004 | One Time Password | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.11 |
| KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-TICKET-00000006 | Ticket | enumeration\_value | both | client\_1\_0 | 00000006 | assigned | KMIPKIT-SRC-spec §11.11 |
| KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-USERNAME-AND-PASSWORD-00000001 | Username and Password | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.11 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-3DES-00000002 | 3DES | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-AES-00000003 | AES | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ARIA-00000029 | ARIA | enumeration\_value | both | client\_1\_0 | 00000029 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-BLOWFISH-00000010 | Blowfish | enumeration\_value | both | client\_1\_0 | 00000010 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-CAMELLIA-00000011 | Camellia | enumeration\_value | both | client\_1\_0 | 00000011 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-CAST5-00000012 | CAST5 | enumeration\_value | both | client\_1\_0 | 00000012 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-CHACHA20-0000001C | ChaCha20 | enumeration\_value | both | client\_1\_0 | 0000001C | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-CHACHA20POLY1305-0000001E | ChaCha20Poly1305 | enumeration\_value | both | client\_1\_0 | 0000001E | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-DES-00000001 | DES | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-DH-0000000D | DH | enumeration\_value | both | client\_1\_0 | 0000000D | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-DSA-00000005 | DSA | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-EC-0000001A | EC | enumeration\_value | both | client\_1\_0 | 0000001A | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ECDH-0000000E | ECDH | enumeration\_value | both | client\_1\_0 | 0000000E | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ECDSA-00000006 | ECDSA | enumeration\_value | both | client\_1\_0 | 00000006 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ECMQV-0000000F | ECMQV | enumeration\_value | both | client\_1\_0 | 0000000F | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ED25519-00000037 | Ed25519 | enumeration\_value | both | client\_1\_0 | 00000037 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ED448-00000038 | Ed448 | enumeration\_value | both | client\_1\_0 | 00000038 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-GOST-28147-89-00000031 | GOST 28147-89 | enumeration\_value | both | client\_1\_0 | 00000031 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-GOST-R-34-10-2012-0000002E | GOST R 34.10-2012 | enumeration\_value | both | client\_1\_0 | 0000002E | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-GOST-R-34-11-2012-0000002F | GOST R 34.11-2012 | enumeration\_value | both | client\_1\_0 | 0000002F | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-GOST-R-34-13-2015-00000030 | GOST R 34.13-2015 | enumeration\_value | both | client\_1\_0 | 00000030 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-MD5-0000000C | HMAC-MD5 | enumeration\_value | both | client\_1\_0 | 0000000C | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA1-00000007 | HMAC-SHA1 | enumeration\_value | both | client\_1\_0 | 00000007 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA224-00000008 | HMAC-SHA224 | enumeration\_value | both | client\_1\_0 | 00000008 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA256-00000009 | HMAC-SHA256 | enumeration\_value | both | client\_1\_0 | 00000009 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA3-224-00000023 | HMAC-SHA3-224 | enumeration\_value | both | client\_1\_0 | 00000023 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA3-256-00000024 | HMAC-SHA3-256 | enumeration\_value | both | client\_1\_0 | 00000024 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA3-384-00000025 | HMAC-SHA3-384 | enumeration\_value | both | client\_1\_0 | 00000025 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA3-512-00000026 | HMAC-SHA3-512 | enumeration\_value | both | client\_1\_0 | 00000026 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA384-0000000A | HMAC-SHA384 | enumeration\_value | both | client\_1\_0 | 0000000A | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA512-0000000B | HMAC-SHA512 | enumeration\_value | both | client\_1\_0 | 0000000B | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-IDEA-00000013 | IDEA | enumeration\_value | both | client\_1\_0 | 00000013 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-MARS-00000014 | MARS | enumeration\_value | both | client\_1\_0 | 00000014 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-MCELIECE-00000034 | McEliece | enumeration\_value | both | client\_1\_0 | 00000034 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-MCELIECE-6960119-00000035 | McEliece-6960119 | enumeration\_value | both | client\_1\_0 | 00000035 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-MCELIECE-8192128-00000036 | McEliece-8192128 | enumeration\_value | both | client\_1\_0 | 00000036 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ONE-TIME-PAD-0000001B | One Time Pad | enumeration\_value | both | client\_1\_0 | 0000001B | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-POLY1305-0000001D | Poly1305 | enumeration\_value | both | client\_1\_0 | 0000001D | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-RC2-00000015 | RC2 | enumeration\_value | both | client\_1\_0 | 00000015 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-RC4-00000016 | RC4 | enumeration\_value | both | client\_1\_0 | 00000016 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-RC5-00000017 | RC5 | enumeration\_value | both | client\_1\_0 | 00000017 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-RSA-00000004 | RSA | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SEED-0000002A | SEED | enumeration\_value | both | client\_1\_0 | 0000002A | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SHA3-224-0000001F | SHA3-224 | enumeration\_value | both | client\_1\_0 | 0000001F | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SHA3-256-00000020 | SHA3-256 | enumeration\_value | both | client\_1\_0 | 00000020 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SHA3-384-00000021 | SHA3-384 | enumeration\_value | both | client\_1\_0 | 00000021 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SHA3-512-00000022 | SHA3-512 | enumeration\_value | both | client\_1\_0 | 00000022 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SHAKE-128-00000027 | SHAKE-128 | enumeration\_value | both | client\_1\_0 | 00000027 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SHAKE-256-00000028 | SHAKE-256 | enumeration\_value | both | client\_1\_0 | 00000028 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SKIPJACK-00000018 | SKIPJACK | enumeration\_value | both | client\_1\_0 | 00000018 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SM2-0000002B | SM2 | enumeration\_value | both | client\_1\_0 | 0000002B | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SM3-0000002C | SM3 | enumeration\_value | both | client\_1\_0 | 0000002C | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SM4-0000002D | SM4 | enumeration\_value | both | client\_1\_0 | 0000002D | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SPHINCS-256-00000033 | SPHINCS-256 | enumeration\_value | both | client\_1\_0 | 00000033 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-TWOFISH-00000019 | Twofish | enumeration\_value | both | client\_1\_0 | 00000019 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-XMSS-00000032 | XMSS | enumeration\_value | both | client\_1\_0 | 00000032 | assigned | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-DATA-DECRYPT-00000001 | Decrypt | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.13 |
| KMIPKIT-ELEM-ENUM-VALUE-DATA-ENCRYPT-00000002 | Encrypt | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.13 |
| KMIPKIT-ELEM-ENUM-VALUE-DATA-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.13 |
| KMIPKIT-ELEM-ENUM-VALUE-DATA-HASH-00000003 | Hash | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.13 |
| KMIPKIT-ELEM-ENUM-VALUE-DATA-MAC-MAC-DATA-00000004 | MAC MAC Data | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.13 |
| KMIPKIT-ELEM-ENUM-VALUE-DATA-RNG-RETRIEVE-00000005 | RNG Retrieve | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.13 |
| KMIPKIT-ELEM-ENUM-VALUE-DATA-SIGN-SIGNATURE-DATA-00000006 | Sign Signature Data | enumeration\_value | both | client\_1\_0 | 00000006 | assigned | KMIPKIT-SRC-spec §11.13 |
| KMIPKIT-ELEM-ENUM-VALUE-DATA-SIGNATURE-VERIFY-00000007 | Signature Verify | enumeration\_value | both | client\_1\_0 | 00000007 | assigned | KMIPKIT-SRC-spec §11.13 |
| KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-ASYMMETRIC-KEY-00000008 | Asymmetric Key | enumeration\_value | both | client\_1\_0 | 00000008 | assigned | KMIPKIT-SRC-spec §11.14 |
| KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-AWS-SIGNATURE-VERSION-4-00000009 | AWS Signature Version 4 | enumeration\_value | both | client\_1\_0 | 00000009 | assigned | KMIPKIT-SRC-spec §11.14 |
| KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-ENCRYPT-00000004 | ENCRYPT | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.14 |
| KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.14 |
| KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-HASH-00000002 | HASH | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.14 |
| KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-HKDF-0000000A | HKDF | enumeration\_value | both | client\_1\_0 | 0000000A | assigned | KMIPKIT-SRC-spec §11.14 |
| KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-HMAC-00000003 | HMAC | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.14 |
| KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-NIST800-108-C-00000005 | NIST800-108-C | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.14 |
| KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-NIST800-108-DPI-00000007 | NIST800-108-DPI | enumeration\_value | both | client\_1\_0 | 00000007 | assigned | KMIPKIT-SRC-spec §11.14 |
| KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-NIST800-108-F-00000006 | NIST800-108-F | enumeration\_value | both | client\_1\_0 | 00000006 | assigned | KMIPKIT-SRC-spec §11.14 |
| KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-PBKDF2-00000001 | PBKDF2 | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.14 |
| KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-DELETED-00000006 | Deleted | enumeration\_value | both | client\_1\_0 | 00000006 | assigned | KMIPKIT-SRC-spec §11.15 |
| KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.15 |
| KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-KEY-MATERIAL-DELETED-00000002 | Key Material Deleted | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.15 |
| KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-KEY-MATERIAL-SHREDDED-00000003 | Key Material Shredded | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.15 |
| KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-META-DATA-DELETED-00000004 | Meta Data Deleted | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.15 |
| KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-META-DATA-SHREDDED-00000005 | Meta Data Shredded | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.15 |
| KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-SHREDDED-00000007 | Shredded | enumeration\_value | both | client\_1\_0 | 00000007 | assigned | KMIPKIT-SRC-spec §11.15 |
| KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-UNSPECIFIED-00000001 | Unspecified | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.15 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-DSA-WITH-SHA-1-00000009 | DSA with SHA-1 | enumeration\_value | both | client\_1\_0 | 00000009 | assigned | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-DSA-WITH-SHA224-0000000A | DSA with SHA224 | enumeration\_value | both | client\_1\_0 | 0000000A | assigned | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-DSA-WITH-SHA256-0000000B | DSA with SHA256 | enumeration\_value | both | client\_1\_0 | 0000000B | assigned | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-ECDSA-WITH-SHA-1-0000000C | ECDSA with SHA-1 | enumeration\_value | both | client\_1\_0 | 0000000C | assigned | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-ECDSA-WITH-SHA224-0000000D | ECDSA with SHA224 | enumeration\_value | both | client\_1\_0 | 0000000D | assigned | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-ECDSA-WITH-SHA256-0000000E | ECDSA with SHA256 | enumeration\_value | both | client\_1\_0 | 0000000E | assigned | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-ECDSA-WITH-SHA384-0000000F | ECDSA with SHA384 | enumeration\_value | both | client\_1\_0 | 0000000F | assigned | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-ECDSA-WITH-SHA512-00000010 | ECDSA with SHA512 | enumeration\_value | both | client\_1\_0 | 00000010 | assigned | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-MD2-WITH-RSA-ENCRYPTION-00000001 | MD2 with RSA Encryption | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-MD5-WITH-RSA-ENCRYPTION-00000002 | MD5 with RSA Encryption | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-RSASSA-PSS-00000008 | RSASSA-PSS | enumeration\_value | both | client\_1\_0 | 00000008 | assigned | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA-1-WITH-RSA-ENCRYPTION-00000003 | SHA-1 with RSA Encryption | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA-224-WITH-RSA-ENCRYPTION-00000004 | SHA-224 with RSA Encryption | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA-256-WITH-RSA-ENCRYPTION-00000005 | SHA-256 with RSA Encryption | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA-384-WITH-RSA-ENCRYPTION-00000006 | SHA-384 with RSA Encryption | enumeration\_value | both | client\_1\_0 | 00000006 | assigned | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA-512-WITH-RSA-ENCRYPTION-00000007 | SHA-512 with RSA Encryption | enumeration\_value | both | client\_1\_0 | 00000007 | assigned | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA3-256-WITH-RSA-ENCRYPTION-00000011 | SHA3-256 with RSA Encryption | enumeration\_value | both | client\_1\_0 | 00000011 | assigned | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA3-384-WITH-RSA-ENCRYPTION-00000012 | SHA3-384 with RSA Encryption | enumeration\_value | both | client\_1\_0 | 00000012 | assigned | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA3-512-WITH-RSA-ENCRYPTION-00000013 | SHA3-512 with RSA Encryption | enumeration\_value | both | client\_1\_0 | 00000013 | assigned | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DRBG-ALGORITHM-CTR-00000005 | CTR | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.17 |
| KMIPKIT-ELEM-ENUM-VALUE-DRBG-ALGORITHM-DUAL-EC-00000002 | Dual-EC | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.17 |
| KMIPKIT-ELEM-ENUM-VALUE-DRBG-ALGORITHM-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.17 |
| KMIPKIT-ELEM-ENUM-VALUE-DRBG-ALGORITHM-HASH-00000003 | Hash | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.17 |
| KMIPKIT-ELEM-ENUM-VALUE-DRBG-ALGORITHM-HMAC-00000004 | HMAC | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.17 |
| KMIPKIT-ELEM-ENUM-VALUE-DRBG-ALGORITHM-UNSPECIFIED-00000001 | Unspecified | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.17 |
| KMIPKIT-ELEM-ENUM-VALUE-ENCODING-OPTION-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.18 |
| KMIPKIT-ELEM-ENUM-VALUE-ENCODING-OPTION-NO-ENCODING-00000001 | No Encoding | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.18 |
| KMIPKIT-ELEM-ENUM-VALUE-ENCODING-OPTION-TTLV-ENCODING-00000002 | TTLV Encoding | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.18 |
| KMIPKIT-ELEM-ENUM-VALUE-ENDPOINT-ROLE-CLIENT-00000001 | Client | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.19 |
| KMIPKIT-ELEM-ENUM-VALUE-ENDPOINT-ROLE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.19 |
| KMIPKIT-ELEM-ENUM-VALUE-ENDPOINT-ROLE-SERVER-00000002 | Server | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.19 |
| KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.20 |
| KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-GP-X-CHANGE-NOTICE-00000003 | GP x-Change Notice | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.20 |
| KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-GP-X-ORIGINAL-00000002 | GP x-Original | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.20 |
| KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-K-CHANGE-NOTICE-00000007 | k-Change Notice | enumeration\_value | both | client\_1\_0 | 00000007 | assigned | KMIPKIT-SRC-spec §11.20 |
| KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-K-ORIGINAL-00000006 | k-Original | enumeration\_value | both | client\_1\_0 | 00000006 | assigned | KMIPKIT-SRC-spec §11.20 |
| KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-UNSPECIFIED-00000001 | Unspecified | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.20 |
| KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-X-CHANGE-NOTICE-00000005 | x-Change Notice | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.20 |
| KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-X-ORIGINAL-00000004 | x-Original | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.20 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-MD2-00000001 | MD2 | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-MD4-00000002 | MD4 | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-MD5-00000003 | MD5 | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-RIPEMD-160-00000009 | RIPEMD-160 | enumeration\_value | both | client\_1\_0 | 00000009 | assigned | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-1-00000004 | SHA-1 | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-224-00000005 | SHA-224 | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-256-00000006 | SHA-256 | enumeration\_value | both | client\_1\_0 | 00000006 | assigned | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-384-00000007 | SHA-384 | enumeration\_value | both | client\_1\_0 | 00000007 | assigned | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-512-00000008 | SHA-512 | enumeration\_value | both | client\_1\_0 | 00000008 | assigned | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-512-224-0000000C | SHA-512/224 | enumeration\_value | both | client\_1\_0 | 0000000C | assigned | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-512-256-0000000D | SHA-512/256 | enumeration\_value | both | client\_1\_0 | 0000000D | assigned | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA3-224-0000000E | SHA3-224 | enumeration\_value | both | client\_1\_0 | 0000000E | assigned | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA3-256-0000000F | SHA3-256 | enumeration\_value | both | client\_1\_0 | 0000000F | assigned | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA3-384-00000010 | SHA3-384 | enumeration\_value | both | client\_1\_0 | 00000010 | assigned | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA3-512-00000011 | SHA3-512 | enumeration\_value | both | client\_1\_0 | 00000011 | assigned | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-TIGER-0000000A | Tiger | enumeration\_value | both | client\_1\_0 | 0000000A | assigned | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-WHIRLPOOL-0000000B | Whirlpool | enumeration\_value | both | client\_1\_0 | 0000000B | assigned | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-INTEROP-FUNCTION-BEGIN-00000001 | Begin | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.22 |
| KMIPKIT-ELEM-ENUM-VALUE-INTEROP-FUNCTION-END-00000002 | End | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.22 |
| KMIPKIT-ELEM-ENUM-VALUE-INTEROP-FUNCTION-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.22 |
| KMIPKIT-ELEM-ENUM-VALUE-INTEROP-FUNCTION-RESET-00000003 | Reset | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.22 |
| KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-BIG-INTEGER-00000004 | Big Integer | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-BOOLEAN-00000006 | Boolean | enumeration\_value | both | client\_1\_0 | 00000006 | assigned | KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-BYTE-STRING-00000008 | Byte String | enumeration\_value | both | client\_1\_0 | 00000008 | assigned | KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-DATE-TIME-00000009 | Date Time | enumeration\_value | both | client\_1\_0 | 00000009 | assigned | KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-DATE-TIME-EXTENDED-0000000B | Date Time Extended | enumeration\_value | both | client\_1\_0 | 0000000B | assigned | KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-ENUMERATION-00000005 | Enumeration | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-INTEGER-00000002 | Integer | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-INTERVAL-0000000A | Interval | enumeration\_value | both | client\_1\_0 | 0000000A | assigned | KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-LONG-INTEGER-00000003 | Long Integer | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-STRUCTURE-00000001 | Structure | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-TEXT-STRING-00000007 | Text String | enumeration\_value | both | client\_1\_0 | 00000007 | assigned | KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-COMPRESSION-TYPE-EC-PUBLIC-KEY-TYPE-UNCOMPRESSED-00000001 | EC Public Key Type Uncompressed | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.24 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-COMPRESSION-TYPE-EC-PUBLIC-KEY-TYPE-X9-62-COMPRESSED-CHAR2-00000003 | EC Public Key Type X9.62 Compressed Char2 | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.24 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-COMPRESSION-TYPE-EC-PUBLIC-KEY-TYPE-X9-62-COMPRESSED-PRIME-00000002 | EC Public Key Type X9.62 Compressed Prime | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.24 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-COMPRESSION-TYPE-EC-PUBLIC-KEY-TYPE-X9-62-HYBRID-00000004 | EC Public Key Type X9.62 Hybrid | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.24 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-COMPRESSION-TYPE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.24 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-ECPRIVATEKEY-00000006 | ECPrivateKey | enumeration\_value | both | client\_1\_0 | 00000006 | assigned | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-OPAQUE-00000002 | Opaque | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-PKCS-1-00000003 | PKCS#1 | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-PKCS-10-00000017 | PKCS#10 | enumeration\_value | both | client\_1\_0 | 00000017 | assigned | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-PKCS-12-00000016 | PKCS#12 | enumeration\_value | both | client\_1\_0 | 00000016 | assigned | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-PKCS-8-00000004 | PKCS#8 | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-RAW-00000001 | Raw | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-RESERVED-0000000E | \(Reserved\) | enumeration\_value | both | client\_1\_0 | 0000000E | reserved | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-RESERVED-0000000F | \(Reserved\) | enumeration\_value | both | client\_1\_0 | 0000000F | reserved | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-RESERVED-00000010 | \(Reserved\) | enumeration\_value | both | client\_1\_0 | 00000010 | reserved | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-RESERVED-00000011 | \(Reserved\) | enumeration\_value | both | client\_1\_0 | 00000011 | reserved | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-RESERVED-00000012 | \(Reserved\) | enumeration\_value | both | client\_1\_0 | 00000012 | reserved | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-RESERVED-00000013 | \(Reserved\) | enumeration\_value | both | client\_1\_0 | 00000013 | reserved | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-DH-PRIVATE-KEY-0000000C | Transparent DH Private Key | enumeration\_value | both | client\_1\_0 | 0000000C | assigned | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-DH-PUBLIC-KEY-0000000D | Transparent DH Public Key | enumeration\_value | both | client\_1\_0 | 0000000D | assigned | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-DSA-PRIVATE-KEY-00000008 | Transparent DSA Private Key | enumeration\_value | both | client\_1\_0 | 00000008 | assigned | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-DSA-PUBLIC-KEY-00000009 | Transparent DSA Public Key | enumeration\_value | both | client\_1\_0 | 00000009 | assigned | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-EC-PRIVATE-KEY-00000014 | Transparent EC Private Key | enumeration\_value | both | client\_1\_0 | 00000014 | assigned | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-EC-PUBLIC-KEY-00000015 | Transparent EC Public Key | enumeration\_value | both | client\_1\_0 | 00000015 | assigned | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-RSA-PRIVATE-KEY-0000000A | Transparent RSA Private Key | enumeration\_value | both | client\_1\_0 | 0000000A | assigned | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-RSA-PUBLIC-KEY-0000000B | Transparent RSA Public Key | enumeration\_value | both | client\_1\_0 | 0000000B | assigned | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-SYMMETRIC-KEY-00000007 | Transparent Symmetric Key | enumeration\_value | both | client\_1\_0 | 00000007 | assigned | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-X-509-00000005 | X.509 | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-BDK-00000001 | BDK | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-CVK-00000002 | CVK | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-DEK-00000003 | DEK | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-DUKPT-00000016 | DUKPT | enumeration\_value | both | client\_1\_0 | 00000016 | assigned | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-IV-00000017 | IV | enumeration\_value | both | client\_1\_0 | 00000017 | assigned | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-KEK-0000000B | KEK | enumeration\_value | both | client\_1\_0 | 0000000B | assigned | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MAC16609-0000000C | MAC16609 | enumeration\_value | both | client\_1\_0 | 0000000C | assigned | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MAC97971-0000000D | MAC97971 | enumeration\_value | both | client\_1\_0 | 0000000D | assigned | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MAC97972-0000000E | MAC97972 | enumeration\_value | both | client\_1\_0 | 0000000E | assigned | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MAC97973-0000000F | MAC97973 | enumeration\_value | both | client\_1\_0 | 0000000F | assigned | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MAC97974-00000010 | MAC97974 | enumeration\_value | both | client\_1\_0 | 00000010 | assigned | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MAC97975-00000011 | MAC97975 | enumeration\_value | both | client\_1\_0 | 00000011 | assigned | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKAC-00000004 | MKAC | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKCP-00000009 | MKCP | enumeration\_value | both | client\_1\_0 | 00000009 | assigned | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKDAC-00000007 | MKDAC | enumeration\_value | both | client\_1\_0 | 00000007 | assigned | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKDN-00000008 | MKDN | enumeration\_value | both | client\_1\_0 | 00000008 | assigned | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKOTH-0000000A | MKOTH | enumeration\_value | both | client\_1\_0 | 0000000A | assigned | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKSMC-00000005 | MKSMC | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKSMI-00000006 | MKSMI | enumeration\_value | both | client\_1\_0 | 00000006 | assigned | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-PVKIBM-00000013 | PVKIBM | enumeration\_value | both | client\_1\_0 | 00000013 | assigned | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-PVKOTH-00000015 | PVKOTH | enumeration\_value | both | client\_1\_0 | 00000015 | assigned | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-PVKPVV-00000014 | PVKPVV | enumeration\_value | both | client\_1\_0 | 00000014 | assigned | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-TRKBK-00000018 | TRKBK | enumeration\_value | both | client\_1\_0 | 00000018 | assigned | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-ZPK-00000012 | ZPK | enumeration\_value | both | client\_1\_0 | 00000012 | assigned | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-VALUE-LOCATION-TYPE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.27 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-VALUE-LOCATION-TYPE-UNINTERPRETED-TEXT-STRING-00000001 | Uninterpreted Text String | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.27 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-VALUE-LOCATION-TYPE-URI-00000002 | URI | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.27 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-WRAP-TYPE-AS-REGISTERED-00000002 | As Registered | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.29 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-WRAP-TYPE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.29 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-WRAP-TYPE-NOT-WRAPPED-00000001 | Not Wrapped | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.29 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-CERTIFICATE-LINK-00000101 | Certificate Link | enumeration\_value | both | client\_1\_0 | 00000101 | assigned | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-CHILD-LINK-00000109 | Child Link | enumeration\_value | both | client\_1\_0 | 00000109 | assigned | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-DERIVATION-BASE-OBJECT-LINK-00000104 | Derivation Base Object Link | enumeration\_value | both | client\_1\_0 | 00000104 | assigned | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-DERIVED-KEY-LINK-00000105 | Derived Key Link | enumeration\_value | both | client\_1\_0 | 00000105 | assigned | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-NEXT-LINK-0000010B | Next Link | enumeration\_value | both | client\_1\_0 | 0000010B | assigned | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-PARENT-LINK-00000108 | Parent Link | enumeration\_value | both | client\_1\_0 | 00000108 | assigned | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-PKCS-12-CERTIFICATE-LINK-0000010C | PKCS#12 Certificate Link | enumeration\_value | both | client\_1\_0 | 0000010C | assigned | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-PKCS-12-PASSWORD-LINK-0000010D | PKCS#12 Password Link | enumeration\_value | both | client\_1\_0 | 0000010D | assigned | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-PREVIOUS-LINK-0000010A | Previous Link | enumeration\_value | both | client\_1\_0 | 0000010A | assigned | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-PRIVATE-KEY-LINK-00000103 | Private Key Link | enumeration\_value | both | client\_1\_0 | 00000103 | assigned | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-PUBLIC-KEY-LINK-00000102 | Public Key Link | enumeration\_value | both | client\_1\_0 | 00000102 | assigned | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-REPLACED-OBJECT-LINK-00000107 | Replaced Object Link | enumeration\_value | both | client\_1\_0 | 00000107 | assigned | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-REPLACEMENT-OBJECT-LINK-00000106 | Replacement Object Link | enumeration\_value | both | client\_1\_0 | 00000106 | assigned | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-WRAPPING-KEY-LINK-0000010E | Wrapping Key Link | enumeration\_value | both | client\_1\_0 | 0000010E | assigned | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-MASK-GENERATOR-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.30 |
| KMIPKIT-ELEM-ENUM-VALUE-MASK-GENERATOR-MFG1-00000001 | MFG1 | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.30 |
| KMIPKIT-ELEM-ENUM-VALUE-NAME-TYPE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.31 |
| KMIPKIT-ELEM-ENUM-VALUE-NAME-TYPE-UNINTERPRETED-TEXT-STRING-00000001 | Uninterpreted Text String | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.31 |
| KMIPKIT-ELEM-ENUM-VALUE-NAME-TYPE-URI-00000002 | URI | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.31 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PRIVATE-AUTHENTICATION-KEY-00000004 | Private authentication key | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PRIVATE-AUTHORIZATION-KEY-00000012 | Private authorization key | enumeration\_value | both | client\_1\_0 | 00000012 | assigned | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PRIVATE-EPHEMERAL-KEY-AGREEMENT-KEY-0000000F | Private ephemeral key agreement key | enumeration\_value | both | client\_1\_0 | 0000000F | assigned | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PRIVATE-KEY-TRANSPORT-KEY-0000000A | Private key transport key | enumeration\_value | both | client\_1\_0 | 0000000A | assigned | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PRIVATE-SIGNATURE-KEY-00000001 | Private signature key | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PRIVATE-STATIC-KEY-AGREEMENT-KEY-0000000D | Private static key agreement key | enumeration\_value | both | client\_1\_0 | 0000000D | assigned | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PUBLIC-AUTHENTICATION-KEY-00000005 | Public authentication key | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PUBLIC-AUTHORIZATION-KEY-00000013 | Public authorization key | enumeration\_value | both | client\_1\_0 | 00000013 | assigned | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PUBLIC-EPHEMERAL-KEY-AGREEMENT-KEY-00000010 | Public ephemeral key agreement key | enumeration\_value | both | client\_1\_0 | 00000010 | assigned | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PUBLIC-KEY-TRANSPORT-KEY-0000000B | Public key transport key | enumeration\_value | both | client\_1\_0 | 0000000B | assigned | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PUBLIC-SIGNATURE-VERIFICATION-KEY-00000002 | Public signature verification key | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PUBLIC-STATIC-KEY-AGREEMENT-KEY-0000000E | Public static key agreement key | enumeration\_value | both | client\_1\_0 | 0000000E | assigned | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-AUTHENTICATION-KEY-00000003 | Symmetric authentication key | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-AUTHORIZATION-KEY-00000011 | Symmetric authorization key | enumeration\_value | both | client\_1\_0 | 00000011 | assigned | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-DATA-ENCRYPTION-KEY-00000006 | Symmetric data encryption key | enumeration\_value | both | client\_1\_0 | 00000006 | assigned | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-KEY-AGREEMENT-KEY-0000000C | Symmetric key agreement key | enumeration\_value | both | client\_1\_0 | 0000000C | assigned | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-KEY-WRAPPING-KEY-00000007 | Symmetric key wrapping key | enumeration\_value | both | client\_1\_0 | 00000007 | assigned | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-MASTER-KEY-00000009 | Symmetric master key | enumeration\_value | both | client\_1\_0 | 00000009 | assigned | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-RANDOM-NUMBER-GENERATION-KEY-00000008 | Symmetric random number generation key | enumeration\_value | both | client\_1\_0 | 00000008 | assigned | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-GROUP-MEMBER-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.33 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-GROUP-MEMBER-GROUP-MEMBER-DEFAULT-00000002 | Group Member Default | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.33 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-GROUP-MEMBER-GROUP-MEMBER-FRESH-00000001 | Group Member Fresh | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.33 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-CERTIFICATE-00000001 | Certificate | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-CERTIFICATE-REQUEST-0000000A | Certificate Request | enumeration\_value | both | client\_1\_0 | 0000000A | assigned | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-OPAQUE-OBJECT-00000008 | Opaque Object | enumeration\_value | both | client\_1\_0 | 00000008 | assigned | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-PGP-KEY-00000009 | PGP Key | enumeration\_value | both | client\_1\_0 | 00000009 | assigned | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-PRIVATE-KEY-00000004 | Private Key | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-PUBLIC-KEY-00000003 | Public Key | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-RESERVED-00000006 | \(Reserved\) | enumeration\_value | both | client\_1\_0 | 00000006 | reserved | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-SECRET-DATA-00000007 | Secret Data | enumeration\_value | both | client\_1\_0 | 00000007 | assigned | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-SPLIT-KEY-00000005 | Split Key | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-SYMMETRIC-KEY-00000002 | Symmetric Key | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-ENUM-VALUE-OPAQUE-DATA-TYPE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.35 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-ACTIVATE-00000012 | Activate | enumeration\_value | both | client\_1\_0 | 00000012 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-ADD-ATTRIBUTE-0000000D | Add Attribute | enumeration\_value | both | client\_1\_0 | 0000000D | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-ADJUST-ATTRIBUTE-00000030 | Adjust Attribute | enumeration\_value | both | client\_1\_0 | 00000030 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-ARCHIVE-00000015 | Archive | enumeration\_value | both | client\_1\_0 | 00000015 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-CANCEL-00000019 | Cancel | enumeration\_value | both | client\_1\_0 | 00000019 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-CERTIFY-00000006 | Certify | enumeration\_value | both | client\_1\_0 | 00000006 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-CHECK-00000009 | Check | enumeration\_value | both | client\_1\_0 | 00000009 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-CREATE-00000001 | Create | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-CREATE-KEY-PAIR-00000002 | Create Key Pair | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-CREATE-SPLIT-KEY-00000028 | Create Split Key | enumeration\_value | both | client\_1\_0 | 00000028 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-DECRYPT-00000020 | Decrypt | enumeration\_value | both | client\_1\_0 | 00000020 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-DELEGATED-LOGIN-0000002F | Delegated Login | enumeration\_value | both | client\_1\_0 | 0000002F | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-DELETE-ATTRIBUTE-0000000F | Delete Attribute | enumeration\_value | both | client\_1\_0 | 0000000F | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-DERIVE-KEY-00000005 | Derive Key | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-DESTROY-00000014 | Destroy | enumeration\_value | both | client\_1\_0 | 00000014 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-DISCOVER-VERSIONS-0000001E | Discover Versions | enumeration\_value | both | client\_1\_0 | 0000001E | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-ENCRYPT-0000001F | Encrypt | enumeration\_value | both | client\_1\_0 | 0000001F | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-EXPORT-0000002B | Export | enumeration\_value | both | client\_1\_0 | 0000002B | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-GET-0000000A | Get | enumeration\_value | both | client\_1\_0 | 0000000A | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-GET-ATTRIBUTE-LIST-0000000C | Get Attribute List | enumeration\_value | both | client\_1\_0 | 0000000C | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-GET-ATTRIBUTES-0000000B | Get Attributes | enumeration\_value | both | client\_1\_0 | 0000000B | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-GET-CONSTRAINTS-00000038 | Get Constraints | enumeration\_value | both | client\_1\_0 | 00000038 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-GET-USAGE-ALLOCATION-00000011 | Get Usage Allocation | enumeration\_value | both | client\_1\_0 | 00000011 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-HASH-00000027 | Hash | enumeration\_value | both | client\_1\_0 | 00000027 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-IMPORT-0000002A | Import | enumeration\_value | both | client\_1\_0 | 0000002A | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-INTEROP-00000034 | Interop | enumeration\_value | both | client\_1\_0 | 00000034 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-JOIN-SPLIT-KEY-00000029 | Join Split Key | enumeration\_value | both | client\_1\_0 | 00000029 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-LOCATE-00000008 | Locate | enumeration\_value | both | client\_1\_0 | 00000008 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-LOG-0000002C | Log | enumeration\_value | both | client\_1\_0 | 0000002C | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-LOGIN-0000002D | Login | enumeration\_value | both | client\_1\_0 | 0000002D | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-LOGOUT-0000002E | Logout | enumeration\_value | both | client\_1\_0 | 0000002E | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-MAC-00000023 | MAC | enumeration\_value | both | client\_1\_0 | 00000023 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-MAC-VERIFY-00000024 | MAC Verify | enumeration\_value | both | client\_1\_0 | 00000024 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-MODIFY-ATTRIBUTE-0000000E | Modify Attribute | enumeration\_value | both | client\_1\_0 | 0000000E | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-NOTIFY-0000001B | Notify | enumeration\_value | both | client\_1\_0 | 0000001B | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-OBTAIN-LEASE-00000010 | Obtain Lease | enumeration\_value | both | client\_1\_0 | 00000010 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-PING-0000003B | Ping | enumeration\_value | both | client\_1\_0 | 0000003B | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-PKCS-11-00000033 | PKCS#11 | enumeration\_value | both | client\_1\_0 | 00000033 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-POLL-0000001A | Poll | enumeration\_value | both | client\_1\_0 | 0000001A | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-PROCESS-0000003A | Process | enumeration\_value | both | client\_1\_0 | 0000003A | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-PUT-0000001C | Put | enumeration\_value | both | client\_1\_0 | 0000001C | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-QUERY-00000018 | Query | enumeration\_value | both | client\_1\_0 | 00000018 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-QUERY-ASYNCHRONOUS-REQUESTS-00000039 | Query Asynchronous Requests | enumeration\_value | both | client\_1\_0 | 00000039 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RE-CERTIFY-00000007 | Re-certify | enumeration\_value | both | client\_1\_0 | 00000007 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RE-KEY-00000004 | Re-key | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RE-KEY-KEY-PAIR-0000001D | Re-key Key Pair | enumeration\_value | both | client\_1\_0 | 0000001D | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RE-PROVISION-00000035 | Re-Provision | enumeration\_value | both | client\_1\_0 | 00000035 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RECOVER-00000016 | Recover | enumeration\_value | both | client\_1\_0 | 00000016 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-REGISTER-00000003 | Register | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-REVOKE-00000013 | Revoke | enumeration\_value | both | client\_1\_0 | 00000013 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RNG-RETRIEVE-00000025 | RNG Retrieve | enumeration\_value | both | client\_1\_0 | 00000025 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RNG-SEED-00000026 | RNG Seed | enumeration\_value | both | client\_1\_0 | 00000026 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SET-ATTRIBUTE-00000031 | Set Attribute | enumeration\_value | both | client\_1\_0 | 00000031 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SET-CONSTRAINTS-00000037 | Set Constraints | enumeration\_value | both | client\_1\_0 | 00000037 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SET-DEFAULTS-00000036 | Set Defaults | enumeration\_value | both | client\_1\_0 | 00000036 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SET-ENDPOINT-ROLE-00000032 | Set Endpoint Role | enumeration\_value | both | client\_1\_0 | 00000032 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SIGN-00000021 | Sign | enumeration\_value | both | client\_1\_0 | 00000021 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SIGNATURE-VERIFY-00000022 | Signature Verify | enumeration\_value | both | client\_1\_0 | 00000022 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-VALIDATE-00000017 | Validate | enumeration\_value | both | client\_1\_0 | 00000017 | assigned | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-ANSI-X9-23-00000006 | ANSI X9.23 | enumeration\_value | both | client\_1\_0 | 00000006 | assigned | KMIPKIT-SRC-spec §11.37 |
| KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.37 |
| KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-ISO-10126-00000007 | ISO 10126 | enumeration\_value | both | client\_1\_0 | 00000007 | assigned | KMIPKIT-SRC-spec §11.37 |
| KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-NONE-00000001 | None | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.37 |
| KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-OAEP-00000002 | OAEP | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.37 |
| KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-PKCS1-V1-5-00000008 | PKCS1 v1.5 | enumeration\_value | both | client\_1\_0 | 00000008 | assigned | KMIPKIT-SRC-spec §11.37 |
| KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-PKCS5-00000003 | PKCS5 | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.37 |
| KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-PSS-0000000A | PSS | enumeration\_value | both | client\_1\_0 | 0000000A | assigned | KMIPKIT-SRC-spec §11.37 |
| KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-SSL3-00000004 | SSL3 | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.37 |
| KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-X9-31-00000009 | X9.31 | enumeration\_value | both | client\_1\_0 | 00000009 | assigned | KMIPKIT-SRC-spec §11.37 |
| KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-ZEROS-00000005 | Zeros | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.37 |
| KMIPKIT-ELEM-ENUM-VALUE-PROCESSING-STAGE-COMPLETED-00000003 | Completed | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.40 |
| KMIPKIT-ELEM-ENUM-VALUE-PROCESSING-STAGE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.40 |
| KMIPKIT-ELEM-ENUM-VALUE-PROCESSING-STAGE-IN-PROCESS-00000002 | In Process | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.40 |
| KMIPKIT-ELEM-ENUM-VALUE-PROCESSING-STAGE-SUBMITTED-00000001 | Submitted | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.40 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-ADVANCED-CRYPTOGRAPHIC-CLIENT-0000010E | Advanced Cryptographic Client | enumeration\_value | both | client\_1\_0 | 0000010E | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-ADVANCED-CRYPTOGRAPHIC-SERVER-0000010F | Advanced Cryptographic Server | enumeration\_value | both | client\_1\_0 | 0000010F | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-ADVANCED-SYMMETRIC-KEY-FOUNDRY-CLIENT-00000114 | Advanced Symmetric Key Foundry Client | enumeration\_value | both | client\_1\_0 | 00000114 | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-AES-XTS-CLIENT-00000124 | AES XTS Client | enumeration\_value | both | client\_1\_0 | 00000124 | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-AES-XTS-SERVER-00000125 | AES XTS Server | enumeration\_value | both | client\_1\_0 | 00000125 | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-ASYMMETRIC-KEY-LIFECYCLE-CLIENT-0000010A | Asymmetric Key Lifecycle Client | enumeration\_value | both | client\_1\_0 | 0000010A | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-ASYMMETRIC-KEY-LIFECYCLE-SERVER-0000010B | Asymmetric Key Lifecycle Server | enumeration\_value | both | client\_1\_0 | 0000010B | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-BASELINE-CLIENT-0000012A | Baseline Client | enumeration\_value | both | client\_1\_0 | 0000012A | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-BASELINE-SERVER-0000012B | Baseline Server | enumeration\_value | both | client\_1\_0 | 0000012B | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-BASIC-CRYPTOGRAPHIC-CLIENT-0000010C | Basic Cryptographic Client | enumeration\_value | both | client\_1\_0 | 0000010C | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-BASIC-CRYPTOGRAPHIC-SERVER-0000010D | Basic Cryptographic Server | enumeration\_value | both | client\_1\_0 | 0000010D | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-BASIC-SYMMETRIC-KEY-FOUNDRY-CLIENT-00000112 | Basic Symmetric Key Foundry Client | enumeration\_value | both | client\_1\_0 | 00000112 | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-COMPLETE-SERVER-0000012C | Complete Server | enumeration\_value | both | client\_1\_0 | 0000012C | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-COMPLETE-SERVER-BASIC-00000104 | Complete Server Basic | enumeration\_value | both | client\_1\_0 | 00000104 | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-COMPLETE-SERVER-TLS-V1-2-00000105 | Complete Server TLS v1.2 | enumeration\_value | both | client\_1\_0 | 00000105 | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-HTTPS-CLIENT-0000011E | HTTPS Client | enumeration\_value | both | client\_1\_0 | 0000011E | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-HTTPS-SERVER-0000011F | HTTPS Server | enumeration\_value | both | client\_1\_0 | 0000011F | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-INTERMEDIATE-SYMMETRIC-KEY-FOUNDRY-CLIENT-00000113 | Intermediate Symmetric Key Foundry Client | enumeration\_value | both | client\_1\_0 | 00000113 | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-JSON-CLIENT-00000120 | JSON Client | enumeration\_value | both | client\_1\_0 | 00000120 | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-JSON-SERVER-00000121 | JSON Server | enumeration\_value | both | client\_1\_0 | 00000121 | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-OPAQUE-MANAGED-OBJECT-STORE-CLIENT-00000116 | Opaque Managed Object Store Client | enumeration\_value | both | client\_1\_0 | 00000116 | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-OPAQUE-MANAGED-OBJECT-STORE-SERVER-00000117 | Opaque Managed Object Store Server | enumeration\_value | both | client\_1\_0 | 00000117 | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-PKCS-11-CLIENT-00000128 | PKCS#11 Client | enumeration\_value | both | client\_1\_0 | 00000128 | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-PKCS-11-SERVER-00000129 | PKCS#11 Server | enumeration\_value | both | client\_1\_0 | 00000129 | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-QUANTUM-SAFE-CLIENT-00000126 | Quantum Safe Client | enumeration\_value | both | client\_1\_0 | 00000126 | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-QUANTUM-SAFE-SERVER-00000127 | Quantum Safe Server | enumeration\_value | both | client\_1\_0 | 00000127 | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-RESERVED-00000001-00000103 | \(Reserved\) | enumeration\_value | both | client\_1\_0 | 00000001-00000103 | reserved | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-RESERVED-00000118 | \(Reserved\) | enumeration\_value | both | client\_1\_0 | 00000118 | reserved | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-RESERVED-00000119 | \(Reserved\) | enumeration\_value | both | client\_1\_0 | 00000119 | reserved | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-RESERVED-0000011A | \(Reserved\) | enumeration\_value | both | client\_1\_0 | 0000011A | reserved | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-RESERVED-0000011B | \(Reserved\) | enumeration\_value | both | client\_1\_0 | 0000011B | reserved | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-RNG-CRYPTOGRAPHIC-CLIENT-00000110 | RNG Cryptographic Client | enumeration\_value | both | client\_1\_0 | 00000110 | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-RNG-CRYPTOGRAPHIC-SERVER-00000111 | RNG Cryptographic Server | enumeration\_value | both | client\_1\_0 | 00000111 | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-STORAGE-ARRAY-WITH-SELF-ENCRYPTING-DRIVE-CLIENT-0000011C | Storage Array with Self Encrypting Drive Client | enumeration\_value | both | client\_1\_0 | 0000011C | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-STORAGE-ARRAY-WITH-SELF-ENCRYPTING-DRIVE-SERVER-0000011D | Storage Array with Self Encrypting Drive Server | enumeration\_value | both | client\_1\_0 | 0000011D | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-SYMMETRIC-KEY-FOUNDRY-SERVER-00000115 | Symmetric Key Foundry Server | enumeration\_value | both | client\_1\_0 | 00000115 | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-SYMMETRIC-KEY-LIFECYCLE-CLIENT-00000108 | Symmetric Key Lifecycle Client | enumeration\_value | both | client\_1\_0 | 00000108 | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-SYMMETRIC-KEY-LIFECYCLE-SERVER-00000109 | Symmetric Key Lifecycle Server | enumeration\_value | both | client\_1\_0 | 00000109 | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-TAPE-LIBRARY-CLIENT-00000106 | Tape Library Client | enumeration\_value | both | client\_1\_0 | 00000106 | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-TAPE-LIBRARY-SERVER-00000107 | Tape Library Server | enumeration\_value | both | client\_1\_0 | 00000107 | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-XML-CLIENT-00000122 | XML Client | enumeration\_value | both | client\_1\_0 | 00000122 | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-XML-SERVER-00000123 | XML Server | enumeration\_value | both | client\_1\_0 | 00000123 | assigned | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROTECTION-LEVEL-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.42 |
| KMIPKIT-ELEM-ENUM-VALUE-PROTECTION-LEVEL-HIGH-00000001 | High | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.42 |
| KMIPKIT-ELEM-ENUM-VALUE-PROTECTION-LEVEL-LOW-00000002 | Low | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.42 |
| KMIPKIT-ELEM-ENUM-VALUE-PUT-FUNCTION-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.43 |
| KMIPKIT-ELEM-ENUM-VALUE-PUT-FUNCTION-NEW-00000001 | New | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.43 |
| KMIPKIT-ELEM-ENUM-VALUE-PUT-FUNCTION-REPLACE-00000002 | Replace | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.43 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-APPLICATION-NAMESPACES-00000004 | Query Application Namespaces | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-ATTESTATION-TYPES-00000007 | Query Attestation Types | enumeration\_value | both | client\_1\_0 | 00000007 | assigned | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-CAPABILITIES-0000000B | Query Capabilities | enumeration\_value | both | client\_1\_0 | 0000000B | assigned | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-CLIENT-REGISTRATION-METHODS-0000000C | Query Client Registration Methods | enumeration\_value | both | client\_1\_0 | 0000000C | assigned | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-DEFAULTS-INFORMATION-0000000D | Query Defaults Information | enumeration\_value | both | client\_1\_0 | 0000000D | assigned | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-EXTENSION-LIST-00000005 | Query Extension List | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-EXTENSION-MAP-00000006 | Query Extension Map | enumeration\_value | both | client\_1\_0 | 00000006 | assigned | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-OBJECTS-00000002 | Query Objects | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-OPERATIONS-00000001 | Query Operations | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-PROFILES-0000000A | Query Profiles | enumeration\_value | both | client\_1\_0 | 0000000A | assigned | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-RNGS-00000008 | Query RNGs | enumeration\_value | both | client\_1\_0 | 00000008 | assigned | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-SERVER-INFORMATION-00000003 | Query Server Information | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-STORAGE-PROTECTION-MASKS-0000000E | Query Storage Protection Masks | enumeration\_value | both | client\_1\_0 | 0000000E | assigned | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-VALIDATIONS-00000009 | Query Validations | enumeration\_value | both | client\_1\_0 | 00000009 | assigned | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB163V1-00000027 | ANSIX9C2PNB163V1 | enumeration\_value | both | client\_1\_0 | 00000027 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB163V2-00000028 | ANSIX9C2PNB163V2 | enumeration\_value | both | client\_1\_0 | 00000028 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB163V3-00000029 | ANSIX9C2PNB163V3 | enumeration\_value | both | client\_1\_0 | 00000029 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB176V1-0000002A | ANSIX9C2PNB176V1 | enumeration\_value | both | client\_1\_0 | 0000002A | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB208W1-0000002E | ANSIX9C2PNB208W1 | enumeration\_value | both | client\_1\_0 | 0000002E | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB272W1-00000032 | ANSIX9C2PNB272W1 | enumeration\_value | both | client\_1\_0 | 00000032 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB304W1-00000033 | ANSIX9C2PNB304W1 | enumeration\_value | both | client\_1\_0 | 00000033 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB368W1-00000035 | ANSIX9C2PNB368W1 | enumeration\_value | both | client\_1\_0 | 00000035 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB191V1-0000002B | ANSIX9C2TNB191V1 | enumeration\_value | both | client\_1\_0 | 0000002B | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB191V2-0000002C | ANSIX9C2TNB191V2 | enumeration\_value | both | client\_1\_0 | 0000002C | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB191V3-0000002D | ANSIX9C2TNB191V3 | enumeration\_value | both | client\_1\_0 | 0000002D | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB239V1-0000002F | ANSIX9C2TNB239V1 | enumeration\_value | both | client\_1\_0 | 0000002F | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB239V2-00000030 | ANSIX9C2TNB239V2 | enumeration\_value | both | client\_1\_0 | 00000030 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB239V3-00000031 | ANSIX9C2TNB239V3 | enumeration\_value | both | client\_1\_0 | 00000031 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB359V1-00000034 | ANSIX9C2TNB359V1 | enumeration\_value | both | client\_1\_0 | 00000034 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB431R1-00000036 | ANSIX9C2TNB431R1 | enumeration\_value | both | client\_1\_0 | 00000036 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9P192V2-00000022 | ANSIX9P192V2 | enumeration\_value | both | client\_1\_0 | 00000022 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9P192V3-00000023 | ANSIX9P192V3 | enumeration\_value | both | client\_1\_0 | 00000023 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9P239V1-00000024 | ANSIX9P239V1 | enumeration\_value | both | client\_1\_0 | 00000024 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9P239V2-00000025 | ANSIX9P239V2 | enumeration\_value | both | client\_1\_0 | 00000025 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9P239V3-00000026 | ANSIX9P239V3 | enumeration\_value | both | client\_1\_0 | 00000026 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-B-163-00000003 | B-163 | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-B-233-00000006 | B-233 | enumeration\_value | both | client\_1\_0 | 00000006 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-B-283-00000009 | B-283 | enumeration\_value | both | client\_1\_0 | 00000009 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-B-409-0000000C | B-409 | enumeration\_value | both | client\_1\_0 | 0000000C | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-B-571-0000000F | B-571 | enumeration\_value | both | client\_1\_0 | 0000000F | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP160R1-00000037 | BRAINPOOLP160R1 | enumeration\_value | both | client\_1\_0 | 00000037 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP160T1-00000038 | BRAINPOOLP160T1 | enumeration\_value | both | client\_1\_0 | 00000038 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP192R1-00000039 | BRAINPOOLP192R1 | enumeration\_value | both | client\_1\_0 | 00000039 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP192T1-0000003A | BRAINPOOLP192T1 | enumeration\_value | both | client\_1\_0 | 0000003A | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP224R1-0000003B | BRAINPOOLP224R1 | enumeration\_value | both | client\_1\_0 | 0000003B | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP224T1-0000003C | BRAINPOOLP224T1 | enumeration\_value | both | client\_1\_0 | 0000003C | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP256R1-0000003D | BRAINPOOLP256R1 | enumeration\_value | both | client\_1\_0 | 0000003D | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP256T1-0000003E | BRAINPOOLP256T1 | enumeration\_value | both | client\_1\_0 | 0000003E | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP320R1-0000003F | BRAINPOOLP320R1 | enumeration\_value | both | client\_1\_0 | 0000003F | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP320T1-00000040 | BRAINPOOLP320T1 | enumeration\_value | both | client\_1\_0 | 00000040 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP384R1-00000041 | BRAINPOOLP384R1 | enumeration\_value | both | client\_1\_0 | 00000041 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP384T1-00000042 | BRAINPOOLP384T1 | enumeration\_value | both | client\_1\_0 | 00000042 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP512R1-00000043 | BRAINPOOLP512R1 | enumeration\_value | both | client\_1\_0 | 00000043 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP512T1-00000044 | BRAINPOOLP512T1 | enumeration\_value | both | client\_1\_0 | 00000044 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-CURVE25519-00000045 | CURVE25519 | enumeration\_value | both | client\_1\_0 | 00000045 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-CURVE448-00000046 | CURVE448 | enumeration\_value | both | client\_1\_0 | 00000046 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-K-163-00000002 | K-163 | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-K-233-00000005 | K-233 | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-K-283-00000008 | K-283 | enumeration\_value | both | client\_1\_0 | 00000008 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-K-409-0000000B | K-409 | enumeration\_value | both | client\_1\_0 | 0000000B | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-K-571-0000000E | K-571 | enumeration\_value | both | client\_1\_0 | 0000000E | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-P-192-00000001 | P-192 | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-P-224-00000004 | P-224 | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-P-256-00000007 | P-256 | enumeration\_value | both | client\_1\_0 | 00000007 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-P-384-0000000A | P-384 | enumeration\_value | both | client\_1\_0 | 0000000A | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-P-521-0000000D | P-521 | enumeration\_value | both | client\_1\_0 | 0000000D | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP112R1-00000010 | SECP112R1 | enumeration\_value | both | client\_1\_0 | 00000010 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP112R2-00000011 | SECP112R2 | enumeration\_value | both | client\_1\_0 | 00000011 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP128R1-00000012 | SECP128R1 | enumeration\_value | both | client\_1\_0 | 00000012 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP128R2-00000013 | SECP128R2 | enumeration\_value | both | client\_1\_0 | 00000013 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP160K1-00000014 | SECP160K1 | enumeration\_value | both | client\_1\_0 | 00000014 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP160R1-00000015 | SECP160R1 | enumeration\_value | both | client\_1\_0 | 00000015 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP160R2-00000016 | SECP160R2 | enumeration\_value | both | client\_1\_0 | 00000016 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP192K1-00000017 | SECP192K1 | enumeration\_value | both | client\_1\_0 | 00000017 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP224K1-00000018 | SECP224K1 | enumeration\_value | both | client\_1\_0 | 00000018 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP256K1-00000019 | SECP256K1 | enumeration\_value | both | client\_1\_0 | 00000019 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT113R1-0000001A | SECT113R1 | enumeration\_value | both | client\_1\_0 | 0000001A | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT113R2-0000001B | SECT113R2 | enumeration\_value | both | client\_1\_0 | 0000001B | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT131R1-0000001C | SECT131R1 | enumeration\_value | both | client\_1\_0 | 0000001C | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT131R2-0000001D | SECT131R2 | enumeration\_value | both | client\_1\_0 | 0000001D | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT163R1-0000001E | SECT163R1 | enumeration\_value | both | client\_1\_0 | 0000001E | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT193R1-0000001F | SECT193R1 | enumeration\_value | both | client\_1\_0 | 0000001F | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT193R2-00000020 | SECT193R2 | enumeration\_value | both | client\_1\_0 | 00000020 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT239K1-00000021 | SECT239K1 | enumeration\_value | both | client\_1\_0 | 00000021 | assigned | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-RESERVED-0000000B | \(Reserved\) | enumeration\_value | both | client\_1\_0 | 0000000B | reserved | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-RESERVED-0000000E | \(Reserved\) | enumeration\_value | both | client\_1\_0 | 0000000E | reserved | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-RESERVED-00000027 | \(Reserved\) | enumeration\_value | both | client\_1\_0 | 00000027 | reserved | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-RESERVED-00000031 | \(Reserved\) | enumeration\_value | both | client\_1\_0 | 00000031 | reserved | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-RESERVED-00000033 | \(Reserved\) | enumeration\_value | both | client\_1\_0 | 00000033 | reserved | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-RESERVED-00000038 | \(Reserved\) | enumeration\_value | both | client\_1\_0 | 00000038 | reserved | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-AFFILIATION-CHANGED-00000004 | Affiliation Changed | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.48 |
| KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-CA-COMPROMISE-00000003 | CA Compromise | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.48 |
| KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-CESSATION-OF-OPERATION-00000006 | Cessation of Operation | enumeration\_value | both | client\_1\_0 | 00000006 | assigned | KMIPKIT-SRC-spec §11.48 |
| KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.48 |
| KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-KEY-COMPROMISE-00000002 | Key Compromise | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.48 |
| KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-PRIVILEGE-WITHDRAWN-00000007 | Privilege Withdrawn | enumeration\_value | both | client\_1\_0 | 00000007 | assigned | KMIPKIT-SRC-spec §11.48 |
| KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-SUPERSEDED-00000005 | Superseded | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.48 |
| KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-UNSPECIFIED-00000001 | Unspecified | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.48 |
| KMIPKIT-ELEM-ENUM-VALUE-RNG-ALGORITHM-ANSI-X9-31-00000005 | ANSI X9.31 | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.49 |
| KMIPKIT-ELEM-ENUM-VALUE-RNG-ALGORITHM-ANSI-X9-62-00000006 | ANSI X9.62 | enumeration\_value | both | client\_1\_0 | 00000006 | assigned | KMIPKIT-SRC-spec §11.49 |
| KMIPKIT-ELEM-ENUM-VALUE-RNG-ALGORITHM-DRBG-00000003 | DRBG | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.49 |
| KMIPKIT-ELEM-ENUM-VALUE-RNG-ALGORITHM-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.49 |
| KMIPKIT-ELEM-ENUM-VALUE-RNG-ALGORITHM-FIPS-186-2-00000002 | FIPS 186-2 | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.49 |
| KMIPKIT-ELEM-ENUM-VALUE-RNG-ALGORITHM-NRBG-00000004 | NRBG | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.49 |
| KMIPKIT-ELEM-ENUM-VALUE-RNG-ALGORITHM-UNSPECIFIED-00000001 | Unspecified | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.49 |
| KMIPKIT-ELEM-ENUM-VALUE-RNG-MODE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.50 |
| KMIPKIT-ELEM-ENUM-VALUE-RNG-MODE-NON-SHARED-INSTANTIATION-00000003 | Non-Shared Instantiation | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.50 |
| KMIPKIT-ELEM-ENUM-VALUE-RNG-MODE-SHARED-INSTANTIATION-00000002 | Shared Instantiation | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.50 |
| KMIPKIT-ELEM-ENUM-VALUE-RNG-MODE-UNSPECIFIED-00000001 | Unspecified | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.50 |
| KMIPKIT-ELEM-ENUM-VALUE-ROTATE-NAME-TYPE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.51 |
| KMIPKIT-ELEM-ENUM-VALUE-ROTATE-NAME-TYPE-UNINTERPRETED-TEXT-STRING-00000001 | Uninterpreted Text String | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.51 |
| KMIPKIT-ELEM-ENUM-VALUE-ROTATE-NAME-TYPE-URI-00000002 | URI | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.51 |
| KMIPKIT-ELEM-ENUM-VALUE-SECRET-DATA-TYPE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.52 |
| KMIPKIT-ELEM-ENUM-VALUE-SECRET-DATA-TYPE-PASSWORD-00000001 | Password | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.52 |
| KMIPKIT-ELEM-ENUM-VALUE-SECRET-DATA-TYPE-SEED-00000002 | Seed | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.52 |
| KMIPKIT-ELEM-ENUM-VALUE-SHREDDING-ALGORITHM-CRYPTOGRAPHIC-00000002 | Cryptographic | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.53 |
| KMIPKIT-ELEM-ENUM-VALUE-SHREDDING-ALGORITHM-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.53 |
| KMIPKIT-ELEM-ENUM-VALUE-SHREDDING-ALGORITHM-UNSPECIFIED-00000001 | Unspecified | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.53 |
| KMIPKIT-ELEM-ENUM-VALUE-SHREDDING-ALGORITHM-UNSUPPORTED-00000003 | Unsupported | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.53 |
| KMIPKIT-ELEM-ENUM-VALUE-SPLIT-KEY-METHOD-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.54 |
| KMIPKIT-ELEM-ENUM-VALUE-SPLIT-KEY-METHOD-POLYNOMIAL-SHARING-GF-2-00000002 | Polynomial Sharing GF \(2¹⁶\) | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.54 |
| KMIPKIT-ELEM-ENUM-VALUE-SPLIT-KEY-METHOD-POLYNOMIAL-SHARING-GF-2-00000004 | Polynomial Sharing GF \(2⁸\) | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.54 |
| KMIPKIT-ELEM-ENUM-VALUE-SPLIT-KEY-METHOD-POLYNOMIAL-SHARING-PRIME-FIELD-00000003 | Polynomial Sharing Prime Field | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.54 |
| KMIPKIT-ELEM-ENUM-VALUE-SPLIT-KEY-METHOD-XOR-00000001 | XOR | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.54 |
| KMIPKIT-ELEM-ENUM-VALUE-STATE-ACTIVE-00000002 | Active | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.55 |
| KMIPKIT-ELEM-ENUM-VALUE-STATE-COMPROMISED-00000004 | Compromised | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.55 |
| KMIPKIT-ELEM-ENUM-VALUE-STATE-DEACTIVATED-00000003 | Deactivated | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.55 |
| KMIPKIT-ELEM-ENUM-VALUE-STATE-DESTROYED-00000005 | Destroyed | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.55 |
| KMIPKIT-ELEM-ENUM-VALUE-STATE-DESTROYED-COMPROMISED-00000006 | Destroyed Compromised | enumeration\_value | both | client\_1\_0 | 00000006 | assigned | KMIPKIT-SRC-spec §11.55 |
| KMIPKIT-ELEM-ENUM-VALUE-STATE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.55 |
| KMIPKIT-ELEM-ENUM-VALUE-STATE-PRE-ACTIVE-00000001 | Pre-Active | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.55 |
| KMIPKIT-ELEM-ENUM-VALUE-TICKET-TYPE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.57 |
| KMIPKIT-ELEM-ENUM-VALUE-TICKET-TYPE-LOGIN-00000001 | Login | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.57 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-CERTIFY-00000002 | Certify | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-CREATE-00000003 | Create | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-CREATE-KEY-PAIR-00000004 | Create Key Pair | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-CREATE-KEY-PAIR-PRIVATE-KEY-00000005 | Create Key Pair Private Key | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-CREATE-KEY-PAIR-PUBLIC-KEY-00000006 | Create Key Pair Public Key | enumeration\_value | both | client\_1\_0 | 00000006 | assigned | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-CREATE-SPLIT-KEY-00000007 | Create Split Key | enumeration\_value | both | client\_1\_0 | 00000007 | assigned | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-DERIVE-KEY-00000008 | Derive Key | enumeration\_value | both | client\_1\_0 | 00000008 | assigned | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-ID-PLACEHOLDER-00000001 | ID Placeholder | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-IMPORT-00000009 | Import | enumeration\_value | both | client\_1\_0 | 00000009 | assigned | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-JOIN-SPLIT-KEY-0000000A | Join Split Key | enumeration\_value | both | client\_1\_0 | 0000000A | assigned | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-LOCATE-0000000B | Locate | enumeration\_value | both | client\_1\_0 | 0000000B | assigned | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-RE-CERTIFY-0000000E | Re-certify | enumeration\_value | both | client\_1\_0 | 0000000E | assigned | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-RE-KEY-0000000D | Re-key | enumeration\_value | both | client\_1\_0 | 0000000D | assigned | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-RE-KEY-KEY-PAIR-0000000F | Re-key Key Pair | enumeration\_value | both | client\_1\_0 | 0000000F | assigned | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-RE-KEY-KEY-PAIR-PRIVATE-KEY-00000010 | Re-key Key Pair Private Key | enumeration\_value | both | client\_1\_0 | 00000010 | assigned | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-RE-KEY-KEY-PAIR-PUBLIC-KEY-00000011 | Re-key Key Pair Public Key | enumeration\_value | both | client\_1\_0 | 00000011 | assigned | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-REGISTER-0000000C | Register | enumeration\_value | both | client\_1\_0 | 0000000C | assigned | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNWRAP-MODE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.59 |
| KMIPKIT-ELEM-ENUM-VALUE-UNWRAP-MODE-NOT-PROCESSED-00000003 | Not Processed | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.59 |
| KMIPKIT-ELEM-ENUM-VALUE-UNWRAP-MODE-PROCESSED-00000002 | Processed | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.59 |
| KMIPKIT-ELEM-ENUM-VALUE-UNWRAP-MODE-UNSPECIFIED-00000001 | Unspecified | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.59 |
| KMIPKIT-ELEM-ENUM-VALUE-USAGE-LIMITS-UNIT-BYTE-00000001 | Byte | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.60 |
| KMIPKIT-ELEM-ENUM-VALUE-USAGE-LIMITS-UNIT-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.60 |
| KMIPKIT-ELEM-ENUM-VALUE-USAGE-LIMITS-UNIT-OBJECT-00000002 | Object | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.60 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-AUTHORITY-TYPE-COMMON-CRITERIA-00000003 | Common Criteria | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.63 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-AUTHORITY-TYPE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.63 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-AUTHORITY-TYPE-NIST-CMVP-00000002 | NIST CMVP | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.63 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-AUTHORITY-TYPE-UNSPECIFIED-00000001 | Unspecified | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.63 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-TYPE-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.64 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-TYPE-FIRMWARE-00000004 | Firmware | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.64 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-TYPE-HARDWARE-00000002 | Hardware | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.64 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-TYPE-HYBRID-00000005 | Hybrid | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.64 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-TYPE-SOFTWARE-00000003 | Software | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.64 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-TYPE-UNSPECIFIED-00000001 | Unspecified | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.64 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDITY-INDICATOR-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.61 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDITY-INDICATOR-INVALID-00000002 | Invalid | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.61 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDITY-INDICATOR-UNKNOWN-00000003 | Unknown | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.61 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDITY-INDICATOR-VALID-00000001 | Valid | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.61 |
| KMIPKIT-ELEM-ENUM-VALUE-WRAPPING-METHOD-ENCRYPT-00000001 | Encrypt | enumeration\_value | both | client\_1\_0 | 00000001 | assigned | KMIPKIT-SRC-spec §11.62 |
| KMIPKIT-ELEM-ENUM-VALUE-WRAPPING-METHOD-ENCRYPT-THEN-MAC-SIGN-00000003 | Encrypt then MAC/sign | enumeration\_value | both | client\_1\_0 | 00000003 | assigned | KMIPKIT-SRC-spec §11.62 |
| KMIPKIT-ELEM-ENUM-VALUE-WRAPPING-METHOD-EXTENSIONS-8XXXXXXX | Extensions | enumeration\_value | both | client\_1\_0 | 8XXXXXXX | extension | KMIPKIT-SRC-spec §11.62 |
| KMIPKIT-ELEM-ENUM-VALUE-WRAPPING-METHOD-MAC-SIGN-00000002 | MAC/sign | enumeration\_value | both | client\_1\_0 | 00000002 | assigned | KMIPKIT-SRC-spec §11.62 |
| KMIPKIT-ELEM-ENUM-VALUE-WRAPPING-METHOD-MAC-SIGN-THEN-ENCRYPT-00000004 | MAC/sign then encrypt | enumeration\_value | both | client\_1\_0 | 00000004 | assigned | KMIPKIT-SRC-spec §11.62 |
| KMIPKIT-ELEM-ENUM-VALUE-WRAPPING-METHOD-TR-31-00000005 | TR-31 | enumeration\_value | both | client\_1\_0 | 00000005 | assigned | KMIPKIT-SRC-spec §11.62 |
| KMIPKIT-ELEM-ENUMERATION-ADJUSTMENT-TYPE | Adjustment Type | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.1 |
| KMIPKIT-ELEM-ENUMERATION-ALTERNATIVE-NAME-TYPE | Alternative Name Type | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.2 |
| KMIPKIT-ELEM-ENUMERATION-ASYNCHRONOUS-INDICATOR | Asynchronous Indicator | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.3 |
| KMIPKIT-ELEM-ENUMERATION-ATTESTATION-TYPE | Attestation Type | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.4 |
| KMIPKIT-ELEM-ENUMERATION-BATCH-ERROR-CONTINUATION-OPTION | Batch Error Continuation Option | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.5 |
| KMIPKIT-ELEM-ENUMERATION-BLOCK-CIPHER-MODE | Block Cipher Mode | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUMERATION-CANCELLATION-RESULT | Cancellation Result | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.7 |
| KMIPKIT-ELEM-ENUMERATION-CERTIFICATE-REQUEST-TYPE | Certificate Request Type | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.8 |
| KMIPKIT-ELEM-ENUMERATION-CERTIFICATE-TYPE | Certificate Type | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.9 |
| KMIPKIT-ELEM-ENUMERATION-CLIENT-REGISTRATION-METHOD | Client Registration Method | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.10 |
| KMIPKIT-ELEM-ENUMERATION-CREDENTIAL-TYPE | Credential Type | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.11 |
| KMIPKIT-ELEM-ENUMERATION-CRYPTOGRAPHIC-ALGORITHM | Cryptographic Algorithm | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUMERATION-DATA | Data | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.13 |
| KMIPKIT-ELEM-ENUMERATION-DERIVATION-METHOD | Derivation Method | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.14 |
| KMIPKIT-ELEM-ENUMERATION-DESTROY-ACTION | Destroy Action | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.15 |
| KMIPKIT-ELEM-ENUMERATION-DIGITAL-SIGNATURE-ALGORITHM | Digital Signature Algorithm | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUMERATION-DRBG-ALGORITHM | DRBG Algorithm | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.17 |
| KMIPKIT-ELEM-ENUMERATION-ENCODING-OPTION | Encoding Option | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.18 |
| KMIPKIT-ELEM-ENUMERATION-ENDPOINT-ROLE | Endpoint Role | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.19 |
| KMIPKIT-ELEM-ENUMERATION-FIPS186-VARIATION | FIPS186 Variation | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.20 |
| KMIPKIT-ELEM-ENUMERATION-HASHING-ALGORITHM | Hashing Algorithm | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUMERATION-INTEROP-FUNCTION | Interop Function | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.22 |
| KMIPKIT-ELEM-ENUMERATION-ITEM-TYPE | Item Type | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-ENUMERATION-KEY-COMPRESSION-TYPE | Key Compression Type | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.24 |
| KMIPKIT-ELEM-ENUMERATION-KEY-FORMAT-TYPE | Key Format Type | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUMERATION-KEY-ROLE-TYPE | Key Role Type | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUMERATION-KEY-VALUE-LOCATION-TYPE | Key Value Location Type | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.27 |
| KMIPKIT-ELEM-ENUMERATION-KEY-WRAP-TYPE | Key Wrap Type | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.29 |
| KMIPKIT-ELEM-ENUMERATION-LINK-TYPE | Link Type | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUMERATION-MASK-GENERATOR | Mask Generator | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.30 |
| KMIPKIT-ELEM-ENUMERATION-NAME-TYPE | Name Type | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.31 |
| KMIPKIT-ELEM-ENUMERATION-NIST-KEY-TYPE | NIST Key Type | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUMERATION-OBJECT-GROUP-MEMBER | Object Group Member | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.33 |
| KMIPKIT-ELEM-ENUMERATION-OBJECT-TYPE | Object Type | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-ENUMERATION-OPAQUE-DATA-TYPE | Opaque Data Type | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.35 |
| KMIPKIT-ELEM-ENUMERATION-OPERATION | Operation | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUMERATION-PADDING-METHOD | Padding Method | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.37 |
| KMIPKIT-ELEM-ENUMERATION-PKCS-11-FUNCTION | PKCS#11 Function | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.38 |
| KMIPKIT-ELEM-ENUMERATION-PKCS-11-RETURN-CODE | PKCS#11 Return Code | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.39 |
| KMIPKIT-ELEM-ENUMERATION-PROCESSING-STAGE | Processing Stage | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.40 |
| KMIPKIT-ELEM-ENUMERATION-PROFILE-NAME | Profile Name | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUMERATION-PROTECTION-LEVEL | Protection Level | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.42 |
| KMIPKIT-ELEM-ENUMERATION-PUT-FUNCTION | Put Function | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.43 |
| KMIPKIT-ELEM-ENUMERATION-QUERY-FUNCTION | Query Function | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUMERATION-RECOMMENDED-CURVE | Recommended Curve | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUMERATION-REVOCATION-REASON-CODE | Revocation Reason Code | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.48 |
| KMIPKIT-ELEM-ENUMERATION-RNG-ALGORITHM | RNG Algorithm | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.49 |
| KMIPKIT-ELEM-ENUMERATION-RNG-MODE | RNG Mode | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.50 |
| KMIPKIT-ELEM-ENUMERATION-ROTATE-NAME-TYPE | Rotate Name Type | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.51 |
| KMIPKIT-ELEM-ENUMERATION-SECRET-DATA-TYPE | Secret Data Type | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.52 |
| KMIPKIT-ELEM-ENUMERATION-SHREDDING-ALGORITHM | Shredding Algorithm | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.53 |
| KMIPKIT-ELEM-ENUMERATION-SPLIT-KEY-METHOD | Split Key Method | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.54 |
| KMIPKIT-ELEM-ENUMERATION-STATE | State | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.55 |
| KMIPKIT-ELEM-ENUMERATION-TAG | Tag | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ENUMERATION-TICKET-TYPE | Ticket Type | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.57 |
| KMIPKIT-ELEM-ENUMERATION-UNIQUE-IDENTIFIER | Unique Identifier | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUMERATION-UNWRAP-MODE | Unwrap Mode | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.59 |
| KMIPKIT-ELEM-ENUMERATION-USAGE-LIMITS-UNIT | Usage Limits Unit | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.60 |
| KMIPKIT-ELEM-ENUMERATION-VALIDATION-AUTHORITY-TYPE | Validation Authority Type | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.63 |
| KMIPKIT-ELEM-ENUMERATION-VALIDATION-TYPE | Validation Type | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.64 |
| KMIPKIT-ELEM-ENUMERATION-VALIDITY-INDICATOR | Validity Indicator | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.61 |
| KMIPKIT-ELEM-ENUMERATION-WRAPPING-METHOD | Wrapping Method | enumeration | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.62 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-4-CREDENTIAL-MAY-BE-REPEATED | Credential, MAY be repeated | message\_field | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.4 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-3-10-KEY-MATERIAL | Key Material | object\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.10 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-3-11-KEY-MATERIAL | Key Material | object\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.11 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-3-12-KEY-MATERIAL | Key Material | object\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.12 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-3-4-KEY-MATERIAL | Key Material | object\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.4 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-3-5-KEY-MATERIAL | Key Material | object\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.5 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-3-6-KEY-MATERIAL | Key Material | object\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.6 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-3-7-KEY-MATERIAL | Key Material | object\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.7 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-3-8-KEY-MATERIAL | Key Material | object\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.8 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-3-9-KEY-MATERIAL | Key Material | object\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.9 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-CERTIFICATE | Certificate | object\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.1 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-CERTIFICATE-REQUEST | Certificate Request | object\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.2 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-ENCRYPTION-KEY-INFORMATION | Encryption Key Information | object\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-KEY-BLOCK | Key Block | object\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-KEY-VALUE | Key Value | object\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.2 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-KEY-WRAPPING-DATA | Key Wrapping Data | object\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-MAC-SIGNATURE-KEY-INFORMATION | MAC/Signature Key Information | object\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-OPAQUE-OBJECT | Opaque Object | object\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.3 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-PGP-KEY | PGP Key | object\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.4 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-PRIVATE-KEY | Private Key | object\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.5 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-PUBLIC-KEY | Public Key | object\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.6 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-SECRET-DATA | Secret Data | object\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.7 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-SPLIT-KEY | Split Key | object\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.8 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-SYMMETRIC-KEY | Symmetric Key | object\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.9 |
| KMIPKIT-ELEM-OBJECT-TYPE-CERTIFICATE | Certificate | object\_type | both | client\_1\_0 | 00000001 |  | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-OBJECT-TYPE-CERTIFICATE-REQUEST | Certificate Request | object\_type | both | client\_1\_0 | 0000000A |  | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-OBJECT-TYPE-OPAQUE-OBJECT | Opaque Object | object\_type | both | client\_1\_0 | 00000008 |  | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-OBJECT-TYPE-PGP-KEY | PGP Key | object\_type | both | client\_1\_0 | 00000009 |  | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-OBJECT-TYPE-PRIVATE-KEY | Private Key | object\_type | both | client\_1\_0 | 00000004 |  | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-OBJECT-TYPE-PUBLIC-KEY | Public Key | object\_type | both | client\_1\_0 | 00000003 |  | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-OBJECT-TYPE-SECRET-DATA | Secret Data | object\_type | both | client\_1\_0 | 00000007 |  | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-OBJECT-TYPE-SPLIT-KEY | Split Key | object\_type | both | client\_1\_0 | 00000005 |  | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-OBJECT-TYPE-SYMMETRIC-KEY | Symmetric Key | object\_type | both | client\_1\_0 | 00000002 |  | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-OP-C2S-ACTIVATE | Activate | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.1 |
| KMIPKIT-ELEM-OP-C2S-ADD-ATTRIBUTE | Add Attribute | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.2 |
| KMIPKIT-ELEM-OP-C2S-ADJUST-ATTRIBUTE | Adjust Attribute | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.3 |
| KMIPKIT-ELEM-OP-C2S-ARCHIVE | Archive | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.4 |
| KMIPKIT-ELEM-OP-C2S-CANCEL | Cancel | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.5 |
| KMIPKIT-ELEM-OP-C2S-CERTIFY | Certify | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.6 |
| KMIPKIT-ELEM-OP-C2S-CHECK | Check | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.7 |
| KMIPKIT-ELEM-OP-C2S-CREATE | Create | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.8 |
| KMIPKIT-ELEM-OP-C2S-CREATE-KEY-PAIR | Create Key Pair | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.9 |
| KMIPKIT-ELEM-OP-C2S-CREATE-SPLIT-KEY | Create Split Key | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.10 |
| KMIPKIT-ELEM-OP-C2S-DECRYPT | Decrypt | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.11 |
| KMIPKIT-ELEM-OP-C2S-DELEGATED-LOGIN | Delegated Login | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.12 |
| KMIPKIT-ELEM-OP-C2S-DELETE-ATTRIBUTE | Delete Attribute | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.13 |
| KMIPKIT-ELEM-OP-C2S-DERIVE-KEY | Derive Key | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.14 |
| KMIPKIT-ELEM-OP-C2S-DESTROY | Destroy | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.15 |
| KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS | Discover Versions | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.16 |
| KMIPKIT-ELEM-OP-C2S-ENCRYPT | Encrypt | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.17 |
| KMIPKIT-ELEM-OP-C2S-EXPORT | Export | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.18 |
| KMIPKIT-ELEM-OP-C2S-GET | Get | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.19 |
| KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST | Get Attribute List | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.21 |
| KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES | Get Attributes | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.20 |
| KMIPKIT-ELEM-OP-C2S-GET-CONSTRAINTS | Get Constraints | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.22 |
| KMIPKIT-ELEM-OP-C2S-GET-USAGE-ALLOCATION | Get Usage Allocation | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.23 |
| KMIPKIT-ELEM-OP-C2S-HASH | Hash | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.24 |
| KMIPKIT-ELEM-OP-C2S-IMPORT | Import | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.25 |
| KMIPKIT-ELEM-OP-C2S-INTEROP | Interop | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.26 |
| KMIPKIT-ELEM-OP-C2S-JOIN-SPLIT-KEY | Join Split Key | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.27 |
| KMIPKIT-ELEM-OP-C2S-LOCATE | Locate | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.28 |
| KMIPKIT-ELEM-OP-C2S-LOG | Log | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.29 |
| KMIPKIT-ELEM-OP-C2S-LOGIN | Login | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.30 |
| KMIPKIT-ELEM-OP-C2S-LOGOUT | Logout | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.31 |
| KMIPKIT-ELEM-OP-C2S-MAC | MAC | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.32 |
| KMIPKIT-ELEM-OP-C2S-MAC-VERIFY | MAC Verify | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.33 |
| KMIPKIT-ELEM-OP-C2S-MODIFY-ATTRIBUTE | Modify Attribute | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.34 |
| KMIPKIT-ELEM-OP-C2S-OBTAIN-LEASE | Obtain Lease | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.35 |
| KMIPKIT-ELEM-OP-C2S-PING | Ping | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.36 |
| KMIPKIT-ELEM-OP-C2S-PKCS-11 | PKCS#11 | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.37 |
| KMIPKIT-ELEM-OP-C2S-POLL | Poll | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.38 |
| KMIPKIT-ELEM-OP-C2S-PROCESS | Process | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.39 |
| KMIPKIT-ELEM-OP-C2S-QUERY | Query | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.40 |
| KMIPKIT-ELEM-OP-C2S-QUERY-ASYNCHRONOUS-REQUESTS | Query Asynchronous Requests | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.41 |
| KMIPKIT-ELEM-OP-C2S-RE-CERTIFY | Re-certify | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.45 |
| KMIPKIT-ELEM-OP-C2S-RE-KEY | Re-key | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.46 |
| KMIPKIT-ELEM-OP-C2S-RE-KEY-KEY-PAIR | Re-key Key Pair | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.47 |
| KMIPKIT-ELEM-OP-C2S-RE-PROVISION | Re-Provision | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.48 |
| KMIPKIT-ELEM-OP-C2S-RECOVER | Recover | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.42 |
| KMIPKIT-ELEM-OP-C2S-REGISTER | Register | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.43 |
| KMIPKIT-ELEM-OP-C2S-REVOKE | Revoke | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.44 |
| KMIPKIT-ELEM-OP-C2S-RNG-RETRIEVE | RNG Retrieve | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.49 |
| KMIPKIT-ELEM-OP-C2S-RNG-SEED | RNG Seed | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.50 |
| KMIPKIT-ELEM-OP-C2S-SET-ATTRIBUTE | Set Attribute | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.51 |
| KMIPKIT-ELEM-OP-C2S-SET-CONSTRAINTS | Set Constraints | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.52 |
| KMIPKIT-ELEM-OP-C2S-SET-DEFAULTS | Set Defaults | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.53 |
| KMIPKIT-ELEM-OP-C2S-SET-ENDPOINT-ROLE | Set Endpoint Role | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.54 |
| KMIPKIT-ELEM-OP-C2S-SIGN | Sign | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.55 |
| KMIPKIT-ELEM-OP-C2S-SIGNATURE-VERIFY | Signature Verify | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.56 |
| KMIPKIT-ELEM-OP-C2S-VALIDATE | Validate | operation | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §6.1.57 |
| KMIPKIT-ELEM-OP-S2C-DISCOVER-VERSIONS | Discover Versions | operation | server\_to\_client | client\_1\_1 |  |  | KMIPKIT-SRC-spec §6.2.1 |
| KMIPKIT-ELEM-OP-S2C-NOTIFY | Notify | operation | server\_to\_client | client\_1\_1 |  |  | KMIPKIT-SRC-spec §6.2.2 |
| KMIPKIT-ELEM-OP-S2C-PUT | Put | operation | server\_to\_client | client\_1\_1 |  |  | KMIPKIT-SRC-spec §6.2.3 |
| KMIPKIT-ELEM-OP-S2C-QUERY | Query | operation | server\_to\_client | client\_1\_1 |  |  | KMIPKIT-SRC-spec §6.2.4 |
| KMIPKIT-ELEM-OP-S2C-SET-ENDPOINT-ROLE | Set Endpoint Role | operation | server\_to\_client | client\_1\_1 |  |  | KMIPKIT-SRC-spec §6.2.5 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-1-ASYNCHRONOUS-CORRELATION-VALUES | Asynchronous Correlation Values | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.1 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-10-DATA-LENGTH | Data Length | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.10 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-11-DEFAULTS-INFORMATION | Defaults Information | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.11 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-12-DERIVATION-PARAMETERS | Derivation Parameters | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.12 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-13-EXTENSION-INFORMATION | Extension Information | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.13 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-14-FINAL-INDICATOR | Final Indicator | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.14 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-15-INTEROP-FUNCTION | Interop Function | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.15 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-16-INTEROP-IDENTIFIER | Interop identifier | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.16 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-17-INIT-INDICATOR | Init Indicator | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.17 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-18-KEY-WRAPPING-SPECIFICATION | Key Wrapping Specification | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.18 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-19-LOG-MESSAGE | Log Message | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.19 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-2-ASYNCHRONOUS-REQUEST | Asynchronous Request | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.2 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-20-MAC-DATA | MAC Data | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.20 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-21-OBJECTS | Objects | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.21 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-22-OBJECT-DEFAULTS | Object Defaults | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.22 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-23-OBJECT-GROUPS | Object Groups | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.23 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-24-OBJECT-TYPES | Object Types | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.24 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-25-OPERATIONS | Operations | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.25 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-26-PKCS-11-FUNCTION | PKCS#11 Function | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.26 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-27-PKCS-11-INPUT-PARAMETERS | PKCS#11 Input Parameters | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.27 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-28-PKCS-11-INTERFACE | PKCS#11 Interface | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.28 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-29-PKCS-11-OUTPUT-PARAMETERS | PKCS#11 Output Parameters | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.29 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-3-AUTHENTICATED-ENCRYPTION-ADDITIONAL-DATA | Authenticated Encryption Additional Data | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.3 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-30-PKCS-11-RETURN-CODE | PKCS#11 Return Code | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.30 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-31-PROFILE-INFORMATION | Profile Information | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.31 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-32-PROFILE-VERSION | Profile Version | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.32 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-33-PROTECTION-STORAGE-MASKS | Protection Storage Masks | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.33 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-34-RIGHT | Right | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.34 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-35-RIGHTS | Rights | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.35 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-36-RNG-PARAMETERS | RNG Parameters | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.36 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-37-SERVER-INFORMATION | Server Information | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.37 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-38-SIGNATURE-DATA | Signature Data | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.38 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-39-TICKET | Ticket | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.39 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-4-AUTHENTICATED-ENCRYPTION-TAG | Authenticated Encryption Tag | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.4 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-40-USAGE-LIMITS | Usage Limits | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.40 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-41-VALIDATION-INFORMATION | Validation Information | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.41 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-5-CAPABILITY-INFORMATION | Capability Information | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.5 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-6-CONSTRAINT | Constraint | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.6 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-7-CONSTRAINTS | Constraints | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.7 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-8-CORRELATION-VALUE | Correlation Value | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.8 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-9-DATA | Data | operation\_structure | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.9 |
| KMIPKIT-ELEM-OPTION-ASYNCHRONOUS-INDICATOR | Asynchronous Indicator | option | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.2, KMIPKIT-SRC-spec §11.3 |
| KMIPKIT-ELEM-OPTION-BATCH-ERROR-CONTINUATION-OPTION | Batch Error Continuation Option | option | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.6, KMIPKIT-SRC-spec §11.5 |
| KMIPKIT-ELEM-OPTION-BATCH-ORDER-OPTION | Batch Order Option | option | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.8 |
| KMIPKIT-ELEM-RESULT-CANCELLATION-RESULT | Cancellation Result | result | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §11.7 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-1-CERTIFICATE-CERTIFICATE-TYPE | Certificate Type | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.1 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-1-CERTIFICATE-CERTIFICATE-VALUE | Certificate Value | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.1 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-2-CERTIFICATE-REQUEST-CERTIFICATE-REQUEST-TYPE | Certificate Request Type | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.2 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-2-CERTIFICATE-REQUEST-CERTIFICATE-REQUEST-VALUE | Certificate Request Value | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.2 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-3-OPAQUE-OBJECT-OPAQUE-DATA-TYPE | Opaque Data Type | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.3 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-3-OPAQUE-OBJECT-OPAQUE-DATA-VALUE | Opaque Data Value | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.3 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-4-PGP-KEY-KEY-BLOCK | Key Block | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.4 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-4-PGP-KEY-PGP-KEY-VERSION | PGP Key Version | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.4 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-5-PRIVATE-KEY-KEY-BLOCK | Key Block | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-6-PUBLIC-KEY-KEY-BLOCK | Key Block | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.6 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-7-SECRET-DATA-KEY-BLOCK | Key Block | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.7 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-7-SECRET-DATA-SECRET-DATA-TYPE | Secret Data Type | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.7 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-8-SPLIT-KEY-KEY-BLOCK | Key Block | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.8 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-8-SPLIT-KEY-KEY-PART-IDENTIFIER | Key Part Identifier | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.8 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-8-SPLIT-KEY-PRIME-FIELD-SIZE | Prime Field Size | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.8 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-8-SPLIT-KEY-SPLIT-KEY-METHOD | Split Key Method | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.8 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-8-SPLIT-KEY-SPLIT-KEY-PARTS | Split Key Parts | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.8 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-8-SPLIT-KEY-SPLIT-KEY-THRESHOLD | Split Key Threshold | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.8 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-9-SYMMETRIC-KEY-KEY-BLOCK | Key Block | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §2.9 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-1-KEY-BLOCK-CRYPTOGRAPHIC-ALGORITHM | Cryptographic Algorithm | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-1-KEY-BLOCK-CRYPTOGRAPHIC-LENGTH | Cryptographic Length | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-1-KEY-BLOCK-KEY-COMPRESSION-TYPE | Key Compression Type | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-1-KEY-BLOCK-KEY-FORMAT-TYPE | Key Format Type | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-1-KEY-BLOCK-KEY-VALUE | Key Value | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-1-KEY-BLOCK-KEY-WRAPPING-DATA | Key Wrapping Data | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-10-KEY-MATERIAL-G | G | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.10 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-10-KEY-MATERIAL-J | J | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.10 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-10-KEY-MATERIAL-P | P | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.10 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-10-KEY-MATERIAL-Q | Q | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.10 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-10-KEY-MATERIAL-Y | Y | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.10 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-11-KEY-MATERIAL-D | D | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-11-KEY-MATERIAL-RECOMMENDED-CURVE | Recommended Curve | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-12-KEY-MATERIAL-Q-STRING | Q String | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.12 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-12-KEY-MATERIAL-RECOMMENDED-CURVE | Recommended Curve | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.12 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-2-KEY-VALUE-ATTRIBUTES | Attributes | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.2 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-2-KEY-VALUE-KEY-MATERIAL | Key Material | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.2 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-3-ENCRYPTION-KEY-INFORMATION-CRYPTOGRAPHIC-PARAMETERS | Cryptographic Parameters | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-3-ENCRYPTION-KEY-INFORMATION-UNIQUE-IDENTIFIER | Unique Identifier | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-3-KEY-WRAPPING-DATA-ENCODING-OPTION | Encoding Option | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-3-KEY-WRAPPING-DATA-ENCRYPTION-KEY-INFORMATION | Encryption Key Information | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-3-KEY-WRAPPING-DATA-IV-COUNTER-NONCE | IV/Counter/Nonce | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-3-KEY-WRAPPING-DATA-MAC-SIGNATURE | MAC/Signature | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-3-KEY-WRAPPING-DATA-MAC-SIGNATURE-KEY-INFORMATION | MAC/Signature Key Information | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-3-KEY-WRAPPING-DATA-WRAPPING-METHOD | Wrapping Method | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-3-MAC-SIGNATURE-KEY-INFORMATION-CRYPTOGRAPHIC-PARAMETERS | Cryptographic Parameters | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-3-MAC-SIGNATURE-KEY-INFORMATION-UNIQUE-IDENTIFIER | Unique Identifier | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-4-KEY-MATERIAL-KEY | Key | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.4 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-5-KEY-MATERIAL-G | G | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-5-KEY-MATERIAL-P | P | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-5-KEY-MATERIAL-Q | Q | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-5-KEY-MATERIAL-X | X | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-6-KEY-MATERIAL-G | G | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.6 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-6-KEY-MATERIAL-P | P | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.6 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-6-KEY-MATERIAL-Q | Q | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.6 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-6-KEY-MATERIAL-Y | Y | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.6 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-7-KEY-MATERIAL-CRT-COEFFICIENT | CRT Coefficient | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.7 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-7-KEY-MATERIAL-MODULUS | Modulus | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.7 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-7-KEY-MATERIAL-P | P | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.7 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-7-KEY-MATERIAL-PRIME-EXPONENT-P | Prime Exponent P | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.7 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-7-KEY-MATERIAL-PRIME-EXPONENT-Q | Prime Exponent Q | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.7 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-7-KEY-MATERIAL-PRIVATE-EXPONENT | Private Exponent | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.7 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-7-KEY-MATERIAL-PUBLIC-EXPONENT | Public Exponent | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.7 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-7-KEY-MATERIAL-Q | Q | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.7 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-8-KEY-MATERIAL-MODULUS | Modulus | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.8 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-8-KEY-MATERIAL-PUBLIC-EXPONENT | Public Exponent | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.8 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-9-KEY-MATERIAL-G | G | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.9 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-9-KEY-MATERIAL-J | J | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.9 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-9-KEY-MATERIAL-P | P | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.9 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-9-KEY-MATERIAL-Q | Q | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.9 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-9-KEY-MATERIAL-X | X | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §3.9 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-14-QLENGTH | Qlength | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.14 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-14-RECOMMENDED-CURVE | Recommended Curve | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.14 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-BLOCK-CIPHER-MODE | Block Cipher Mode | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-COUNTER-LENGTH | Counter Length | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-CRYPTOGRAPHIC-ALGORITHM | Cryptographic Algorithm | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-DIGITAL-SIGNATURE-ALGORITHM | Digital Signature Algorithm | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-FIXED-FIELD-LENGTH | Fixed Field Length | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-HASHING-ALGORITHM | Hashing Algorithm | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-INITIAL-COUNTER-VALUE | Initial Counter Value | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-INVOCATION-FIELD-LENGTH | Invocation Field Length | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-IV-LENGTH | IV Length | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-KEY-ROLE-TYPE | Key Role Type | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-MASK-GENERATOR | Mask Generator | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-MASK-GENERATOR-HASHING-ALGORITHM | Mask Generator Hashing Algorithm | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-P-SOURCE | P Source | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-PADDING-METHOD | Padding Method | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-RANDOM-IV | Random IV | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-SALT-LENGTH | Salt Length | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-TAG-LENGTH | Tag Length | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-TRAILER-FIELD | Trailer Field | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-2-ALTERNATIVE-NAME-TYPE | Alternative Name Type | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.2 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-2-ALTERNATIVE-NAME-VALUE | Alternative Name Value | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.2 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-21-DIGEST-VALUE | Digest Value | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.21 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-21-HASHING-ALGORITHM | Hashing Algorithm | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.21 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-21-KEY-FORMAT-TYPE | Key Format Type | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.21 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-27-KEY-VALUE-LOCATION-TYPE | Key Value Location Type | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.27 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-27-KEY-VALUE-LOCATION-VALUE | Key Value Location Value | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.27 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-31-LINK-TYPE | Link Type | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.31 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-31-LINKED-OBJECT-IDENTIFIER | Linked Object Identifier | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.31 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-32-NAME-TYPE | Name Type | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.32 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-32-NAME-VALUE | Name Value | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.32 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-4-APPLICATION-DATA | Application Data | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.4 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-4-APPLICATION-NAMESPACE | Application Namespace | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.4 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-47-REVOCATION-MESSAGE | Revocation Message | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.47 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-47-REVOCATION-REASON-CODE | Revocation Reason Code | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.47 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-53-ROTATE-NAME-TYPE | Rotate Name Type | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.53 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-53-ROTATE-NAME-VALUE | Rotate Name Value | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.53 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-60-ATTRIBUTE-NAME | Attribute Name | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.60 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-60-ATTRIBUTE-VALUE | Attribute Value | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.60 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-60-VENDOR-IDENTIFICATION | Vendor Identification | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.60 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-61-CERTIFICATE-SERIAL-NUMBER | Certificate Serial Number | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.61 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-61-ISSUER-DISTINGUISHED-NAME | Issuer Distinguished Name | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.61 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-62-ISSUER-ALTERNATIVE-NAME | Issuer Alternative Name | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.62 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-62-ISSUER-DISTINGUISHED-NAME | Issuer Distinguished Name | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.62 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-63-SUBJECT-ALTERNATIVE-NAME | Subject Alternative Name | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.63 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-63-SUBJECT-DISTINGUISHED-NAME | Subject Distinguished Name | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §4.63 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-5-1-ANY-ATTRIBUTE-IN-4-OBJECT-ATTRIBUTES | Any attribute in §4 - Object Attributes | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §5.1 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-5-2-ANY-ATTRIBUTE-IN-4-OBJECT-ATTRIBUTES | Any attribute in §4 - Object Attributes | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §5.2 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-5-3-ANY-ATTRIBUTE-IN-4-OBJECT-ATTRIBUTES | Any attribute in §4 - Object Attributes | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §5.3 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-5-4-ANY-ATTRIBUTE-IN-4-OBJECT-ATTRIBUTES | Any attribute in §4 - Object Attributes | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §5.4 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-5-5-ATTRIBUTE-NAME | Attribute Name | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §5.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-5-5-ATTRIBUTE-REFERENCE | Attribute Reference | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §5.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-5-5-VENDOR-IDENTIFICATION | Vendor Identification | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §5.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-5-6-ANY-ATTRIBUTE-IN-4-OBJECT-ATTRIBUTES | Any attribute in §4 - Object Attributes | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §5.6 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-5-7-ANY-ATTRIBUTE-IN-4-OBJECT-ATTRIBUTES | Any attribute in §4 - Object Attributes | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §5.7 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-1-ASYNCHRONOUS-CORRELATION-VALUE | Asynchronous Correlation Value | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.1 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-11-OBJECT-DEFAULTS | Object Defaults | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-12-CRYPTOGRAPHIC-PARAMETERS | Cryptographic Parameters, | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.12 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-12-DERIVATION-DATA | Derivation Data | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.12 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-12-INITIALIZATION-VECTOR | Initialization Vector | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.12 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-12-ITERATION-COUNT | Iteration Count | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.12 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-12-SALT | Salt | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.12 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-13-EXTENSION-ATTRIBUTE | Extension Attribute | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.13 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-13-EXTENSION-DESCRIPTION | Extension Description | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.13 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-13-EXTENSION-ENUMERATION | Extension Enumeration | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.13 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-13-EXTENSION-NAME | Extension Name | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.13 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-13-EXTENSION-PARENT-STRUCTURE-TAG | Extension Parent Structure Tag | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.13 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-13-EXTENSION-TAG | Extension Tag | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.13 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-13-EXTENSION-TYPE | Extension Type | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.13 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-18-ATTRIBUTE-NAME | Attribute Name | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.18 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-18-ENCODING-OPTION | Encoding Option | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.18 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-18-ENCRYPTION-KEY-INFORMATION | Encryption Key Information | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.18 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-18-MAC-SIGNATURE-KEY-INFORMATION | MAC/Signature Key Information | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.18 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-18-WRAPPING-METHOD | Wrapping Method | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.18 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-2-ASYNCHRONOUS-CORRELATION-VALUE | Asynchronous Correlation Value | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.2 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-2-OPERATION | Operation | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.2 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-2-PROCESSING-STAGE | Processing Stage | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.2 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-2-SUBMISSION-DATE | Submission Date | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.2 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-21-UNIQUE-IDENTIFIER | Unique Identifier | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.21 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-22-ATTRIBUTES | Attributes | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.22 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-22-OBJECT-GROUPS | Object Groups | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.22 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-22-OBJECT-TYPE-OBJECTTYPES | Object Type \| ObjectTypes | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.22 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-23-OBJECT-GROUP | Object Group | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.23 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-24-OBJECT-TYPE | Object Type | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.24 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-25-OPERATION | Operation | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.25 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-26-PKCS-11-FUNCTION | PKCS#11 Function | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.26 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-27-PKCS-11-INPUT-PARAMETERS | PKCS#11 Input Parameters | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.27 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-28-PKCS-11-INTERFACE | PKCS#11 Interface | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.28 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-29-PKCS-11-OUTPUT-PARAMETERS | PKCS#11 Output Parameters | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.29 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-30-PKCS-11-RETURN-CODE | PKCS#11 Return Code | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.30 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-31-PROFILE-NAME | Profile Name | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.31 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-31-PROFILE-VERSION | Profile Version | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.31 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-31-SERVER-PORT | Server Port | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.31 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-31-SERVER-URI | Server URI | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.31 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-32-PROFILE-VERSION-MAJOR | Profile Version Major | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.32 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-32-PROFILE-VERSION-MINOR | Profile Version Minor | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.32 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-34-OBJECT-GROUPS | Object Groups | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.34 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-34-OBJECTS | Objects | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.34 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-34-OPERATIONS | Operations | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.34 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-34-USAGE-LIMITS | Usage Limits | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.34 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-35-RIGHT | Right | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.35 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-36-CRYPTOGRAPHIC-ALGORITHM | Cryptographic Algorithm | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.36 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-36-CRYPTOGRAPHIC-LENGTH | Cryptographic Length | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.36 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-36-DRBG-ALGORITHM | DRBG Algorithm | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.36 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-36-FIPS186-VARIATION | FIPS186 Variation | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.36 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-36-HASHING-ALGORITHM | Hashing Algorithm | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.36 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-36-PREDICTION-RESISTANCE | Prediction Resistance | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.36 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-36-RECOMMENDED-CURVE | Recommended Curve | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.36 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-36-RNG-ALGORITHM | RNG Algorithm | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.36 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-37-ALTERNATIVE-FAILOVER-ENDPOINTS | Alternative failover endpoints | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.37 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-37-BUILD-DATE | Build date | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.37 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-37-BUILD-LEVEL | Build level | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.37 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-37-CLUSTER-INFO | Cluster info | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.37 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-37-PRODUCT-NAME | Product name | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.37 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-37-SERVER-LOAD | Server load | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.37 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-37-SERVER-NAME | Server name | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.37 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-37-SERVER-SERIAL-NUMBER | Server serial number | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.37 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-37-SERVER-VERSION | Server version | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.37 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-37-VENDOR-SPECIFIC | Vendor-Specific | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.37 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-39-TICKET-TYPE | Ticket Type | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.39 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-39-TICKET-VALUE | Ticket Value | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.39 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-40-USAGE-LIMITS-COUNT | Usage Limits Count | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.40 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-40-USAGE-LIMITS-TOTAL | Usage Limits Total | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.40 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-40-USAGE-LIMITS-UNIT | Usage Limits Unit | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.40 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-41-VALIDATION-AUTHORITY-COUNTRY | Validation Authority Country | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.41 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-41-VALIDATION-AUTHORITY-TYPE | Validation Authority Type | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.41 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-41-VALIDATION-AUTHORITY-URI | Validation Authority URI | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.41 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-41-VALIDATION-CERTIFICATE-IDENTIFIER | Validation Certificate Identifier | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.41 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-41-VALIDATION-CERTIFICATE-URI | Validation Certificate URI | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.41 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-41-VALIDATION-LEVEL | Validation Level | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.41 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-41-VALIDATION-PROFILE | Validation Profile | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.41 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-41-VALIDATION-TYPE | Validation Type | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.41 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-41-VALIDATION-VENDOR-URI | Validation Vendor URI | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.41 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-41-VALIDATION-VERSION-MAJOR | Validation Version Major | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.41 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-41-VALIDATION-VERSION-MINOR | Validation Version Minor | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.41 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-5-ASYNCHRONOUS-CAPABILITY | Asynchronous Capability | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-5-ATTESTATION-CAPABILITY | Attestation Capability | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-5-BATCH-CONTINUE-CAPABILITY | Batch Continue Capability | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-5-BATCH-UNDO-CAPABILITY | Batch Undo Capability | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-5-DESTROY-ACTION | Destroy Action | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-5-QUANTUM-SAFE-CAPABILITY | Quantum Safe Capability | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-5-RNG-MODE | RNG Mode | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-5-SHREDDING-ALGORITHM | Shredding Algorithm | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-5-STREAMING-CAPABILITY | Streaming Capability | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-5-UNWRAP-MODE | Unwrap Mode | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-6-ATTRIBUTES | Attributes | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.6 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-6-OBJECT-GROUPS | Object Groups | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.6 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-6-OBJECT-TYPES | Object Types | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.6 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-7-CONSTRAINT | Constraint | structure\_member | both | client\_1\_0 |  |  | KMIPKIT-SRC-spec §7.7 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ATTESTATION-ATTESTATION-ASSERTION | Attestation Assertion | structure\_member | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ATTESTATION-ATTESTATION-MEASUREMENT | Attestation Measurement | structure\_member | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ATTESTATION-ATTESTATION-TYPE | Attestation Type | structure\_member | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ATTESTATION-NONCE | Nonce | structure\_member | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-CREDENTIAL-CREDENTIAL-TYPE | Credential Type | structure\_member | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-CREDENTIAL-CREDENTIAL-VALUE | Credential Value | structure\_member | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-DEVICE-DEVICE-IDENTIFIER | Device Identifier | structure\_member | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-DEVICE-DEVICE-SERIAL-NUMBER | Device Serial Number | structure\_member | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-DEVICE-MACHINE-IDENTIFIER | Machine Identifier | structure\_member | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-DEVICE-MEDIA-IDENTIFIER | Media Identifier | structure\_member | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-DEVICE-NETWORK-IDENTIFIER | Network Identifier | structure\_member | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-DEVICE-PASSWORD | Password | structure\_member | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-HASHED-PASSWORD-HASHED-PASSWORD | Hashed Password | structure\_member | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-HASHED-PASSWORD-HASHING-ALGORITHM | Hashing Algorithm | structure\_member | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-HASHED-PASSWORD-TIMESTAMP | Timestamp | structure\_member | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-HASHED-PASSWORD-USERNAME | Username | structure\_member | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ONE-TIME-PASSWORD-ONE-TIME-PASSWORD | One Time Password | structure\_member | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ONE-TIME-PASSWORD-PASSWORD | Password | structure\_member | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ONE-TIME-PASSWORD-USERNAME | Username | structure\_member | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-TICKET-TICKET | Ticket | structure\_member | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-USERNAME-AND-PASSWORD-PASSWORD | Password | structure\_member | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-USERNAME-AND-PASSWORD-USERNAME | Username | structure\_member | client\_to\_server | client\_1\_0 |  |  | KMIPKIT-SRC-spec §9.11 |

## Profile states

| Profile | Name | Role | Applicability | Claim state | Source | Source clauses | Requirements | Elements | Dependencies | Mandatory / optional tests | Transport | Encoding |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| KMIPKIT-PROFILE-AES-XTS-CLIENT | AES XTS Client | client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.13.1, KMIPKIT-SRC-profiles §5.13.3, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.30 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.13.1-001, KMIPKIT-CLAUSE-PROF-5.13.1-002, KMIPKIT-CLAUSE-PROF-5.13.1-003, KMIPKIT-CLAUSE-PROF-6.1-001, KMIPKIT-CLAUSE-PROF-6.1-002, KMIPKIT-CLAUSE-PROF-6.1-003, KMIPKIT-CLAUSE-PROF-6.1-004, KMIPKIT-CLAUSE-PROF-6.30-001, KMIPKIT-CLAUSE-PROF-6.30-002, KMIPKIT-CLAUSE-PROF-6.30-003, KMIPKIT-CLAUSE-PROF-6.30-004, KMIPKIT-CLAUSE-PROF-6.30-005 | KMIPKIT-REQ-PROF-3.1-001, KMIPKIT-REQ-PROF-5.13.1-001, KMIPKIT-REQ-PROF-5.13.1-003, KMIPKIT-REQ-PROF-6.1-001, KMIPKIT-REQ-PROF-6.1-002, KMIPKIT-REQ-PROF-6.1-003, KMIPKIT-REQ-PROF-6.1-004, KMIPKIT-REQ-PROF-6.30-001, KMIPKIT-REQ-PROF-6.30-002, KMIPKIT-REQ-PROF-6.30-003, KMIPKIT-REQ-PROF-6.30-004, KMIPKIT-REQ-PROF-6.30-005 | KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-QUERY | KMIPKIT-PROFILE-BASELINE-CLIENT | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-13-3-1 \(AX-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-13-3-2 \(AX-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\) |  |  |
| KMIPKIT-PROFILE-AES-XTS-SERVER | AES XTS Server | server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.13.2, KMIPKIT-SRC-profiles §5.13.3, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.31 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.13.2-001, KMIPKIT-CLAUSE-PROF-5.13.2-002, KMIPKIT-CLAUSE-PROF-5.13.2-003, KMIPKIT-CLAUSE-PROF-5.13.2-004, KMIPKIT-CLAUSE-PROF-5.13.2-005, KMIPKIT-CLAUSE-PROF-5.13.2-006, KMIPKIT-CLAUSE-PROF-5.13.2-007, KMIPKIT-CLAUSE-PROF-6.31-001, KMIPKIT-CLAUSE-PROF-6.31-002, KMIPKIT-CLAUSE-PROF-6.31-003, KMIPKIT-CLAUSE-PROF-6.31-004, KMIPKIT-CLAUSE-PROF-6.31-005 | KMIPKIT-REQ-PROF-3.1-001 | KMIPKIT-ELEM-OP-C2S-ACTIVATE, KMIPKIT-ELEM-OP-C2S-ADD-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-ADJUST-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-CHECK, KMIPKIT-ELEM-OP-C2S-CREATE, KMIPKIT-ELEM-OP-C2S-DELETE-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-DESTROY, KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-C2S-EXPORT, KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-IMPORT, KMIPKIT-ELEM-OP-C2S-INTEROP, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-LOG, KMIPKIT-ELEM-OP-C2S-MODIFY-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-QUERY, KMIPKIT-ELEM-OP-C2S-REGISTER, KMIPKIT-ELEM-OP-C2S-REVOKE, KMIPKIT-ELEM-OP-C2S-SET-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-SET-ENDPOINT-ROLE, KMIPKIT-ELEM-OP-S2C-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-S2C-NOTIFY, KMIPKIT-ELEM-OP-S2C-PUT, KMIPKIT-ELEM-OP-S2C-QUERY, KMIPKIT-ELEM-OP-S2C-SET-ENDPOINT-ROLE | KMIPKIT-PROFILE-BASELINE-SERVER | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-13-3-1 \(AX-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-13-3-2 \(AX-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\) |  |  |
| KMIPKIT-PROFILE-ASYMMETRIC-KEY-LIFECYCLE-CLIENT | Asymmetric Key Lifecycle Client | client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.8.1, KMIPKIT-SRC-profiles §5.8.3, KMIPKIT-SRC-profiles §5.8.4, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.16 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.8.1-001, KMIPKIT-CLAUSE-PROF-5.8.1-002, KMIPKIT-CLAUSE-PROF-5.8.1-003, KMIPKIT-CLAUSE-PROF-6.1-001, KMIPKIT-CLAUSE-PROF-6.1-002, KMIPKIT-CLAUSE-PROF-6.1-003, KMIPKIT-CLAUSE-PROF-6.1-004, KMIPKIT-CLAUSE-PROF-6.16-001, KMIPKIT-CLAUSE-PROF-6.16-002, KMIPKIT-CLAUSE-PROF-6.16-003, KMIPKIT-CLAUSE-PROF-6.16-004, KMIPKIT-CLAUSE-PROF-6.16-005 | KMIPKIT-REQ-PROF-3.1-001, KMIPKIT-REQ-PROF-5.8.1-001, KMIPKIT-REQ-PROF-5.8.1-002, KMIPKIT-REQ-PROF-5.8.1-003, KMIPKIT-REQ-PROF-6.1-001, KMIPKIT-REQ-PROF-6.1-002, KMIPKIT-REQ-PROF-6.1-003, KMIPKIT-REQ-PROF-6.1-004, KMIPKIT-REQ-PROF-6.16-001, KMIPKIT-REQ-PROF-6.16-002, KMIPKIT-REQ-PROF-6.16-003, KMIPKIT-REQ-PROF-6.16-004, KMIPKIT-REQ-PROF-6.16-005 | KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-QUERY | KMIPKIT-PROFILE-BASELINE-CLIENT | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-8-3-1 \(AKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-8-3-2 \(AKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-8-3-3 \(AKLC-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-8-4-1 \(AKLC-O-1-21; optional\) |  |  |
| KMIPKIT-PROFILE-ASYMMETRIC-KEY-LIFECYCLE-SERVER | Asymmetric Key Lifecycle Server | server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.8.2, KMIPKIT-SRC-profiles §5.8.3, KMIPKIT-SRC-profiles §5.8.4, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.17 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.8.2-001, KMIPKIT-CLAUSE-PROF-5.8.2-002, KMIPKIT-CLAUSE-PROF-5.8.2-003, KMIPKIT-CLAUSE-PROF-5.8.2-004, KMIPKIT-CLAUSE-PROF-5.8.2-005, KMIPKIT-CLAUSE-PROF-5.8.2-006, KMIPKIT-CLAUSE-PROF-6.17-001, KMIPKIT-CLAUSE-PROF-6.17-002, KMIPKIT-CLAUSE-PROF-6.17-003, KMIPKIT-CLAUSE-PROF-6.17-004, KMIPKIT-CLAUSE-PROF-6.17-005 | KMIPKIT-REQ-PROF-3.1-001 | KMIPKIT-ELEM-OP-C2S-ACTIVATE, KMIPKIT-ELEM-OP-C2S-ADD-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-ADJUST-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-CHECK, KMIPKIT-ELEM-OP-C2S-DELETE-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-DESTROY, KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-C2S-EXPORT, KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-IMPORT, KMIPKIT-ELEM-OP-C2S-INTEROP, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-LOG, KMIPKIT-ELEM-OP-C2S-MODIFY-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-QUERY, KMIPKIT-ELEM-OP-C2S-REGISTER, KMIPKIT-ELEM-OP-C2S-REVOKE, KMIPKIT-ELEM-OP-C2S-SET-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-SET-ENDPOINT-ROLE, KMIPKIT-ELEM-OP-S2C-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-S2C-NOTIFY, KMIPKIT-ELEM-OP-S2C-PUT, KMIPKIT-ELEM-OP-S2C-QUERY, KMIPKIT-ELEM-OP-S2C-SET-ENDPOINT-ROLE | KMIPKIT-PROFILE-BASELINE-SERVER | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-8-3-1 \(AKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-8-3-2 \(AKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-8-3-3 \(AKLC-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-8-4-1 \(AKLC-O-1-21; optional\) |  |  |
| KMIPKIT-PROFILE-BASELINE-CLIENT | Baseline Client | client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.1.3, KMIPKIT-SRC-profiles §5.6.3, KMIPKIT-SRC-profiles §6.1 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.1.1-001, KMIPKIT-CLAUSE-PROF-6.1-001, KMIPKIT-CLAUSE-PROF-6.1-002, KMIPKIT-CLAUSE-PROF-6.1-003, KMIPKIT-CLAUSE-PROF-6.1-004 | KMIPKIT-REQ-PROF-3.1-001, KMIPKIT-REQ-PROF-6.1-001, KMIPKIT-REQ-PROF-6.1-002, KMIPKIT-REQ-PROF-6.1-003, KMIPKIT-REQ-PROF-6.1-004 | KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-QUERY |  | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\) |  |  |
| KMIPKIT-PROFILE-BASELINE-SERVER | Baseline Server | server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.1.3, KMIPKIT-SRC-profiles §5.6.3, KMIPKIT-SRC-profiles §6.2 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.1.2-001, KMIPKIT-CLAUSE-PROF-6.2-001, KMIPKIT-CLAUSE-PROF-6.2-002, KMIPKIT-CLAUSE-PROF-6.2-003, KMIPKIT-CLAUSE-PROF-6.2-004 | KMIPKIT-REQ-PROF-3.1-001 | KMIPKIT-ELEM-OP-C2S-ACTIVATE, KMIPKIT-ELEM-OP-C2S-ADD-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-ADJUST-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-CHECK, KMIPKIT-ELEM-OP-C2S-DELETE-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-DESTROY, KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-C2S-EXPORT, KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-IMPORT, KMIPKIT-ELEM-OP-C2S-INTEROP, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-LOG, KMIPKIT-ELEM-OP-C2S-MODIFY-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-QUERY, KMIPKIT-ELEM-OP-C2S-REGISTER, KMIPKIT-ELEM-OP-C2S-REVOKE, KMIPKIT-ELEM-OP-C2S-SET-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-SET-ENDPOINT-ROLE, KMIPKIT-ELEM-OP-S2C-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-S2C-NOTIFY, KMIPKIT-ELEM-OP-S2C-PUT, KMIPKIT-ELEM-OP-S2C-QUERY, KMIPKIT-ELEM-OP-S2C-SET-ENDPOINT-ROLE |  | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\) |  |  |
| KMIPKIT-PROFILE-COMPLETE-SERVER | Complete Server | server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.2, KMIPKIT-SRC-profiles §5.3.2, KMIPKIT-SRC-profiles §5.4.3, KMIPKIT-SRC-profiles §5.5.3, KMIPKIT-SRC-profiles §5.6.2, KMIPKIT-SRC-profiles §5.7.4, KMIPKIT-SRC-profiles §5.8.2, KMIPKIT-SRC-profiles §5.9.4, KMIPKIT-SRC-profiles §5.9.5, KMIPKIT-SRC-profiles §5.9.6, KMIPKIT-SRC-profiles §5.10.2, KMIPKIT-SRC-profiles §5.11.2, KMIPKIT-SRC-profiles §5.12.5, KMIPKIT-SRC-profiles §5.13.2, KMIPKIT-SRC-profiles §5.16, KMIPKIT-SRC-profiles §5.18.4, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.3, KMIPKIT-SRC-profiles §6.5, KMIPKIT-SRC-profiles §6.7, KMIPKIT-SRC-profiles §6.9, KMIPKIT-SRC-profiles §6.11, KMIPKIT-SRC-profiles §6.15, KMIPKIT-SRC-profiles §6.17, KMIPKIT-SRC-profiles §6.21, KMIPKIT-SRC-profiles §6.22, KMIPKIT-SRC-profiles §6.23, KMIPKIT-SRC-profiles §6.25, KMIPKIT-SRC-profiles §6.27, KMIPKIT-SRC-profiles §6.29, KMIPKIT-SRC-profiles §6.31, KMIPKIT-SRC-profiles §6.33, KMIPKIT-SRC-profiles §6.35 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-3.2-001, KMIPKIT-CLAUSE-PROF-6.3-001, KMIPKIT-CLAUSE-PROF-6.3-002, KMIPKIT-CLAUSE-PROF-6.3-003, KMIPKIT-CLAUSE-PROF-6.3-004 | KMIPKIT-REQ-PROF-3.1-001, KMIPKIT-REQ-PROF-3.2-001 | KMIPKIT-ELEM-OP-C2S-ACTIVATE, KMIPKIT-ELEM-OP-C2S-ADD-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-ADJUST-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-CERTIFY, KMIPKIT-ELEM-OP-C2S-CHECK, KMIPKIT-ELEM-OP-C2S-CREATE, KMIPKIT-ELEM-OP-C2S-CREATE-KEY-PAIR, KMIPKIT-ELEM-OP-C2S-DECRYPT, KMIPKIT-ELEM-OP-C2S-DELETE-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-DESTROY, KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-C2S-ENCRYPT, KMIPKIT-ELEM-OP-C2S-EXPORT, KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-HASH, KMIPKIT-ELEM-OP-C2S-IMPORT, KMIPKIT-ELEM-OP-C2S-INTEROP, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-LOG, KMIPKIT-ELEM-OP-C2S-MAC, KMIPKIT-ELEM-OP-C2S-MAC-VERIFY, KMIPKIT-ELEM-OP-C2S-MODIFY-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-PKCS-11, KMIPKIT-ELEM-OP-C2S-QUERY, KMIPKIT-ELEM-OP-C2S-RE-CERTIFY, KMIPKIT-ELEM-OP-C2S-RE-KEY, KMIPKIT-ELEM-OP-C2S-RE-KEY-KEY-PAIR, KMIPKIT-ELEM-OP-C2S-REGISTER, KMIPKIT-ELEM-OP-C2S-REVOKE, KMIPKIT-ELEM-OP-C2S-RNG-RETRIEVE, KMIPKIT-ELEM-OP-C2S-RNG-SEED, KMIPKIT-ELEM-OP-C2S-SET-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-SET-ENDPOINT-ROLE, KMIPKIT-ELEM-OP-C2S-SIGN, KMIPKIT-ELEM-OP-C2S-SIGNATURE-VERIFY, KMIPKIT-ELEM-OP-S2C-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-S2C-NOTIFY, KMIPKIT-ELEM-OP-S2C-PUT, KMIPKIT-ELEM-OP-S2C-QUERY, KMIPKIT-ELEM-OP-S2C-SET-ENDPOINT-ROLE | KMIPKIT-PROFILE-AES-XTS-SERVER, KMIPKIT-PROFILE-ASYMMETRIC-KEY-LIFECYCLE-SERVER, KMIPKIT-PROFILE-BASELINE-SERVER, KMIPKIT-PROFILE-CRYPTOGRAPHIC-ADVANCED-SERVER, KMIPKIT-PROFILE-CRYPTOGRAPHIC-BASIC-SERVER, KMIPKIT-PROFILE-CRYPTOGRAPHIC-RNG-SERVER, KMIPKIT-PROFILE-HTTPS-SERVER, KMIPKIT-PROFILE-JSON-SERVER, KMIPKIT-PROFILE-OPAQUE-MANAGED-OBJECT-STORE-SERVER, KMIPKIT-PROFILE-PKCS11-SERVER, KMIPKIT-PROFILE-QUANTUM-SAFE-SERVER, KMIPKIT-PROFILE-STORAGE-ARRAY-SELF-ENCRYPTING-DRIVES-SERVER, KMIPKIT-PROFILE-SYMMETRIC-KEY-FOUNDRY-SERVER, KMIPKIT-PROFILE-SYMMETRIC-KEY-LIFECYCLE-SERVER, KMIPKIT-PROFILE-TAPE-LIBRARY-SERVER, KMIPKIT-PROFILE-XML-SERVER | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-10-3-1 \(OMOS-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-10-4-1 \(OMOS-O-1-21; optional\), KMIPKIT-TEST-PROF-5-11-3-1 \(SASED-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-11-3-2 \(SASED-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-11-3-3 \(SASED-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-12-6-1 \(TL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-12-6-2 \(TL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-12-6-3 \(TL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-13-3-1 \(AX-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-13-3-2 \(AX-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-17-1 \(QS-M-1-12; mandatory\), KMIPKIT-TEST-PROF-5-17-2 \(QS-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-18-5-1 \(PKCS11-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-3-3-1 \(MSGENC-HTTPS-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-4-4-1 \(MSGENC-XML-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-5-4-1 \(MSGENC-JSON-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-6-4-1 \(SKLC-O-1-21; optional\), KMIPKIT-TEST-PROF-5-7-5-1 \(SKFF-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-7-5-2 \(SKFF-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-7-5-3 \(SKFF-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-7-5-4 \(SKFF-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-7-6-1 \(SKFF-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-7-6-2 \(SKFF-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-7-6-3 \(SKFF-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-7-6-4 \(SKFF-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-7-7-1 \(SKFF-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-7-7-2 \(SKFF-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-7-7-3 \(SKFF-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-7-7-4 \(SKFF-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-8-3-1 \(AKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-8-3-2 \(AKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-8-3-3 \(AKLC-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-8-4-1 \(AKLC-O-1-21; optional\), KMIPKIT-TEST-PROF-5-9-10-1 \(CS-RNG-O-1-21; optional\), KMIPKIT-TEST-PROF-5-9-10-2 \(CS-RNG-O-2-21; optional\), KMIPKIT-TEST-PROF-5-9-10-3 \(CS-RNG-O-3-21; optional\), KMIPKIT-TEST-PROF-5-9-10-4 \(CS-RNG-O-4-21; optional\), KMIPKIT-TEST-PROF-5-9-7-1 \(CS-BC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-10 \(CS-BC-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-11 \(CS-BC-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-12 \(CS-BC-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-13 \(CS-BC-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-14 \(CS-BC-M-14-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-15 \(CS-BC-M-GCM-1-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-16 \(CS-BC-M-GCM-2-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-17 \(CS-BC-M-GCM-3-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-18 \(CS-BC-M-CHACHA20-1-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-19 \(CS-BC-M-CHACHA20-2-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-2 \(CS-BC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-20 \(CS-BC-M-CHACHA20-3-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-21 \(CS-BC-M-CHACHA20POLY1305-1-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-3 \(CS-BC-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-4 \(CS-BC-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-5 \(CS-BC-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-6 \(CS-BC-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-7 \(CS-BC-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-8 \(CS-BC-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-9 \(CS-BC-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-1 \(CS-AC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-10 \(CS-AC-M-OAEP-2-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-11 \(CS-AC-M-OAEP-3-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-12 \(CS-AC-M-OAEP-4-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-13 \(CS-AC-M-OAEP-5-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-14 \(CS-AC-M-OAEP-6-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-15 \(CS-AC-M-OAEP-7-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-16 \(CS-AC-M-OAEP-8-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-17 \(CS-AC-M-OAEP-9-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-18 \(CS-AC-M-OAEP-10-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-2 \(CS-AC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-3 \(CS-AC-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-4 \(CS-AC-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-5 \(CS-AC-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-6 \(CS-AC-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-7 \(CS-AC-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-8 \(CS-AC-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-9 \(CS-AC-M-OAEP-1-21; mandatory\), KMIPKIT-TEST-PROF-5-9-9-1 \(CS-RNG-M-1-21; mandatory\) |  |  |
| KMIPKIT-PROFILE-CRYPTOGRAPHIC-ADVANCED-CLIENT | Advanced Cryptographic Client | client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.9.2, KMIPKIT-SRC-profiles §5.9.8, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.19 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.9.2-001, KMIPKIT-CLAUSE-PROF-5.9.2-002, KMIPKIT-CLAUSE-PROF-5.9.2-003, KMIPKIT-CLAUSE-PROF-5.9.2-004, KMIPKIT-CLAUSE-PROF-6.1-001, KMIPKIT-CLAUSE-PROF-6.1-002, KMIPKIT-CLAUSE-PROF-6.1-003, KMIPKIT-CLAUSE-PROF-6.1-004, KMIPKIT-CLAUSE-PROF-6.19-001, KMIPKIT-CLAUSE-PROF-6.19-002, KMIPKIT-CLAUSE-PROF-6.19-003, KMIPKIT-CLAUSE-PROF-6.19-004, KMIPKIT-CLAUSE-PROF-6.19-005 | KMIPKIT-REQ-PROF-3.1-001, KMIPKIT-REQ-PROF-5.9.2-001, KMIPKIT-REQ-PROF-5.9.2-002, KMIPKIT-REQ-PROF-5.9.2-003, KMIPKIT-REQ-PROF-5.9.2-004, KMIPKIT-REQ-PROF-6.1-001, KMIPKIT-REQ-PROF-6.1-002, KMIPKIT-REQ-PROF-6.1-003, KMIPKIT-REQ-PROF-6.1-004, KMIPKIT-REQ-PROF-6.19-001, KMIPKIT-REQ-PROF-6.19-002, KMIPKIT-REQ-PROF-6.19-003, KMIPKIT-REQ-PROF-6.19-004, KMIPKIT-REQ-PROF-6.19-005 | KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-QUERY | KMIPKIT-PROFILE-BASELINE-CLIENT | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-1 \(CS-AC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-10 \(CS-AC-M-OAEP-2-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-11 \(CS-AC-M-OAEP-3-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-12 \(CS-AC-M-OAEP-4-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-13 \(CS-AC-M-OAEP-5-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-14 \(CS-AC-M-OAEP-6-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-15 \(CS-AC-M-OAEP-7-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-16 \(CS-AC-M-OAEP-8-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-17 \(CS-AC-M-OAEP-9-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-18 \(CS-AC-M-OAEP-10-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-2 \(CS-AC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-3 \(CS-AC-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-4 \(CS-AC-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-5 \(CS-AC-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-6 \(CS-AC-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-7 \(CS-AC-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-8 \(CS-AC-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-9 \(CS-AC-M-OAEP-1-21; mandatory\) |  |  |
| KMIPKIT-PROFILE-CRYPTOGRAPHIC-ADVANCED-SERVER | Advanced Cryptographic Server | server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.9.5, KMIPKIT-SRC-profiles §5.9.8, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.22 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.9.5-001, KMIPKIT-CLAUSE-PROF-5.9.5-002, KMIPKIT-CLAUSE-PROF-5.9.5-003, KMIPKIT-CLAUSE-PROF-5.9.5-004, KMIPKIT-CLAUSE-PROF-6.22-001, KMIPKIT-CLAUSE-PROF-6.22-002, KMIPKIT-CLAUSE-PROF-6.22-003, KMIPKIT-CLAUSE-PROF-6.22-004, KMIPKIT-CLAUSE-PROF-6.22-005 | KMIPKIT-REQ-PROF-3.1-001 | KMIPKIT-ELEM-OP-C2S-ACTIVATE, KMIPKIT-ELEM-OP-C2S-ADD-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-ADJUST-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-CHECK, KMIPKIT-ELEM-OP-C2S-DECRYPT, KMIPKIT-ELEM-OP-C2S-DELETE-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-DESTROY, KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-C2S-ENCRYPT, KMIPKIT-ELEM-OP-C2S-EXPORT, KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-HASH, KMIPKIT-ELEM-OP-C2S-IMPORT, KMIPKIT-ELEM-OP-C2S-INTEROP, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-LOG, KMIPKIT-ELEM-OP-C2S-MAC, KMIPKIT-ELEM-OP-C2S-MAC-VERIFY, KMIPKIT-ELEM-OP-C2S-MODIFY-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-QUERY, KMIPKIT-ELEM-OP-C2S-REGISTER, KMIPKIT-ELEM-OP-C2S-REVOKE, KMIPKIT-ELEM-OP-C2S-RNG-RETRIEVE, KMIPKIT-ELEM-OP-C2S-RNG-SEED, KMIPKIT-ELEM-OP-C2S-SET-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-SET-ENDPOINT-ROLE, KMIPKIT-ELEM-OP-C2S-SIGN, KMIPKIT-ELEM-OP-C2S-SIGNATURE-VERIFY, KMIPKIT-ELEM-OP-S2C-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-S2C-NOTIFY, KMIPKIT-ELEM-OP-S2C-PUT, KMIPKIT-ELEM-OP-S2C-QUERY, KMIPKIT-ELEM-OP-S2C-SET-ENDPOINT-ROLE | KMIPKIT-PROFILE-BASELINE-SERVER | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-1 \(CS-AC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-10 \(CS-AC-M-OAEP-2-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-11 \(CS-AC-M-OAEP-3-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-12 \(CS-AC-M-OAEP-4-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-13 \(CS-AC-M-OAEP-5-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-14 \(CS-AC-M-OAEP-6-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-15 \(CS-AC-M-OAEP-7-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-16 \(CS-AC-M-OAEP-8-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-17 \(CS-AC-M-OAEP-9-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-18 \(CS-AC-M-OAEP-10-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-2 \(CS-AC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-3 \(CS-AC-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-4 \(CS-AC-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-5 \(CS-AC-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-6 \(CS-AC-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-7 \(CS-AC-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-8 \(CS-AC-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-9-8-9 \(CS-AC-M-OAEP-1-21; mandatory\) |  |  |
| KMIPKIT-PROFILE-CRYPTOGRAPHIC-BASIC-CLIENT | Basic Cryptographic Client | client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.9.1, KMIPKIT-SRC-profiles §5.9.7, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.18 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.9.1-001, KMIPKIT-CLAUSE-PROF-5.9.1-002, KMIPKIT-CLAUSE-PROF-5.9.1-003, KMIPKIT-CLAUSE-PROF-5.9.1-004, KMIPKIT-CLAUSE-PROF-6.1-001, KMIPKIT-CLAUSE-PROF-6.1-002, KMIPKIT-CLAUSE-PROF-6.1-003, KMIPKIT-CLAUSE-PROF-6.1-004, KMIPKIT-CLAUSE-PROF-6.18-001, KMIPKIT-CLAUSE-PROF-6.18-002, KMIPKIT-CLAUSE-PROF-6.18-003, KMIPKIT-CLAUSE-PROF-6.18-004, KMIPKIT-CLAUSE-PROF-6.18-005 | KMIPKIT-REQ-PROF-3.1-001, KMIPKIT-REQ-PROF-5.9.1-001, KMIPKIT-REQ-PROF-5.9.1-002, KMIPKIT-REQ-PROF-5.9.1-003, KMIPKIT-REQ-PROF-5.9.1-004, KMIPKIT-REQ-PROF-6.1-001, KMIPKIT-REQ-PROF-6.1-002, KMIPKIT-REQ-PROF-6.1-003, KMIPKIT-REQ-PROF-6.1-004, KMIPKIT-REQ-PROF-6.18-001, KMIPKIT-REQ-PROF-6.18-002, KMIPKIT-REQ-PROF-6.18-003, KMIPKIT-REQ-PROF-6.18-004, KMIPKIT-REQ-PROF-6.18-005 | KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-QUERY | KMIPKIT-PROFILE-BASELINE-CLIENT | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-1 \(CS-BC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-10 \(CS-BC-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-11 \(CS-BC-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-12 \(CS-BC-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-13 \(CS-BC-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-14 \(CS-BC-M-14-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-15 \(CS-BC-M-GCM-1-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-16 \(CS-BC-M-GCM-2-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-17 \(CS-BC-M-GCM-3-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-18 \(CS-BC-M-CHACHA20-1-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-19 \(CS-BC-M-CHACHA20-2-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-2 \(CS-BC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-20 \(CS-BC-M-CHACHA20-3-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-21 \(CS-BC-M-CHACHA20POLY1305-1-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-3 \(CS-BC-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-4 \(CS-BC-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-5 \(CS-BC-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-6 \(CS-BC-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-7 \(CS-BC-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-8 \(CS-BC-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-9 \(CS-BC-M-9-21; mandatory\) |  |  |
| KMIPKIT-PROFILE-CRYPTOGRAPHIC-BASIC-SERVER | Basic Cryptographic Server | server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.9.4, KMIPKIT-SRC-profiles §5.9.7, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.21 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.9.4-001, KMIPKIT-CLAUSE-PROF-5.9.4-002, KMIPKIT-CLAUSE-PROF-5.9.4-003, KMIPKIT-CLAUSE-PROF-5.9.4-004, KMIPKIT-CLAUSE-PROF-6.21-001, KMIPKIT-CLAUSE-PROF-6.21-002, KMIPKIT-CLAUSE-PROF-6.21-003, KMIPKIT-CLAUSE-PROF-6.21-004, KMIPKIT-CLAUSE-PROF-6.21-005 | KMIPKIT-REQ-PROF-3.1-001 | KMIPKIT-ELEM-OP-C2S-ACTIVATE, KMIPKIT-ELEM-OP-C2S-ADD-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-ADJUST-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-CHECK, KMIPKIT-ELEM-OP-C2S-DECRYPT, KMIPKIT-ELEM-OP-C2S-DELETE-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-DESTROY, KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-C2S-ENCRYPT, KMIPKIT-ELEM-OP-C2S-EXPORT, KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-IMPORT, KMIPKIT-ELEM-OP-C2S-INTEROP, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-LOG, KMIPKIT-ELEM-OP-C2S-MODIFY-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-QUERY, KMIPKIT-ELEM-OP-C2S-REGISTER, KMIPKIT-ELEM-OP-C2S-REVOKE, KMIPKIT-ELEM-OP-C2S-SET-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-SET-ENDPOINT-ROLE, KMIPKIT-ELEM-OP-S2C-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-S2C-NOTIFY, KMIPKIT-ELEM-OP-S2C-PUT, KMIPKIT-ELEM-OP-S2C-QUERY, KMIPKIT-ELEM-OP-S2C-SET-ENDPOINT-ROLE | KMIPKIT-PROFILE-BASELINE-SERVER | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-1 \(CS-BC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-10 \(CS-BC-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-11 \(CS-BC-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-12 \(CS-BC-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-13 \(CS-BC-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-14 \(CS-BC-M-14-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-15 \(CS-BC-M-GCM-1-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-16 \(CS-BC-M-GCM-2-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-17 \(CS-BC-M-GCM-3-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-18 \(CS-BC-M-CHACHA20-1-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-19 \(CS-BC-M-CHACHA20-2-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-2 \(CS-BC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-20 \(CS-BC-M-CHACHA20-3-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-21 \(CS-BC-M-CHACHA20POLY1305-1-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-3 \(CS-BC-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-4 \(CS-BC-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-5 \(CS-BC-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-6 \(CS-BC-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-7 \(CS-BC-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-8 \(CS-BC-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-9-7-9 \(CS-BC-M-9-21; mandatory\) |  |  |
| KMIPKIT-PROFILE-CRYPTOGRAPHIC-RNG-CLIENT | RNG Cryptographic Client | client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.9.3, KMIPKIT-SRC-profiles §5.9.9, KMIPKIT-SRC-profiles §5.9.10, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.20 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.9.3-001, KMIPKIT-CLAUSE-PROF-5.9.3-002, KMIPKIT-CLAUSE-PROF-5.9.3-003, KMIPKIT-CLAUSE-PROF-5.9.3-004, KMIPKIT-CLAUSE-PROF-6.1-001, KMIPKIT-CLAUSE-PROF-6.1-002, KMIPKIT-CLAUSE-PROF-6.1-003, KMIPKIT-CLAUSE-PROF-6.1-004, KMIPKIT-CLAUSE-PROF-6.20-001, KMIPKIT-CLAUSE-PROF-6.20-002, KMIPKIT-CLAUSE-PROF-6.20-003, KMIPKIT-CLAUSE-PROF-6.20-004, KMIPKIT-CLAUSE-PROF-6.20-005 | KMIPKIT-REQ-PROF-3.1-001, KMIPKIT-REQ-PROF-5.9.3-001, KMIPKIT-REQ-PROF-5.9.3-002, KMIPKIT-REQ-PROF-5.9.3-003, KMIPKIT-REQ-PROF-5.9.3-004, KMIPKIT-REQ-PROF-6.1-001, KMIPKIT-REQ-PROF-6.1-002, KMIPKIT-REQ-PROF-6.1-003, KMIPKIT-REQ-PROF-6.1-004, KMIPKIT-REQ-PROF-6.20-001, KMIPKIT-REQ-PROF-6.20-002, KMIPKIT-REQ-PROF-6.20-003, KMIPKIT-REQ-PROF-6.20-004, KMIPKIT-REQ-PROF-6.20-005 | KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-QUERY | KMIPKIT-PROFILE-BASELINE-CLIENT | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-9-10-1 \(CS-RNG-O-1-21; optional\), KMIPKIT-TEST-PROF-5-9-10-2 \(CS-RNG-O-2-21; optional\), KMIPKIT-TEST-PROF-5-9-10-3 \(CS-RNG-O-3-21; optional\), KMIPKIT-TEST-PROF-5-9-10-4 \(CS-RNG-O-4-21; optional\), KMIPKIT-TEST-PROF-5-9-9-1 \(CS-RNG-M-1-21; mandatory\) |  |  |
| KMIPKIT-PROFILE-CRYPTOGRAPHIC-RNG-SERVER | RNG Cryptographic Server | server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.9.6, KMIPKIT-SRC-profiles §5.9.9, KMIPKIT-SRC-profiles §5.9.10, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.23 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.9.6-001, KMIPKIT-CLAUSE-PROF-5.9.6-002, KMIPKIT-CLAUSE-PROF-5.9.6-003, KMIPKIT-CLAUSE-PROF-5.9.6-004, KMIPKIT-CLAUSE-PROF-6.23-001, KMIPKIT-CLAUSE-PROF-6.23-002, KMIPKIT-CLAUSE-PROF-6.23-003, KMIPKIT-CLAUSE-PROF-6.23-004, KMIPKIT-CLAUSE-PROF-6.23-005 | KMIPKIT-REQ-PROF-3.1-001 | KMIPKIT-ELEM-OP-C2S-ACTIVATE, KMIPKIT-ELEM-OP-C2S-ADD-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-ADJUST-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-CHECK, KMIPKIT-ELEM-OP-C2S-DELETE-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-DESTROY, KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-C2S-EXPORT, KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-IMPORT, KMIPKIT-ELEM-OP-C2S-INTEROP, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-LOG, KMIPKIT-ELEM-OP-C2S-MODIFY-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-QUERY, KMIPKIT-ELEM-OP-C2S-REGISTER, KMIPKIT-ELEM-OP-C2S-REVOKE, KMIPKIT-ELEM-OP-C2S-RNG-RETRIEVE, KMIPKIT-ELEM-OP-C2S-RNG-SEED, KMIPKIT-ELEM-OP-C2S-SET-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-SET-ENDPOINT-ROLE, KMIPKIT-ELEM-OP-S2C-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-S2C-NOTIFY, KMIPKIT-ELEM-OP-S2C-PUT, KMIPKIT-ELEM-OP-S2C-QUERY, KMIPKIT-ELEM-OP-S2C-SET-ENDPOINT-ROLE | KMIPKIT-PROFILE-BASELINE-SERVER | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-9-10-1 \(CS-RNG-O-1-21; optional\), KMIPKIT-TEST-PROF-5-9-10-2 \(CS-RNG-O-2-21; optional\), KMIPKIT-TEST-PROF-5-9-10-3 \(CS-RNG-O-3-21; optional\), KMIPKIT-TEST-PROF-5-9-10-4 \(CS-RNG-O-4-21; optional\), KMIPKIT-TEST-PROF-5-9-9-1 \(CS-RNG-M-1-21; mandatory\) |  |  |
| KMIPKIT-PROFILE-HTTPS-CLIENT | HTTPS Client | client | conditional | not\_claimed | KMIPKIT-SRC-profiles §3.2, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.3.1, KMIPKIT-SRC-profiles §5.3.3, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.4 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-3.2-001, KMIPKIT-CLAUSE-PROF-5.3.1-001, KMIPKIT-CLAUSE-PROF-5.3.1-002, KMIPKIT-CLAUSE-PROF-5.3.1-003, KMIPKIT-CLAUSE-PROF-5.3.1-004, KMIPKIT-CLAUSE-PROF-5.3.1-005, KMIPKIT-CLAUSE-PROF-5.3.1-006, KMIPKIT-CLAUSE-PROF-5.3.1-007, KMIPKIT-CLAUSE-PROF-5.3.1-008, KMIPKIT-CLAUSE-PROF-5.3.1-009, KMIPKIT-CLAUSE-PROF-5.3.1-010, KMIPKIT-CLAUSE-PROF-5.3.1-011, KMIPKIT-CLAUSE-PROF-6.1-001, KMIPKIT-CLAUSE-PROF-6.1-002, KMIPKIT-CLAUSE-PROF-6.1-003, KMIPKIT-CLAUSE-PROF-6.1-004, KMIPKIT-CLAUSE-PROF-6.4-001, KMIPKIT-CLAUSE-PROF-6.4-002, KMIPKIT-CLAUSE-PROF-6.4-003, KMIPKIT-CLAUSE-PROF-6.4-004, KMIPKIT-CLAUSE-PROF-6.4-005 | KMIPKIT-REQ-PROF-3.1-001, KMIPKIT-REQ-PROF-3.2-001, KMIPKIT-REQ-PROF-5.3.1-001, KMIPKIT-REQ-PROF-5.3.1-002, KMIPKIT-REQ-PROF-5.3.1-003, KMIPKIT-REQ-PROF-5.3.1-004, KMIPKIT-REQ-PROF-5.3.1-005, KMIPKIT-REQ-PROF-5.3.1-008, KMIPKIT-REQ-PROF-5.3.1-009, KMIPKIT-REQ-PROF-5.3.1-010, KMIPKIT-REQ-PROF-6.1-001, KMIPKIT-REQ-PROF-6.1-002, KMIPKIT-REQ-PROF-6.1-003, KMIPKIT-REQ-PROF-6.1-004, KMIPKIT-REQ-PROF-6.4-001, KMIPKIT-REQ-PROF-6.4-002, KMIPKIT-REQ-PROF-6.4-003, KMIPKIT-REQ-PROF-6.4-004, KMIPKIT-REQ-PROF-6.4-005 | KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-QUERY | KMIPKIT-PROFILE-BASELINE-CLIENT | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-3-3-1 \(MSGENC-HTTPS-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\) | HTTP/1.0, HTTP/1.1, TLS | JSON, TTLV, XML |
| KMIPKIT-PROFILE-HTTPS-SERVER | HTTPS Server | server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.2, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.3.2, KMIPKIT-SRC-profiles §5.3.3, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.5 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-3.2-001, KMIPKIT-CLAUSE-PROF-5.3.2-001, KMIPKIT-CLAUSE-PROF-5.3.2-002, KMIPKIT-CLAUSE-PROF-5.3.2-003, KMIPKIT-CLAUSE-PROF-5.3.2-004, KMIPKIT-CLAUSE-PROF-5.3.2-005, KMIPKIT-CLAUSE-PROF-5.3.2-006, KMIPKIT-CLAUSE-PROF-5.3.2-007, KMIPKIT-CLAUSE-PROF-5.3.2-008, KMIPKIT-CLAUSE-PROF-5.3.2-009, KMIPKIT-CLAUSE-PROF-5.3.2-010, KMIPKIT-CLAUSE-PROF-6.5-001, KMIPKIT-CLAUSE-PROF-6.5-002, KMIPKIT-CLAUSE-PROF-6.5-003, KMIPKIT-CLAUSE-PROF-6.5-004, KMIPKIT-CLAUSE-PROF-6.5-005 | KMIPKIT-REQ-PROF-3.1-001, KMIPKIT-REQ-PROF-3.2-001 | KMIPKIT-ELEM-OP-C2S-ACTIVATE, KMIPKIT-ELEM-OP-C2S-ADD-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-ADJUST-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-CHECK, KMIPKIT-ELEM-OP-C2S-DELETE-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-DESTROY, KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-C2S-EXPORT, KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-IMPORT, KMIPKIT-ELEM-OP-C2S-INTEROP, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-LOG, KMIPKIT-ELEM-OP-C2S-MODIFY-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-QUERY, KMIPKIT-ELEM-OP-C2S-REGISTER, KMIPKIT-ELEM-OP-C2S-REVOKE, KMIPKIT-ELEM-OP-C2S-SET-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-SET-ENDPOINT-ROLE, KMIPKIT-ELEM-OP-S2C-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-S2C-NOTIFY, KMIPKIT-ELEM-OP-S2C-PUT, KMIPKIT-ELEM-OP-S2C-QUERY, KMIPKIT-ELEM-OP-S2C-SET-ENDPOINT-ROLE | KMIPKIT-PROFILE-BASELINE-SERVER | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-3-3-1 \(MSGENC-HTTPS-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\) | HTTP/1.0, HTTP/1.1, TLS | JSON, TTLV, XML |
| KMIPKIT-PROFILE-JSON-CLIENT | JSON Client | client | out\_of\_scope | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.5.2, KMIPKIT-SRC-profiles §5.5.4, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.8 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.5.2-001, KMIPKIT-CLAUSE-PROF-5.5.2-002, KMIPKIT-CLAUSE-PROF-5.5.2-003, KMIPKIT-CLAUSE-PROF-5.5.2-004, KMIPKIT-CLAUSE-PROF-6.1-001, KMIPKIT-CLAUSE-PROF-6.1-002, KMIPKIT-CLAUSE-PROF-6.1-003, KMIPKIT-CLAUSE-PROF-6.1-004, KMIPKIT-CLAUSE-PROF-6.8-001, KMIPKIT-CLAUSE-PROF-6.8-002, KMIPKIT-CLAUSE-PROF-6.8-003, KMIPKIT-CLAUSE-PROF-6.8-004, KMIPKIT-CLAUSE-PROF-6.8-005 | KMIPKIT-REQ-PROF-3.1-001, KMIPKIT-REQ-PROF-6.1-001, KMIPKIT-REQ-PROF-6.1-002, KMIPKIT-REQ-PROF-6.1-003, KMIPKIT-REQ-PROF-6.1-004 | KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-QUERY | KMIPKIT-PROFILE-BASELINE-CLIENT | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-5-4-1 \(MSGENC-JSON-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\) |  | JSON |
| KMIPKIT-PROFILE-JSON-SERVER | JSON Server | server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.5.3, KMIPKIT-SRC-profiles §5.5.4, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.9 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.5.3-001, KMIPKIT-CLAUSE-PROF-5.5.3-002, KMIPKIT-CLAUSE-PROF-5.5.3-003, KMIPKIT-CLAUSE-PROF-5.5.3-004, KMIPKIT-CLAUSE-PROF-6.9-001, KMIPKIT-CLAUSE-PROF-6.9-002, KMIPKIT-CLAUSE-PROF-6.9-003, KMIPKIT-CLAUSE-PROF-6.9-004, KMIPKIT-CLAUSE-PROF-6.9-005, KMIPKIT-CLAUSE-PROF-6.9-006 | KMIPKIT-REQ-PROF-3.1-001 | KMIPKIT-ELEM-OP-C2S-ACTIVATE, KMIPKIT-ELEM-OP-C2S-ADD-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-ADJUST-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-CHECK, KMIPKIT-ELEM-OP-C2S-DELETE-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-DESTROY, KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-C2S-EXPORT, KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-IMPORT, KMIPKIT-ELEM-OP-C2S-INTEROP, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-LOG, KMIPKIT-ELEM-OP-C2S-MODIFY-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-QUERY, KMIPKIT-ELEM-OP-C2S-REGISTER, KMIPKIT-ELEM-OP-C2S-REVOKE, KMIPKIT-ELEM-OP-C2S-SET-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-SET-ENDPOINT-ROLE, KMIPKIT-ELEM-OP-S2C-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-S2C-NOTIFY, KMIPKIT-ELEM-OP-S2C-PUT, KMIPKIT-ELEM-OP-S2C-QUERY, KMIPKIT-ELEM-OP-S2C-SET-ENDPOINT-ROLE | KMIPKIT-PROFILE-BASELINE-SERVER | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-5-4-1 \(MSGENC-JSON-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\) |  | JSON |
| KMIPKIT-PROFILE-OPAQUE-MANAGED-OBJECT-STORE-CLIENT | Opaque Managed Object Store Client | client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.10.1, KMIPKIT-SRC-profiles §5.10.3, KMIPKIT-SRC-profiles §5.10.4, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.24 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.10.1-001, KMIPKIT-CLAUSE-PROF-5.10.1-002, KMIPKIT-CLAUSE-PROF-5.10.1-003, KMIPKIT-CLAUSE-PROF-6.1-001, KMIPKIT-CLAUSE-PROF-6.1-002, KMIPKIT-CLAUSE-PROF-6.1-003, KMIPKIT-CLAUSE-PROF-6.1-004, KMIPKIT-CLAUSE-PROF-6.24-001, KMIPKIT-CLAUSE-PROF-6.24-002, KMIPKIT-CLAUSE-PROF-6.24-003, KMIPKIT-CLAUSE-PROF-6.24-004, KMIPKIT-CLAUSE-PROF-6.24-005 | KMIPKIT-REQ-PROF-3.1-001, KMIPKIT-REQ-PROF-5.10.1-001, KMIPKIT-REQ-PROF-5.10.1-002, KMIPKIT-REQ-PROF-5.10.1-003, KMIPKIT-REQ-PROF-6.1-001, KMIPKIT-REQ-PROF-6.1-002, KMIPKIT-REQ-PROF-6.1-003, KMIPKIT-REQ-PROF-6.1-004, KMIPKIT-REQ-PROF-6.24-001, KMIPKIT-REQ-PROF-6.24-002, KMIPKIT-REQ-PROF-6.24-003, KMIPKIT-REQ-PROF-6.24-004, KMIPKIT-REQ-PROF-6.24-005 | KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-QUERY | KMIPKIT-PROFILE-BASELINE-CLIENT | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-10-3-1 \(OMOS-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-10-4-1 \(OMOS-O-1-21; optional\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\) |  |  |
| KMIPKIT-PROFILE-OPAQUE-MANAGED-OBJECT-STORE-SERVER | Opaque Managed Object Store Server | server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.10.2, KMIPKIT-SRC-profiles §5.10.3, KMIPKIT-SRC-profiles §5.10.4, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.25 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.10.2-001, KMIPKIT-CLAUSE-PROF-5.10.2-002, KMIPKIT-CLAUSE-PROF-5.10.2-003, KMIPKIT-CLAUSE-PROF-5.10.2-004, KMIPKIT-CLAUSE-PROF-5.10.2-005, KMIPKIT-CLAUSE-PROF-5.10.2-006, KMIPKIT-CLAUSE-PROF-5.10.2-007, KMIPKIT-CLAUSE-PROF-6.25-001, KMIPKIT-CLAUSE-PROF-6.25-002, KMIPKIT-CLAUSE-PROF-6.25-003, KMIPKIT-CLAUSE-PROF-6.25-004, KMIPKIT-CLAUSE-PROF-6.25-005 | KMIPKIT-REQ-PROF-3.1-001 | KMIPKIT-ELEM-OP-C2S-ACTIVATE, KMIPKIT-ELEM-OP-C2S-ADD-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-ADJUST-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-CHECK, KMIPKIT-ELEM-OP-C2S-DELETE-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-DESTROY, KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-C2S-EXPORT, KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-IMPORT, KMIPKIT-ELEM-OP-C2S-INTEROP, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-LOG, KMIPKIT-ELEM-OP-C2S-MODIFY-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-QUERY, KMIPKIT-ELEM-OP-C2S-REGISTER, KMIPKIT-ELEM-OP-C2S-REVOKE, KMIPKIT-ELEM-OP-C2S-SET-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-SET-ENDPOINT-ROLE, KMIPKIT-ELEM-OP-S2C-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-S2C-NOTIFY, KMIPKIT-ELEM-OP-S2C-PUT, KMIPKIT-ELEM-OP-S2C-QUERY, KMIPKIT-ELEM-OP-S2C-SET-ENDPOINT-ROLE | KMIPKIT-PROFILE-BASELINE-SERVER | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-10-3-1 \(OMOS-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-10-4-1 \(OMOS-O-1-21; optional\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\) |  |  |
| KMIPKIT-PROFILE-PKCS11-CLIENT | PKCS#11 Client | client | out\_of\_scope | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.18.3, KMIPKIT-SRC-profiles §5.18.5, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.34 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.18.3-001, KMIPKIT-CLAUSE-PROF-5.18.3-002, KMIPKIT-CLAUSE-PROF-5.18.3-003, KMIPKIT-CLAUSE-PROF-5.18.3-004, KMIPKIT-CLAUSE-PROF-5.18.3-005, KMIPKIT-CLAUSE-PROF-6.1-001, KMIPKIT-CLAUSE-PROF-6.1-002, KMIPKIT-CLAUSE-PROF-6.1-003, KMIPKIT-CLAUSE-PROF-6.1-004, KMIPKIT-CLAUSE-PROF-6.34-001, KMIPKIT-CLAUSE-PROF-6.34-002, KMIPKIT-CLAUSE-PROF-6.34-003, KMIPKIT-CLAUSE-PROF-6.34-004, KMIPKIT-CLAUSE-PROF-6.34-005 | KMIPKIT-REQ-PROF-3.1-001, KMIPKIT-REQ-PROF-5.18.3-001, KMIPKIT-REQ-PROF-5.18.3-002, KMIPKIT-REQ-PROF-5.18.3-003, KMIPKIT-REQ-PROF-5.18.3-005, KMIPKIT-REQ-PROF-6.1-001, KMIPKIT-REQ-PROF-6.1-002, KMIPKIT-REQ-PROF-6.1-003, KMIPKIT-REQ-PROF-6.1-004, KMIPKIT-REQ-PROF-6.34-001, KMIPKIT-REQ-PROF-6.34-002, KMIPKIT-REQ-PROF-6.34-003, KMIPKIT-REQ-PROF-6.34-004, KMIPKIT-REQ-PROF-6.34-005 | KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-PKCS-11, KMIPKIT-ELEM-OP-C2S-QUERY | KMIPKIT-PROFILE-BASELINE-CLIENT | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-18-5-1 \(PKCS11-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\) |  | PKCS#11 |
| KMIPKIT-PROFILE-PKCS11-SERVER | PKCS#11 Server | server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.18.4, KMIPKIT-SRC-profiles §5.18.5, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.35 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.18.4-001, KMIPKIT-CLAUSE-PROF-5.18.4-002, KMIPKIT-CLAUSE-PROF-5.18.4-003, KMIPKIT-CLAUSE-PROF-5.18.4-004, KMIPKIT-CLAUSE-PROF-5.18.4-005, KMIPKIT-CLAUSE-PROF-6.35-001, KMIPKIT-CLAUSE-PROF-6.35-002, KMIPKIT-CLAUSE-PROF-6.35-003, KMIPKIT-CLAUSE-PROF-6.35-004, KMIPKIT-CLAUSE-PROF-6.35-005 | KMIPKIT-REQ-PROF-3.1-001 | KMIPKIT-ELEM-OP-C2S-ACTIVATE, KMIPKIT-ELEM-OP-C2S-ADD-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-ADJUST-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-CHECK, KMIPKIT-ELEM-OP-C2S-DELETE-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-DESTROY, KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-C2S-EXPORT, KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-IMPORT, KMIPKIT-ELEM-OP-C2S-INTEROP, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-LOG, KMIPKIT-ELEM-OP-C2S-MODIFY-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-PKCS-11, KMIPKIT-ELEM-OP-C2S-QUERY, KMIPKIT-ELEM-OP-C2S-REGISTER, KMIPKIT-ELEM-OP-C2S-REVOKE, KMIPKIT-ELEM-OP-C2S-SET-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-SET-ENDPOINT-ROLE, KMIPKIT-ELEM-OP-S2C-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-S2C-NOTIFY, KMIPKIT-ELEM-OP-S2C-PUT, KMIPKIT-ELEM-OP-S2C-QUERY, KMIPKIT-ELEM-OP-S2C-SET-ENDPOINT-ROLE | KMIPKIT-PROFILE-BASELINE-SERVER | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-18-5-1 \(PKCS11-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\) |  | PKCS#11 |
| KMIPKIT-PROFILE-QUANTUM-SAFE-CLIENT | Quantum Safe Client | client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.15, KMIPKIT-SRC-profiles §5.17, KMIPKIT-SRC-profiles §5.17.1, KMIPKIT-SRC-profiles §5.17.2, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.32 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.15-001, KMIPKIT-CLAUSE-PROF-5.15-002, KMIPKIT-CLAUSE-PROF-5.15-003, KMIPKIT-CLAUSE-PROF-5.15-004, KMIPKIT-CLAUSE-PROF-5.17-001, KMIPKIT-CLAUSE-PROF-6.1-001, KMIPKIT-CLAUSE-PROF-6.1-002, KMIPKIT-CLAUSE-PROF-6.1-003, KMIPKIT-CLAUSE-PROF-6.1-004, KMIPKIT-CLAUSE-PROF-6.32-001, KMIPKIT-CLAUSE-PROF-6.32-002, KMIPKIT-CLAUSE-PROF-6.32-003, KMIPKIT-CLAUSE-PROF-6.32-004, KMIPKIT-CLAUSE-PROF-6.32-005 | KMIPKIT-REQ-PROF-3.1-001, KMIPKIT-REQ-PROF-5.15-001, KMIPKIT-REQ-PROF-5.15-002, KMIPKIT-REQ-PROF-5.15-003, KMIPKIT-REQ-PROF-5.15-004, KMIPKIT-REQ-PROF-5.17-001, KMIPKIT-REQ-PROF-6.1-001, KMIPKIT-REQ-PROF-6.1-002, KMIPKIT-REQ-PROF-6.1-003, KMIPKIT-REQ-PROF-6.1-004, KMIPKIT-REQ-PROF-6.32-001, KMIPKIT-REQ-PROF-6.32-002, KMIPKIT-REQ-PROF-6.32-003, KMIPKIT-REQ-PROF-6.32-004, KMIPKIT-REQ-PROF-6.32-005 | KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-QUERY | KMIPKIT-PROFILE-BASELINE-CLIENT | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-17-1 \(QS-M-1-12; mandatory\), KMIPKIT-TEST-PROF-5-17-2 \(QS-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\) | TLS 1.3 |  |
| KMIPKIT-PROFILE-QUANTUM-SAFE-SERVER | Quantum Safe Server | server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.16, KMIPKIT-SRC-profiles §5.17.1, KMIPKIT-SRC-profiles §5.17.2, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.33 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.16-001, KMIPKIT-CLAUSE-PROF-5.16-002, KMIPKIT-CLAUSE-PROF-5.16-003, KMIPKIT-CLAUSE-PROF-5.16-004, KMIPKIT-CLAUSE-PROF-5.16-005, KMIPKIT-CLAUSE-PROF-5.16-006, KMIPKIT-CLAUSE-PROF-5.16-007, KMIPKIT-CLAUSE-PROF-5.16-008, KMIPKIT-CLAUSE-PROF-5.16-009, KMIPKIT-CLAUSE-PROF-5.16-010, KMIPKIT-CLAUSE-PROF-6.33-001, KMIPKIT-CLAUSE-PROF-6.33-002, KMIPKIT-CLAUSE-PROF-6.33-003, KMIPKIT-CLAUSE-PROF-6.33-004, KMIPKIT-CLAUSE-PROF-6.33-005 | KMIPKIT-REQ-PROF-3.1-001 | KMIPKIT-ELEM-OP-C2S-ACTIVATE, KMIPKIT-ELEM-OP-C2S-ADD-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-ADJUST-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-CERTIFY, KMIPKIT-ELEM-OP-C2S-CHECK, KMIPKIT-ELEM-OP-C2S-CREATE, KMIPKIT-ELEM-OP-C2S-CREATE-KEY-PAIR, KMIPKIT-ELEM-OP-C2S-DECRYPT, KMIPKIT-ELEM-OP-C2S-DELETE-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-DESTROY, KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-C2S-ENCRYPT, KMIPKIT-ELEM-OP-C2S-EXPORT, KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-IMPORT, KMIPKIT-ELEM-OP-C2S-INTEROP, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-LOG, KMIPKIT-ELEM-OP-C2S-MODIFY-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-QUERY, KMIPKIT-ELEM-OP-C2S-RE-CERTIFY, KMIPKIT-ELEM-OP-C2S-RE-KEY, KMIPKIT-ELEM-OP-C2S-RE-KEY-KEY-PAIR, KMIPKIT-ELEM-OP-C2S-REGISTER, KMIPKIT-ELEM-OP-C2S-REVOKE, KMIPKIT-ELEM-OP-C2S-SET-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-SET-ENDPOINT-ROLE, KMIPKIT-ELEM-OP-C2S-SIGN, KMIPKIT-ELEM-OP-C2S-SIGNATURE-VERIFY, KMIPKIT-ELEM-OP-S2C-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-S2C-NOTIFY, KMIPKIT-ELEM-OP-S2C-PUT, KMIPKIT-ELEM-OP-S2C-QUERY, KMIPKIT-ELEM-OP-S2C-SET-ENDPOINT-ROLE | KMIPKIT-PROFILE-BASELINE-SERVER | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-17-1 \(QS-M-1-12; mandatory\), KMIPKIT-TEST-PROF-5-17-2 \(QS-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\) | TLS 1.3 |  |
| KMIPKIT-PROFILE-STORAGE-ARRAY-SELF-ENCRYPTING-DRIVES-CLIENT | Storage Array with Self-Encrypting Drives Client | client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.11.1, KMIPKIT-SRC-profiles §5.11.3, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.26 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.11.1-001, KMIPKIT-CLAUSE-PROF-5.11.1-002, KMIPKIT-CLAUSE-PROF-5.11.1-003, KMIPKIT-CLAUSE-PROF-5.11.1-004, KMIPKIT-CLAUSE-PROF-6.1-001, KMIPKIT-CLAUSE-PROF-6.1-002, KMIPKIT-CLAUSE-PROF-6.1-003, KMIPKIT-CLAUSE-PROF-6.1-004, KMIPKIT-CLAUSE-PROF-6.26-001, KMIPKIT-CLAUSE-PROF-6.26-002, KMIPKIT-CLAUSE-PROF-6.26-003, KMIPKIT-CLAUSE-PROF-6.26-004, KMIPKIT-CLAUSE-PROF-6.26-005 | KMIPKIT-REQ-PROF-3.1-001, KMIPKIT-REQ-PROF-5.11.1-001, KMIPKIT-REQ-PROF-5.11.1-002, KMIPKIT-REQ-PROF-5.11.1-003, KMIPKIT-REQ-PROF-5.11.1-004, KMIPKIT-REQ-PROF-6.1-001, KMIPKIT-REQ-PROF-6.1-002, KMIPKIT-REQ-PROF-6.1-003, KMIPKIT-REQ-PROF-6.1-004, KMIPKIT-REQ-PROF-6.26-001, KMIPKIT-REQ-PROF-6.26-002, KMIPKIT-REQ-PROF-6.26-003, KMIPKIT-REQ-PROF-6.26-004, KMIPKIT-REQ-PROF-6.26-005 | KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-QUERY | KMIPKIT-PROFILE-BASELINE-CLIENT | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-11-3-1 \(SASED-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-11-3-2 \(SASED-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-11-3-3 \(SASED-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\) |  |  |
| KMIPKIT-PROFILE-STORAGE-ARRAY-SELF-ENCRYPTING-DRIVES-SERVER | Storage Array with Self-Encrypting Drives Server | server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.11.2, KMIPKIT-SRC-profiles §5.11.3, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.27 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.11.2-001, KMIPKIT-CLAUSE-PROF-5.11.2-002, KMIPKIT-CLAUSE-PROF-5.11.2-003, KMIPKIT-CLAUSE-PROF-5.11.2-004, KMIPKIT-CLAUSE-PROF-5.11.2-005, KMIPKIT-CLAUSE-PROF-5.11.2-006, KMIPKIT-CLAUSE-PROF-5.11.2-007, KMIPKIT-CLAUSE-PROF-5.11.2-008, KMIPKIT-CLAUSE-PROF-5.11.2-009, KMIPKIT-CLAUSE-PROF-5.11.2-010, KMIPKIT-CLAUSE-PROF-5.11.2-011, KMIPKIT-CLAUSE-PROF-6.27-001, KMIPKIT-CLAUSE-PROF-6.27-002, KMIPKIT-CLAUSE-PROF-6.27-003, KMIPKIT-CLAUSE-PROF-6.27-004, KMIPKIT-CLAUSE-PROF-6.27-005 | KMIPKIT-REQ-PROF-3.1-001 | KMIPKIT-ELEM-OP-C2S-ACTIVATE, KMIPKIT-ELEM-OP-C2S-ADD-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-ADJUST-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-CHECK, KMIPKIT-ELEM-OP-C2S-DELETE-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-DESTROY, KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-C2S-EXPORT, KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-IMPORT, KMIPKIT-ELEM-OP-C2S-INTEROP, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-LOG, KMIPKIT-ELEM-OP-C2S-MODIFY-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-QUERY, KMIPKIT-ELEM-OP-C2S-REGISTER, KMIPKIT-ELEM-OP-C2S-REVOKE, KMIPKIT-ELEM-OP-C2S-SET-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-SET-ENDPOINT-ROLE, KMIPKIT-ELEM-OP-S2C-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-S2C-NOTIFY, KMIPKIT-ELEM-OP-S2C-PUT, KMIPKIT-ELEM-OP-S2C-QUERY, KMIPKIT-ELEM-OP-S2C-SET-ENDPOINT-ROLE | KMIPKIT-PROFILE-BASELINE-SERVER | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-11-3-1 \(SASED-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-11-3-2 \(SASED-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-11-3-3 \(SASED-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\) |  |  |
| KMIPKIT-PROFILE-SYMMETRIC-KEY-FOUNDRY-ADVANCED-CLIENT | Advanced Symmetric Key Foundry Client | client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.7.3, KMIPKIT-SRC-profiles §5.7.7, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.14 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.7.3-001, KMIPKIT-CLAUSE-PROF-5.7.3-002, KMIPKIT-CLAUSE-PROF-5.7.3-003, KMIPKIT-CLAUSE-PROF-6.1-001, KMIPKIT-CLAUSE-PROF-6.1-002, KMIPKIT-CLAUSE-PROF-6.1-003, KMIPKIT-CLAUSE-PROF-6.1-004, KMIPKIT-CLAUSE-PROF-6.14-001, KMIPKIT-CLAUSE-PROF-6.14-002, KMIPKIT-CLAUSE-PROF-6.14-003, KMIPKIT-CLAUSE-PROF-6.14-004, KMIPKIT-CLAUSE-PROF-6.14-005 | KMIPKIT-REQ-PROF-3.1-001, KMIPKIT-REQ-PROF-5.7.3-001, KMIPKIT-REQ-PROF-5.7.3-002, KMIPKIT-REQ-PROF-5.7.3-003, KMIPKIT-REQ-PROF-6.1-001, KMIPKIT-REQ-PROF-6.1-002, KMIPKIT-REQ-PROF-6.1-003, KMIPKIT-REQ-PROF-6.1-004, KMIPKIT-REQ-PROF-6.14-001, KMIPKIT-REQ-PROF-6.14-002, KMIPKIT-REQ-PROF-6.14-004, KMIPKIT-REQ-PROF-6.14-005 | KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-QUERY | KMIPKIT-PROFILE-BASELINE-CLIENT | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-7-7-1 \(SKFF-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-7-7-2 \(SKFF-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-7-7-3 \(SKFF-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-7-7-4 \(SKFF-M-12-21; mandatory\) |  |  |
| KMIPKIT-PROFILE-SYMMETRIC-KEY-FOUNDRY-BASIC-CLIENT | Basic Symmetric Key Foundry Client | client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.7.1, KMIPKIT-SRC-profiles §5.7.5, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.12 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.7.1-001, KMIPKIT-CLAUSE-PROF-5.7.1-002, KMIPKIT-CLAUSE-PROF-5.7.1-003, KMIPKIT-CLAUSE-PROF-6.1-001, KMIPKIT-CLAUSE-PROF-6.1-002, KMIPKIT-CLAUSE-PROF-6.1-003, KMIPKIT-CLAUSE-PROF-6.1-004, KMIPKIT-CLAUSE-PROF-6.12-001, KMIPKIT-CLAUSE-PROF-6.12-002, KMIPKIT-CLAUSE-PROF-6.12-003, KMIPKIT-CLAUSE-PROF-6.12-004, KMIPKIT-CLAUSE-PROF-6.12-005 | KMIPKIT-REQ-PROF-3.1-001, KMIPKIT-REQ-PROF-5.7.1-001, KMIPKIT-REQ-PROF-5.7.1-002, KMIPKIT-REQ-PROF-5.7.1-003, KMIPKIT-REQ-PROF-6.1-001, KMIPKIT-REQ-PROF-6.1-002, KMIPKIT-REQ-PROF-6.1-003, KMIPKIT-REQ-PROF-6.1-004, KMIPKIT-REQ-PROF-6.12-001, KMIPKIT-REQ-PROF-6.12-002, KMIPKIT-REQ-PROF-6.12-003, KMIPKIT-REQ-PROF-6.12-004, KMIPKIT-REQ-PROF-6.12-005 | KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-QUERY | KMIPKIT-PROFILE-BASELINE-CLIENT | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-7-5-1 \(SKFF-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-7-5-2 \(SKFF-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-7-5-3 \(SKFF-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-7-5-4 \(SKFF-M-4-21; mandatory\) |  |  |
| KMIPKIT-PROFILE-SYMMETRIC-KEY-FOUNDRY-INTERMEDIATE-CLIENT | Intermediate Symmetric Key Foundry Client | client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.7.2, KMIPKIT-SRC-profiles §5.7.6, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.13 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.7.2-001, KMIPKIT-CLAUSE-PROF-5.7.2-002, KMIPKIT-CLAUSE-PROF-5.7.2-003, KMIPKIT-CLAUSE-PROF-6.1-001, KMIPKIT-CLAUSE-PROF-6.1-002, KMIPKIT-CLAUSE-PROF-6.1-003, KMIPKIT-CLAUSE-PROF-6.1-004, KMIPKIT-CLAUSE-PROF-6.13-001, KMIPKIT-CLAUSE-PROF-6.13-002, KMIPKIT-CLAUSE-PROF-6.13-003, KMIPKIT-CLAUSE-PROF-6.13-004, KMIPKIT-CLAUSE-PROF-6.13-005 | KMIPKIT-REQ-PROF-3.1-001, KMIPKIT-REQ-PROF-5.7.2-001, KMIPKIT-REQ-PROF-5.7.2-002, KMIPKIT-REQ-PROF-5.7.2-003, KMIPKIT-REQ-PROF-6.1-001, KMIPKIT-REQ-PROF-6.1-002, KMIPKIT-REQ-PROF-6.1-003, KMIPKIT-REQ-PROF-6.1-004, KMIPKIT-REQ-PROF-6.13-001, KMIPKIT-REQ-PROF-6.13-002, KMIPKIT-REQ-PROF-6.13-004, KMIPKIT-REQ-PROF-6.13-005 | KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-QUERY | KMIPKIT-PROFILE-BASELINE-CLIENT | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-7-6-1 \(SKFF-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-7-6-2 \(SKFF-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-7-6-3 \(SKFF-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-7-6-4 \(SKFF-M-8-21; mandatory\) |  |  |
| KMIPKIT-PROFILE-SYMMETRIC-KEY-FOUNDRY-SERVER | Symmetric Key Foundry Server | server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.7.4, KMIPKIT-SRC-profiles §5.7.5, KMIPKIT-SRC-profiles §5.7.6, KMIPKIT-SRC-profiles §5.7.7, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.15 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.7.4-001, KMIPKIT-CLAUSE-PROF-5.7.4-002, KMIPKIT-CLAUSE-PROF-5.7.4-003, KMIPKIT-CLAUSE-PROF-5.7.4-004, KMIPKIT-CLAUSE-PROF-5.7.4-005, KMIPKIT-CLAUSE-PROF-5.7.4-006, KMIPKIT-CLAUSE-PROF-5.7.4-007, KMIPKIT-CLAUSE-PROF-6.15-001, KMIPKIT-CLAUSE-PROF-6.15-002, KMIPKIT-CLAUSE-PROF-6.15-003, KMIPKIT-CLAUSE-PROF-6.15-004, KMIPKIT-CLAUSE-PROF-6.15-005, KMIPKIT-CLAUSE-PROF-6.15-006, KMIPKIT-CLAUSE-PROF-6.15-007 | KMIPKIT-REQ-PROF-3.1-001 | KMIPKIT-ELEM-OP-C2S-ACTIVATE, KMIPKIT-ELEM-OP-C2S-ADD-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-ADJUST-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-CHECK, KMIPKIT-ELEM-OP-C2S-CREATE, KMIPKIT-ELEM-OP-C2S-DELETE-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-DESTROY, KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-C2S-EXPORT, KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-IMPORT, KMIPKIT-ELEM-OP-C2S-INTEROP, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-LOG, KMIPKIT-ELEM-OP-C2S-MODIFY-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-QUERY, KMIPKIT-ELEM-OP-C2S-REGISTER, KMIPKIT-ELEM-OP-C2S-REVOKE, KMIPKIT-ELEM-OP-C2S-SET-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-SET-ENDPOINT-ROLE, KMIPKIT-ELEM-OP-S2C-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-S2C-NOTIFY, KMIPKIT-ELEM-OP-S2C-PUT, KMIPKIT-ELEM-OP-S2C-QUERY, KMIPKIT-ELEM-OP-S2C-SET-ENDPOINT-ROLE | KMIPKIT-PROFILE-BASELINE-SERVER | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-7-5-1 \(SKFF-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-7-5-2 \(SKFF-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-7-5-3 \(SKFF-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-7-5-4 \(SKFF-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-7-6-1 \(SKFF-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-7-6-2 \(SKFF-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-7-6-3 \(SKFF-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-7-6-4 \(SKFF-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-7-7-1 \(SKFF-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-7-7-2 \(SKFF-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-7-7-3 \(SKFF-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-7-7-4 \(SKFF-M-12-21; mandatory\) |  |  |
| KMIPKIT-PROFILE-SYMMETRIC-KEY-LIFECYCLE-CLIENT | Symmetric Key Lifecycle Client | client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.6.1, KMIPKIT-SRC-profiles §5.6.3, KMIPKIT-SRC-profiles §5.6.4, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.10 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.6.1-001, KMIPKIT-CLAUSE-PROF-5.6.1-002, KMIPKIT-CLAUSE-PROF-5.6.1-003, KMIPKIT-CLAUSE-PROF-6.1-001, KMIPKIT-CLAUSE-PROF-6.1-002, KMIPKIT-CLAUSE-PROF-6.1-003, KMIPKIT-CLAUSE-PROF-6.1-004, KMIPKIT-CLAUSE-PROF-6.10-001, KMIPKIT-CLAUSE-PROF-6.10-002, KMIPKIT-CLAUSE-PROF-6.10-003, KMIPKIT-CLAUSE-PROF-6.10-004, KMIPKIT-CLAUSE-PROF-6.10-005 | KMIPKIT-REQ-PROF-3.1-001, KMIPKIT-REQ-PROF-5.6.1-001, KMIPKIT-REQ-PROF-5.6.1-002, KMIPKIT-REQ-PROF-5.6.1-003, KMIPKIT-REQ-PROF-6.1-001, KMIPKIT-REQ-PROF-6.1-002, KMIPKIT-REQ-PROF-6.1-003, KMIPKIT-REQ-PROF-6.1-004, KMIPKIT-REQ-PROF-6.10-001, KMIPKIT-REQ-PROF-6.10-002, KMIPKIT-REQ-PROF-6.10-003, KMIPKIT-REQ-PROF-6.10-004, KMIPKIT-REQ-PROF-6.10-005 | KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-QUERY | KMIPKIT-PROFILE-BASELINE-CLIENT | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-6-4-1 \(SKLC-O-1-21; optional\) |  |  |
| KMIPKIT-PROFILE-SYMMETRIC-KEY-LIFECYCLE-SERVER | Symmetric Key Lifecycle Server | server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.6.2, KMIPKIT-SRC-profiles §5.6.3, KMIPKIT-SRC-profiles §5.6.4, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.11 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.6.2-001, KMIPKIT-CLAUSE-PROF-5.6.2-002, KMIPKIT-CLAUSE-PROF-5.6.2-003, KMIPKIT-CLAUSE-PROF-5.6.2-004, KMIPKIT-CLAUSE-PROF-5.6.2-005, KMIPKIT-CLAUSE-PROF-5.6.2-006, KMIPKIT-CLAUSE-PROF-5.6.2-007, KMIPKIT-CLAUSE-PROF-6.11-001, KMIPKIT-CLAUSE-PROF-6.11-002, KMIPKIT-CLAUSE-PROF-6.11-003, KMIPKIT-CLAUSE-PROF-6.11-004, KMIPKIT-CLAUSE-PROF-6.11-005 | KMIPKIT-REQ-PROF-3.1-001 | KMIPKIT-ELEM-OP-C2S-ACTIVATE, KMIPKIT-ELEM-OP-C2S-ADD-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-ADJUST-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-CHECK, KMIPKIT-ELEM-OP-C2S-CREATE, KMIPKIT-ELEM-OP-C2S-DELETE-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-DESTROY, KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-C2S-EXPORT, KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-IMPORT, KMIPKIT-ELEM-OP-C2S-INTEROP, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-LOG, KMIPKIT-ELEM-OP-C2S-MODIFY-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-QUERY, KMIPKIT-ELEM-OP-C2S-REGISTER, KMIPKIT-ELEM-OP-C2S-REVOKE, KMIPKIT-ELEM-OP-C2S-SET-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-SET-ENDPOINT-ROLE, KMIPKIT-ELEM-OP-S2C-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-S2C-NOTIFY, KMIPKIT-ELEM-OP-S2C-PUT, KMIPKIT-ELEM-OP-S2C-QUERY, KMIPKIT-ELEM-OP-S2C-SET-ENDPOINT-ROLE | KMIPKIT-PROFILE-BASELINE-SERVER | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-6-4-1 \(SKLC-O-1-21; optional\) |  |  |
| KMIPKIT-PROFILE-TAPE-LIBRARY-CLIENT | Tape Library Client | client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.12.2, KMIPKIT-SRC-profiles §5.12.3, KMIPKIT-SRC-profiles §5.12.4, KMIPKIT-SRC-profiles §5.12.6, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.28 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.12.2-002, KMIPKIT-CLAUSE-PROF-5.12.2-003, KMIPKIT-CLAUSE-PROF-5.12.2-004, KMIPKIT-CLAUSE-PROF-5.12.2-005, KMIPKIT-CLAUSE-PROF-5.12.2-007, KMIPKIT-CLAUSE-PROF-5.12.3-001, KMIPKIT-CLAUSE-PROF-5.12.3-003, KMIPKIT-CLAUSE-PROF-5.12.4-001, KMIPKIT-CLAUSE-PROF-5.12.4-002, KMIPKIT-CLAUSE-PROF-5.12.4-003, KMIPKIT-CLAUSE-PROF-5.12.4-004, KMIPKIT-CLAUSE-PROF-5.12.4-005, KMIPKIT-CLAUSE-PROF-5.12.4-006, KMIPKIT-CLAUSE-PROF-5.12.4-007, KMIPKIT-CLAUSE-PROF-5.12.4-008, KMIPKIT-CLAUSE-PROF-5.12.4-009, KMIPKIT-CLAUSE-PROF-6.1-001, KMIPKIT-CLAUSE-PROF-6.1-002, KMIPKIT-CLAUSE-PROF-6.1-003, KMIPKIT-CLAUSE-PROF-6.1-004, KMIPKIT-CLAUSE-PROF-6.28-001, KMIPKIT-CLAUSE-PROF-6.28-002, KMIPKIT-CLAUSE-PROF-6.28-003, KMIPKIT-CLAUSE-PROF-6.28-004, KMIPKIT-CLAUSE-PROF-6.28-005, KMIPKIT-CLAUSE-PROF-6.28-006, KMIPKIT-CLAUSE-PROF-6.28-007 | KMIPKIT-REQ-PROF-3.1-001, KMIPKIT-REQ-PROF-5.12.2-002, KMIPKIT-REQ-PROF-5.12.2-003, KMIPKIT-REQ-PROF-5.12.2-004-001, KMIPKIT-REQ-PROF-5.12.2-004-002, KMIPKIT-REQ-PROF-5.12.2-005, KMIPKIT-REQ-PROF-5.12.2-007-001, KMIPKIT-REQ-PROF-5.12.2-007-002, KMIPKIT-REQ-PROF-5.12.3-001, KMIPKIT-REQ-PROF-5.12.3-003, KMIPKIT-REQ-PROF-5.12.4-001, KMIPKIT-REQ-PROF-5.12.4-002, KMIPKIT-REQ-PROF-5.12.4-003, KMIPKIT-REQ-PROF-5.12.4-004, KMIPKIT-REQ-PROF-5.12.4-005, KMIPKIT-REQ-PROF-5.12.4-006, KMIPKIT-REQ-PROF-5.12.4-007, KMIPKIT-REQ-PROF-5.12.4-008, KMIPKIT-REQ-PROF-5.12.4-009, KMIPKIT-REQ-PROF-6.1-001, KMIPKIT-REQ-PROF-6.1-002, KMIPKIT-REQ-PROF-6.1-003, KMIPKIT-REQ-PROF-6.1-004, KMIPKIT-REQ-PROF-6.28-001, KMIPKIT-REQ-PROF-6.28-002, KMIPKIT-REQ-PROF-6.28-003, KMIPKIT-REQ-PROF-6.28-004, KMIPKIT-REQ-PROF-6.28-005, KMIPKIT-REQ-PROF-6.28-006, KMIPKIT-REQ-PROF-6.28-007 | KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-QUERY | KMIPKIT-PROFILE-BASELINE-CLIENT | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-12-6-1 \(TL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-12-6-2 \(TL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-12-6-3 \(TL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\) |  |  |
| KMIPKIT-PROFILE-TAPE-LIBRARY-SERVER | Tape Library Server | server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.12.5, KMIPKIT-SRC-profiles §5.12.6, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.29 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.12.5-001, KMIPKIT-CLAUSE-PROF-5.12.5-002, KMIPKIT-CLAUSE-PROF-5.12.5-003, KMIPKIT-CLAUSE-PROF-5.12.5-004, KMIPKIT-CLAUSE-PROF-5.12.5-005, KMIPKIT-CLAUSE-PROF-5.12.5-006, KMIPKIT-CLAUSE-PROF-5.12.5-007, KMIPKIT-CLAUSE-PROF-5.12.5-008, KMIPKIT-CLAUSE-PROF-5.12.5-009, KMIPKIT-CLAUSE-PROF-5.12.5-010, KMIPKIT-CLAUSE-PROF-5.12.5-011, KMIPKIT-CLAUSE-PROF-5.12.5-012, KMIPKIT-CLAUSE-PROF-6.29-001, KMIPKIT-CLAUSE-PROF-6.29-002, KMIPKIT-CLAUSE-PROF-6.29-003, KMIPKIT-CLAUSE-PROF-6.29-004, KMIPKIT-CLAUSE-PROF-6.29-005, KMIPKIT-CLAUSE-PROF-6.29-006, KMIPKIT-CLAUSE-PROF-6.29-007 | KMIPKIT-REQ-PROF-3.1-001 | KMIPKIT-ELEM-OP-C2S-ACTIVATE, KMIPKIT-ELEM-OP-C2S-ADD-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-ADJUST-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-CHECK, KMIPKIT-ELEM-OP-C2S-CREATE, KMIPKIT-ELEM-OP-C2S-DELETE-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-DESTROY, KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-C2S-EXPORT, KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-IMPORT, KMIPKIT-ELEM-OP-C2S-INTEROP, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-LOG, KMIPKIT-ELEM-OP-C2S-MODIFY-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-QUERY, KMIPKIT-ELEM-OP-C2S-REGISTER, KMIPKIT-ELEM-OP-C2S-REVOKE, KMIPKIT-ELEM-OP-C2S-SET-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-SET-ENDPOINT-ROLE, KMIPKIT-ELEM-OP-S2C-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-S2C-NOTIFY, KMIPKIT-ELEM-OP-S2C-PUT, KMIPKIT-ELEM-OP-S2C-QUERY, KMIPKIT-ELEM-OP-S2C-SET-ENDPOINT-ROLE | KMIPKIT-PROFILE-BASELINE-SERVER | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-12-6-1 \(TL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-12-6-2 \(TL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-12-6-3 \(TL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\) |  |  |
| KMIPKIT-PROFILE-XML-CLIENT | XML Client | client | out\_of\_scope | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.4.2, KMIPKIT-SRC-profiles §5.4.4, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.6 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.4.2-001, KMIPKIT-CLAUSE-PROF-5.4.2-002, KMIPKIT-CLAUSE-PROF-5.4.2-003, KMIPKIT-CLAUSE-PROF-5.4.2-004, KMIPKIT-CLAUSE-PROF-6.1-001, KMIPKIT-CLAUSE-PROF-6.1-002, KMIPKIT-CLAUSE-PROF-6.1-003, KMIPKIT-CLAUSE-PROF-6.1-004, KMIPKIT-CLAUSE-PROF-6.6-001, KMIPKIT-CLAUSE-PROF-6.6-002, KMIPKIT-CLAUSE-PROF-6.6-003, KMIPKIT-CLAUSE-PROF-6.6-004, KMIPKIT-CLAUSE-PROF-6.6-005 | KMIPKIT-REQ-PROF-3.1-001, KMIPKIT-REQ-PROF-6.1-001, KMIPKIT-REQ-PROF-6.1-002, KMIPKIT-REQ-PROF-6.1-003, KMIPKIT-REQ-PROF-6.1-004 | KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-QUERY | KMIPKIT-PROFILE-BASELINE-CLIENT | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-4-4-1 \(MSGENC-XML-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\) |  | XML |
| KMIPKIT-PROFILE-XML-SERVER | XML Server | server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.4.3, KMIPKIT-SRC-profiles §5.4.4, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.7 | KMIPKIT-CLAUSE-PROF-3.1-001, KMIPKIT-CLAUSE-PROF-5.4.3-001, KMIPKIT-CLAUSE-PROF-5.4.3-002, KMIPKIT-CLAUSE-PROF-5.4.3-003, KMIPKIT-CLAUSE-PROF-5.4.3-004, KMIPKIT-CLAUSE-PROF-6.7-001, KMIPKIT-CLAUSE-PROF-6.7-002, KMIPKIT-CLAUSE-PROF-6.7-003, KMIPKIT-CLAUSE-PROF-6.7-004, KMIPKIT-CLAUSE-PROF-6.7-005, KMIPKIT-CLAUSE-PROF-6.7-006 | KMIPKIT-REQ-PROF-3.1-001 | KMIPKIT-ELEM-OP-C2S-ACTIVATE, KMIPKIT-ELEM-OP-C2S-ADD-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-ADJUST-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-CHECK, KMIPKIT-ELEM-OP-C2S-DELETE-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-DESTROY, KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-C2S-EXPORT, KMIPKIT-ELEM-OP-C2S-GET, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST, KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES, KMIPKIT-ELEM-OP-C2S-IMPORT, KMIPKIT-ELEM-OP-C2S-INTEROP, KMIPKIT-ELEM-OP-C2S-LOCATE, KMIPKIT-ELEM-OP-C2S-LOG, KMIPKIT-ELEM-OP-C2S-MODIFY-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-QUERY, KMIPKIT-ELEM-OP-C2S-REGISTER, KMIPKIT-ELEM-OP-C2S-REVOKE, KMIPKIT-ELEM-OP-C2S-SET-ATTRIBUTE, KMIPKIT-ELEM-OP-C2S-SET-ENDPOINT-ROLE, KMIPKIT-ELEM-OP-S2C-DISCOVER-VERSIONS, KMIPKIT-ELEM-OP-S2C-NOTIFY, KMIPKIT-ELEM-OP-S2C-PUT, KMIPKIT-ELEM-OP-S2C-QUERY, KMIPKIT-ELEM-OP-S2C-SET-ENDPOINT-ROLE | KMIPKIT-PROFILE-BASELINE-SERVER | KMIPKIT-TEST-PROF-5-1-3-1 \(BL-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-10 \(BL-M-10-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-11 \(BL-M-11-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-12 \(BL-M-12-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-13 \(BL-M-13-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-2 \(BL-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-3 \(BL-M-3-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-4 \(BL-M-4-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-5 \(BL-M-5-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-6 \(BL-M-6-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-7 \(BL-M-7-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-8 \(BL-M-8-21; mandatory\), KMIPKIT-TEST-PROF-5-1-3-9 \(BL-M-9-21; mandatory\), KMIPKIT-TEST-PROF-5-4-4-1 \(MSGENC-XML-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-1 \(SKLC-M-1-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-2 \(SKLC-M-2-21; mandatory\), KMIPKIT-TEST-PROF-5-6-3-3 \(SKLC-M-3-21; mandatory\) |  | XML |

### Profiles by applicability and claim state

| Dimension | Value | Count |
| --- | --- | --- |
| Applicability | client\_1\_0 | 14 |
| Applicability | conditional | 1 |
| Applicability | out\_of\_scope | 3 |
| Applicability | server\_only | 17 |
| Claim state | not\_claimed | 35 |

## Test fixture availability

| Test | Official ID | Fixture status | Local fixture |
| --- | --- | --- | --- |
| KMIPKIT-TEST-PROF-5-8-3-1 | AKLC-M-1-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-8-3-2 | AKLC-M-2-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-8-3-3 | AKLC-M-3-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-8-4-1 | AKLC-O-1-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-13-3-1 | AX-M-1-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-13-3-2 | AX-M-2-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-1-3-1 | BL-M-1-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-1-3-10 | BL-M-10-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-1-3-11 | BL-M-11-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-1-3-12 | BL-M-12-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-1-3-13 | BL-M-13-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-1-3-2 | BL-M-2-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-1-3-3 | BL-M-3-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-1-3-4 | BL-M-4-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-1-3-5 | BL-M-5-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-1-3-6 | BL-M-6-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-1-3-7 | BL-M-7-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-1-3-8 | BL-M-8-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-1-3-9 | BL-M-9-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-8-1 | CS-AC-M-1-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-8-2 | CS-AC-M-2-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-8-3 | CS-AC-M-3-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-8-4 | CS-AC-M-4-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-8-5 | CS-AC-M-5-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-8-6 | CS-AC-M-6-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-8-7 | CS-AC-M-7-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-8-8 | CS-AC-M-8-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-8-9 | CS-AC-M-OAEP-1-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-8-18 | CS-AC-M-OAEP-10-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-8-10 | CS-AC-M-OAEP-2-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-8-11 | CS-AC-M-OAEP-3-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-8-12 | CS-AC-M-OAEP-4-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-8-13 | CS-AC-M-OAEP-5-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-8-14 | CS-AC-M-OAEP-6-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-8-15 | CS-AC-M-OAEP-7-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-8-16 | CS-AC-M-OAEP-8-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-8-17 | CS-AC-M-OAEP-9-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-7-1 | CS-BC-M-1-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-7-10 | CS-BC-M-10-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-7-11 | CS-BC-M-11-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-7-12 | CS-BC-M-12-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-7-13 | CS-BC-M-13-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-7-14 | CS-BC-M-14-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-7-2 | CS-BC-M-2-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-7-3 | CS-BC-M-3-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-7-4 | CS-BC-M-4-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-7-5 | CS-BC-M-5-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-7-6 | CS-BC-M-6-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-7-7 | CS-BC-M-7-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-7-8 | CS-BC-M-8-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-7-9 | CS-BC-M-9-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-7-18 | CS-BC-M-CHACHA20-1-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-7-19 | CS-BC-M-CHACHA20-2-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-7-20 | CS-BC-M-CHACHA20-3-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-7-21 | CS-BC-M-CHACHA20POLY1305-1-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-7-15 | CS-BC-M-GCM-1-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-7-16 | CS-BC-M-GCM-2-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-7-17 | CS-BC-M-GCM-3-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-9-1 | CS-RNG-M-1-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-10-1 | CS-RNG-O-1-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-10-2 | CS-RNG-O-2-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-10-3 | CS-RNG-O-3-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-9-10-4 | CS-RNG-O-4-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-3-3-1 | MSGENC-HTTPS-M-1-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-5-4-1 | MSGENC-JSON-M-1-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-4-4-1 | MSGENC-XML-M-1-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-10-3-1 | OMOS-M-1-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-10-4-1 | OMOS-O-1-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-18-5-1 | PKCS11-M-1-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-17-1 | QS-M-1-12 | unavailable | — |
| KMIPKIT-TEST-PROF-5-17-2 | QS-M-2-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-11-3-1 | SASED-M-1-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-11-3-2 | SASED-M-2-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-11-3-3 | SASED-M-3-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-7-5-1 | SKFF-M-1-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-7-7-2 | SKFF-M-10-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-7-7-3 | SKFF-M-11-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-7-7-4 | SKFF-M-12-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-7-5-2 | SKFF-M-2-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-7-5-3 | SKFF-M-3-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-7-5-4 | SKFF-M-4-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-7-6-1 | SKFF-M-5-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-7-6-2 | SKFF-M-6-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-7-6-3 | SKFF-M-7-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-7-6-4 | SKFF-M-8-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-7-7-1 | SKFF-M-9-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-6-3-1 | SKLC-M-1-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-6-3-2 | SKLC-M-2-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-6-3-3 | SKLC-M-3-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-6-4-1 | SKLC-O-1-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-12-6-1 | TL-M-1-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-12-6-2 | TL-M-2-21 | unavailable | — |
| KMIPKIT-TEST-PROF-5-12-6-3 | TL-M-3-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-1 | TC-ASYNC-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-10 | TC-ASYNC-10-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-2 | TC-ASYNC-2-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-3 | TC-ASYNC-3-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-4 | TC-ASYNC-4-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-5 | TC-ASYNC-5-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-6 | TC-ASYNC-6-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-7 | TC-ASYNC-7-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-8 | TC-ASYNC-8-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-9 | TC-ASYNC-9-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-11 | TC-CERTATTR-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-12 | TC-CREATE-SD-1-21 | available | specification/oasis/kmip-2.1/fixtures/TC-CREATE-SD-1-21.xml |
| KMIPKIT-TEST-CN01-2-13 | TC-CS-CORVAL-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-14 | TC-DERIVEKEY-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-15 | TC-DERIVEKEY-2-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-16 | TC-DERIVEKEY-3-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-17 | TC-DERIVEKEY-4-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-18 | TC-DERIVEKEY-5-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-19 | TC-DERIVEKEY-6-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-20 | TC-DIGESTS-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-21 | TC-DLOGIN-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-30 | TC-DLOGIN-10-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-31 | TC-DLOGIN-11-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-32 | TC-DLOGIN-12-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-33 | TC-DLOGIN-13-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-22 | TC-DLOGIN-2-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-23 | TC-DLOGIN-3-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-24 | TC-DLOGIN-4-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-25 | TC-DLOGIN-5-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-26 | TC-DLOGIN-6-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-27 | TC-DLOGIN-7-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-28 | TC-DLOGIN-8-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-29 | TC-DLOGIN-9-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-34 | TC-ECC-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-35 | TC-ECC-2-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-36 | TC-ECC-3-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-37 | TC-ECDSA-SIGN-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-38 | TC-ECDSA-SIGN-DIGESTEDDATA | unavailable | — |
| KMIPKIT-TEST-CN01-2-39 | TC-EXTRACTABLE-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-40 | TC-I18N-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-41 | TC-I18N-2-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-42 | TC-I18N-3-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-43 | TC-IMPEXP-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-44 | TC-IMPEXP-2-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-45 | TC-IMPEXP-3-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-46 | TC-IMPEXP-4-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-47 | TC-IMPEXP-5-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-48 | TC-LOGIN-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-49 | TC-LOGIN-2-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-50 | TC-LOGIN-3-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-51 | TC-MD-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-52 | TC-MD-2-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-57 | TC-MD-21-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-58 | TC-MD-22-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-59 | TC-MD-23-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-60 | TC-MD-24-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-53 | TC-MD-3-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-54 | TC-MD-4-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-55 | TC-MD-5-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-56 | TC-MD-6-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-61 | TC-MDO-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-62 | TC-MDO-2-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-63 | TC-MDO-3-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-64 | TC-OFFSET-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-65 | TC-OFFSET-2-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-66 | TC-PGP-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-67 | TC-PING-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-68 | TC-PKCS12-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-69 | TC-PKCS12-2-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-70 | TC-REENCRYPT-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-71 | TC-REENCRYPT-2-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-72 | TC-REENCRYPT-3-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-73 | TC-REENCRYPT-4-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-74 | TC-REENCRYPT-5-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-75 | TC-REENCRYPT-6-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-76 | TC-REKEY-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-85 | TC-REKEY-10-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-86 | TC-REKEY-11-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-87 | TC-REKEY-12-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-77 | TC-REKEY-2-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-78 | TC-REKEY-3-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-79 | TC-REKEY-4-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-80 | TC-REKEY-5-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-81 | TC-REKEY-6-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-82 | TC-REKEY-7-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-83 | TC-REKEY-8-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-84 | TC-REKEY-9-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-88 | TC-RNG-ATTR-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-89 | TC-RNG-ATTR-2-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-90 | TC-RSA-SIGN-DIGESTEDDATA | unavailable | — |
| KMIPKIT-TEST-CN01-2-91 | TC-SENSITIVE-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-92 | TC-SETTATTR-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-93 | TC-SETTATTR-2-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-94 | TC-SETTATTR-3-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-95 | TC-SJ-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-96 | TC-SJ-2-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-97 | TC-SJ-3-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-98 | TC-SJ-4-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-99 | TC-STREAM-ENC-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-100 | TC-STREAM-ENC-2-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-101 | TC-STREAM-ENCDEC-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-102 | TC-STREAM-HASH-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-103 | TC-STREAM-HASH-2-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-104 | TC-STREAM-HASH-3-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-105 | TC-STREAM-MAC-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-106 | TC-STREAM-SIGN-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-107 | TC-STREAM-SIGNVFY-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-108 | TC-WRAP-1-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-109 | TC-WRAP-2-21 | unavailable | — |
| KMIPKIT-TEST-CN01-2-110 | TC-WRAP-3-21 | unavailable | — |

### Test evidence by fixture availability

| Fixture state | Count |
| --- | --- |
| available | 1 |
| unavailable | 202 |

## Open discrepancies

| Discrepancy | State | Implementation gate | Affected records | Summary | Source |
| --- | --- | --- | --- | --- | --- |
| KMIPKIT-DISC-001 | open | blocked for affected records | 6 elements | Batch Error Continuation wording | KMIPKIT-SRC-spec §11.5 |
| KMIPKIT-DISC-002 | open | blocked for affected records | 8 requirements, 3 profiles | HTTPS encoding declarations conflict with a binary TTLV HTTP body requirement | KMIPKIT-SRC-profiles §5.3.1, KMIPKIT-SRC-profiles §5.3.2 |
| KMIPKIT-DISC-003 | open | blocked for affected records | 4 requirements, 35 profiles | Baseline conformance points to a second mandatory test suite | KMIPKIT-SRC-profiles §5.1.3, KMIPKIT-SRC-profiles §5.6.3, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.2 |
| KMIPKIT-DISC-004 | open | blocked for affected records | 1 profiles | Complete Server clause includes its own conformance clause | KMIPKIT-SRC-profiles §6.3 |
| KMIPKIT-DISC-005 | open | blocked for affected records | 2 profiles | XML Client MAY clause points to the JSON Client section | KMIPKIT-SRC-profiles §5.4.2, KMIPKIT-SRC-profiles §5.5.2 |
| KMIPKIT-DISC-006 | open | blocked for affected records | 3 profiles | XML Server MAY clause points to the JSON Server section | KMIPKIT-SRC-profiles §5.4.3, KMIPKIT-SRC-profiles §5.5.3 |
| KMIPKIT-DISC-007 | open | blocked for affected records | 5 requirements, 2 profiles | AES XTS Client MAY clause points to Opaque Managed Object Client | KMIPKIT-SRC-profiles §5.10.1, KMIPKIT-SRC-profiles §5.13.1 |
| KMIPKIT-DISC-008 | open | blocked for affected records | 4 requirements, 2 profiles | PKCS#11 Client MAY clause points to JSON Client | KMIPKIT-SRC-profiles §5.5.2, KMIPKIT-SRC-profiles §5.18.3 |
| KMIPKIT-DISC-009 | open | blocked for affected records | 3 profiles | PKCS#11 Server MAY clause points to JSON Server | KMIPKIT-SRC-profiles §5.5.3, KMIPKIT-SRC-profiles §5.18.4 |
| KMIPKIT-DISC-010 | open | blocked for affected records | 3 profiles | JSON Server conformance names JSON Client conditions | KMIPKIT-SRC-profiles §5.5.2, KMIPKIT-SRC-profiles §5.5.3, KMIPKIT-SRC-profiles §6.9 |
| KMIPKIT-DISC-011 | open | blocked for affected records | 10 requirements, 2 profiles | Intermediate Foundry conformance references Basic Client conditions | KMIPKIT-SRC-profiles §5.7.1, KMIPKIT-SRC-profiles §5.7.2, KMIPKIT-SRC-profiles §6.13 |
| KMIPKIT-DISC-012 | open | blocked for affected records | 10 requirements, 2 profiles | Advanced Foundry conformance references Basic Client conditions | KMIPKIT-SRC-profiles §5.7.1, KMIPKIT-SRC-profiles §5.7.3, KMIPKIT-SRC-profiles §6.14 |
| KMIPKIT-DISC-013 | open | blocked for affected records | 13 requirements, 1 elements | Protect Stop Date and Process Stop Date references disagree | KMIPKIT-SRC-spec §4.57 |
| KMIPKIT-DISC-014 | open | blocked for affected records | 2 requirements, 1 elements | MAY NOT has no defined normative strength | KMIPKIT-SRC-spec §4.34 |
| KMIPKIT-DISC-015 | open | blocked for affected records | 2 requirements, 1 elements | Lowercase shall appears in Get PKCS#12 output guidance | KMIPKIT-SRC-spec §6.1.19 |
| KMIPKIT-DISC-016 | open | blocked for affected records | 1 elements | Get Constraints error introduction is misspelled | KMIPKIT-SRC-spec §6.1.22, KMIPKIT-SRC-spec §6.1.22.1 |
| KMIPKIT-DISC-017 | open | blocked for affected records | 1 elements | Log error introduction names Query | KMIPKIT-SRC-spec §6.1.29, KMIPKIT-SRC-spec §6.1.29.1 |
| KMIPKIT-DISC-018 | open | blocked for affected records | 1 elements | PKCS#11 Correlation Value uses mixed-case Must | KMIPKIT-SRC-spec §6.1.37 |
| KMIPKIT-DISC-019 | open | blocked for affected records | 1 requirements, 1 elements | Set Attribute error introduction says Add Attribute | KMIPKIT-SRC-spec §6.1.51, KMIPKIT-SRC-spec §6.1.51.1 |
| KMIPKIT-DISC-020 | open | blocked for affected records | 2 elements | Set Constraints and Set Defaults error introductions end at Set | KMIPKIT-SRC-spec §6.1.52, KMIPKIT-SRC-spec §6.1.52.1, KMIPKIT-SRC-spec §6.1.53, KMIPKIT-SRC-spec §6.1.53.1 |
| KMIPKIT-DISC-021 | open | blocked for affected records | 4 requirements, 6 elements | HKDF text uses lowercase optional and may | KMIPKIT-SRC-spec §7.12 |
| KMIPKIT-DISC-022 | open | blocked for affected records | 3 elements | Same-major backward compatibility conflicts with the KMIPKit 2.1-only 1.x boundary | KMIPKIT-SRC-spec §9.16 |
| KMIPKIT-DISC-023 | open | blocked for affected records | 16 elements | One replacement-link invariant does not identify its actor | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-DISC-024 | open | blocked for affected records | 80 elements | REQUIRED appears in a result-reason description | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-DISC-025 | open | blocked for affected records | 19 elements | Lowercase may appears in a mixed client-rule block | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-DISC-026 | open | blocked for affected records | 26 elements | X.509 Key Usage MAY has unclear KMIP actor and applicability | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-DISC-027 | open | review before dependent implementation | none linked | Cryptographic profile case labels and target links disagree | KMIPKIT-SRC-profiles §5.9.7.13, KMIPKIT-SRC-profiles §5.9.7.14 |
| KMIPKIT-DISC-028 | open | blocked for affected records | 2 requirements, 2 profiles | Quantum Safe case heading labels QS-M-1-21 as QS-M-1-12 | KMIPKIT-SRC-profiles §5.17.1 |
| KMIPKIT-DISC-029 | open | review before dependent implementation | none linked | ECDSA test heading omits the fixture version suffix | KMIPKIT-SRC-testcases §2.38 |
| KMIPKIT-DISC-030 | open | review before dependent implementation | none linked | Login fixture href omits the dot before xml | KMIPKIT-SRC-testcases §2.48 |
| KMIPKIT-DISC-031 | open | review before dependent implementation | none linked | MAC digest test fixture target differs from its heading | KMIPKIT-SRC-testcases §2.60 |
| KMIPKIT-DISC-032 | open | review before dependent implementation | none linked | PKCS#12 test labels differ in hyphenation from target basenames | KMIPKIT-SRC-testcases §2.68, KMIPKIT-SRC-testcases §2.69 |
| KMIPKIT-DISC-033 | open | review before dependent implementation | none linked | RSA digest fixture href omits the dot before xml | KMIPKIT-SRC-testcases §2.90 |
| KMIPKIT-DISC-034 | open | review before dependent implementation | none linked | Set Attribute test labels and fixture basenames use different abbreviations | KMIPKIT-SRC-testcases §2.92, KMIPKIT-SRC-testcases §2.93, KMIPKIT-SRC-testcases §2.94 |
| KMIPKIT-DISC-035 | open | review before dependent implementation | none linked | Signed JSON test section 2.97 links to the previous case | KMIPKIT-SRC-testcases §2.96, KMIPKIT-SRC-testcases §2.97 |
| KMIPKIT-DISC-036 | open | review before dependent implementation | none linked | All 203 referenced XML fixtures are absent from the pinned source tree | KMIPKIT-SRC-profiles §5.1.3.1, KMIPKIT-SRC-testcases §2.1 |
| KMIPKIT-DISC-038 | open | blocked for affected records | 3 elements, 2 profiles | JSON profile example uses Template for a reserved Object Type value | KMIPKIT-SRC-profiles §5.5.4.1, KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-DISC-039 | open | blocked for affected records | 1 elements | Query Asynchronous Requests response table is labeled as a PKCS#11 response | KMIPKIT-SRC-spec §6.1.41 |
| KMIPKIT-DISC-040 | open | blocked for affected records | 1 elements | Table 315 RNG Retrieve Errors caption is stranded in Re-Provision error handling | KMIPKIT-SRC-spec §6.1.48.1 |
| KMIPKIT-DISC-041 | open | blocked for affected records | 1 requirements, 1 elements | The lowercase “must” in section 9.4 has unresolved RFC 2119 classification under the uppercase key-word definition in section 1.2. | KMIPKIT-SRC-spec §1.2, KMIPKIT-SRC-spec §9.4 |
| KMIPKIT-DISC-042 | open | blocked for affected records | 1 requirements, 6 elements | The section 9.11 requirement to provide at least one Device Credential field does not specify its field set. | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-DISC-045 | open | blocked for affected records | 5 elements | Single-request Encrypt/Decrypt Data optionality when both Init and Final Indicator are true | KMIPKIT-SRC-spec §6.1, KMIPKIT-SRC-spec §6.1.11, KMIPKIT-SRC-spec §6.1.17 |

## Project policies

| Policy | Provenance | Summary |
| --- | --- | --- |
| KMIPKIT-POLICY-BATCH-ERROR-CONTINUATION-ASSIGNED-OUTBOUND | approved\_product\_decision | Accept only OASIS-assigned Batch Error Continuation values for outbound requests in KMIPKIT-0007; preserve raw Enumeration values when decoding and do not claim the Table 435 extension allocation invalid. |
| KMIPKIT-POLICY-EXTENSION-PRESERVATION | AGENTS.md | Preserve unknown KMIP extension data losslessly and expose it without interpreting it as a standardized value. |
| KMIPKIT-POLICY-UNKNOWN-FUTURE-VALUE-PRESERVATION | AGENTS.md | Preserve unknown or future tags, enumeration values, and bitmask bits without assigning them standardized KMIP semantics. |
| KMIPKIT-POLICY-VENDOR-VALUE-PRESERVATION | AGENTS.md | Preserve vendor-defined values distinctly from OASIS-assigned, reserved, unused, and unknown-to-KMIPKit values. |
