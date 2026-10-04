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
| Normative requirements | 567 |
| Profiles | 35 |
| Test cases | 203 |
| Open discrepancies | 39 |
| Project policies | 3 |

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
| Strength | mandatory | 317 |
| Strength | permission\_or\_optional | 180 |
| Strength | prohibited | 47 |
| Strength | recommended | 21 |
| Scope | client\_1\_0 | 371 |
| Scope | out\_of\_scope | 1 |
| Scope | profile\_conditional | 195 |
| Direction | both | 14 |
| Direction | client\_to\_server | 378 |
| Direction | server\_to\_client | 16 |
| Direction | unclear | 159 |

## Source clause dispositions

| Disposition | Count |
| --- | --- |
| informative\_context | 118 |
| later\_1\_1 | 38 |
| non\_applicable | 345 |
| profile\_conditional | 184 |
| requirement | 264 |
| server\_only | 436 |
| source\_discrepancy | 26 |

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
| KMIPKIT-SRC-spec | 4.16 | 5 | informative\_context: 1, requirement: 4 |
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
| KMIPKIT-SRC-spec | 9.4 | 2 | requirement: 2 |
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
| KMIPKIT-REQ-SPEC-10.1.2-002-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §10.1.2 |
| KMIPKIT-REQ-SPEC-10.1.2-002-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §10.1.2 |
| KMIPKIT-REQ-SPEC-10.1.5-001-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §10.1.5 |
| KMIPKIT-REQ-SPEC-10.1.5-001-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §10.1.5 |
| KMIPKIT-REQ-SPEC-10.4-001-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §10.4 |
| KMIPKIT-REQ-SPEC-10.4-001-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §10.4 |
| KMIPKIT-REQ-SPEC-10.4-001-003 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §10.4 |
| KMIPKIT-REQ-SPEC-11-001 | prohibited | client\_1\_0 | KMIPKIT-SRC-spec §11 |
| KMIPKIT-REQ-SPEC-11.56-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
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
| KMIPKIT-REQ-SPEC-4.16-004 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §4.16 |
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
| KMIPKIT-REQ-SPEC-8-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §8 |
| KMIPKIT-REQ-SPEC-8-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §8 |
| KMIPKIT-REQ-SPEC-8-003-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §8 |
| KMIPKIT-REQ-SPEC-8-003-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §8 |
| KMIPKIT-REQ-SPEC-8.1-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §8.1 |
| KMIPKIT-REQ-SPEC-8.2-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §8.2 |
| KMIPKIT-REQ-SPEC-8.3-003 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §8.3 |
| KMIPKIT-REQ-SPEC-8.3-004 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §8.3 |
| KMIPKIT-REQ-SPEC-9.1-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.1 |
| KMIPKIT-REQ-SPEC-9.11-001 | permission\_or\_optional | profile\_conditional | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-REQ-SPEC-9.11-004-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-REQ-SPEC-9.11-004-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-REQ-SPEC-9.11-004-003 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-REQ-SPEC-9.11-006 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-REQ-SPEC-9.11-010-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-REQ-SPEC-9.11-010-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-REQ-SPEC-9.12-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §9.12 |
| KMIPKIT-REQ-SPEC-9.12-001-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.12 |
| KMIPKIT-REQ-SPEC-9.12-001-003 | recommended | client\_1\_0 | KMIPKIT-SRC-spec §9.12 |
| KMIPKIT-REQ-SPEC-9.13-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §9.13 |
| KMIPKIT-REQ-SPEC-9.13-001-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.13 |
| KMIPKIT-REQ-SPEC-9.13-001-003 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.13 |
| KMIPKIT-REQ-SPEC-9.13-001-004 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.13 |
| KMIPKIT-REQ-SPEC-9.13-001-005 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §9.13 |
| KMIPKIT-REQ-SPEC-9.19-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.19 |
| KMIPKIT-REQ-SPEC-9.2-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.2 |
| KMIPKIT-REQ-SPEC-9.20-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §9.20 |
| KMIPKIT-REQ-SPEC-9.20-001-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §9.20 |
| KMIPKIT-REQ-SPEC-9.21-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §9.21 |
| KMIPKIT-REQ-SPEC-9.21-001-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.21 |
| KMIPKIT-REQ-SPEC-9.3-001-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.3 |
| KMIPKIT-REQ-SPEC-9.3-001-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.3 |
| KMIPKIT-REQ-SPEC-9.4-001-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §9.4 |
| KMIPKIT-REQ-SPEC-9.4-001-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.4 |
| KMIPKIT-REQ-SPEC-9.4-001-003 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.4 |
| KMIPKIT-REQ-SPEC-9.4-002 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §9.4 |
| KMIPKIT-REQ-SPEC-9.5-001-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.5 |
| KMIPKIT-REQ-SPEC-9.5-001-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.5 |
| KMIPKIT-REQ-SPEC-9.6-001-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.6 |
| KMIPKIT-REQ-SPEC-9.6-001-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.6 |
| KMIPKIT-REQ-SPEC-9.6-001-003 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.6 |
| KMIPKIT-REQ-SPEC-9.7-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.7 |
| KMIPKIT-REQ-SPEC-9.8-001-001 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.8 |
| KMIPKIT-REQ-SPEC-9.8-001-002 | mandatory | client\_1\_0 | KMIPKIT-SRC-spec §9.8 |
| KMIPKIT-REQ-SPEC-9.8-001-003 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §9.8 |
| KMIPKIT-REQ-SPEC-9.9-001 | permission\_or\_optional | client\_1\_0 | KMIPKIT-SRC-spec §9.9 |

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

| Element | Kind | Direction | Scope | Source |
| --- | --- | --- | --- | --- |
| KMIPKIT-ELEM-ATTRIBUTE-ACTIVATION-DATE | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.1, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-ALTERNATIVE-NAME | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.2, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-ALWAYS-SENSITIVE | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.3, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-APPLICATION-SPECIFIC-INFORMATION | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.4, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-ARCHIVE-DATE | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.5, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-CERTIFICATE-ATTRIBUTES | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.6 |
| KMIPKIT-ELEM-ATTRIBUTE-CERTIFICATE-LENGTH | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.8, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-CERTIFICATE-TYPE | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.7, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-COMMENT | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.9, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-COMPROMISE-DATE | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.10, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-COMPROMISE-OCCURRENCE-DATE | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.11, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-CONTACT-INFORMATION | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.12, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-CRYPTOGRAPHIC-ALGORITHM | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.13, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-CRYPTOGRAPHIC-DOMAIN-PARAMETERS | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.14, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-CRYPTOGRAPHIC-LENGTH | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.15, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-CRYPTOGRAPHIC-PARAMETERS | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.16, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-CRYPTOGRAPHIC-USAGE-MASK | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.17, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-DEACTIVATION-DATE | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.18, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-DESCRIPTION | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.19, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-DESTROY-DATE | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.20, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-DIGEST | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.21, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-DIGITAL-SIGNATURE-ALGORITHM | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.22, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-EXTRACTABLE | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.23, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-FRESH | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.24, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-INITIAL-DATE | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.25, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-KEY-FORMAT-TYPE | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.26, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-KEY-VALUE-LOCATION | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.27, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-KEY-VALUE-PRESENT | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.28, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-LAST-CHANGE-DATE | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.29, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-LEASE-TIME | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.30, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-LINK | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.31, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-NAME | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.32, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-NEVER-EXTRACTABLE | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.33, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-NIST-KEY-TYPE | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.34, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-OBJECT-GROUP | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.35, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-OBJECT-TYPE | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.36, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-OPAQUE-DATA-TYPE | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.37, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-ORIGINAL-CREATION-DATE | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.38, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-PKCS-12-FRIENDLY-NAME | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.39, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-PROCESS-START-DATE | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.40, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-PROTECT-STOP-DATE | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.41, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-PROTECTION-LEVEL | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.42, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-PROTECTION-PERIOD | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.43, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-PROTECTION-STORAGE-MASK | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.44, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-QUANTUM-SAFE | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.45, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-RANDOM-NUMBER-GENERATOR | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.46, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-REVOCATION-REASON | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.47, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-ROTATE-AUTOMATIC | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.48, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-ROTATE-DATE | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.49, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-ROTATE-GENERATION | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.50, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-ROTATE-INTERVAL | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.51, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-ROTATE-LATEST | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.52, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-ROTATE-NAME | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.53, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-ROTATE-OFFSET | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.54, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-SENSITIVE | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.55, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-SHORT-UNIQUE-IDENTIFIER | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.56, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-STATE | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.57, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-1-ATTRIBUTES | attribute\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §5.1 |
| KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-2-COMMON-ATTRIBUTES | attribute\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §5.2 |
| KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-3-PRIVATE-KEY-ATTRIBUTES | attribute\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §5.3 |
| KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-4-PUBLIC-KEY-ATTRIBUTES | attribute\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §5.4 |
| KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-5-ATTRIBUTE-REFERENCE | attribute\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §5.5 |
| KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-6-CURRENT-ATTRIBUTE | attribute\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §5.6 |
| KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-7-NEW-ATTRIBUTE | attribute\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §5.7 |
| KMIPKIT-ELEM-ATTRIBUTE-UNIQUE-IDENTIFIER | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.58, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-USAGE-LIMITS | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.59, KMIPKIT-SRC-spec §7.40, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-VENDOR-ATTRIBUTE | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.60 |
| KMIPKIT-ELEM-ATTRIBUTE-X-509-CERTIFICATE-IDENTIFIER | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.61, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-X-509-CERTIFICATE-ISSUER | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.62, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ATTRIBUTE-X-509-CERTIFICATE-SUBJECT | attribute | both | client\_1\_0 | KMIPKIT-SRC-spec §4.63, KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-BITMASK-CRYPTOGRAPHIC-USAGE-MASK | bitmask | both | client\_1\_0 | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-PROTECTION-STORAGE-MASK | bitmask | both | client\_1\_0 | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-STORAGE-STATUS-MASK | bitmask | both | client\_1\_0 | KMIPKIT-SRC-spec §12.3 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-AUTHENTICATE-00100000 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-CERTIFICATE-SIGN-00001000 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-CRL-SIGN-00002000 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-DECRYPT-00000008 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-DERIVE-KEY-00000200 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-ENCRYPT-00000004 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-EXTENSIONS-XXX00000 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-FPE-DECRYPT-00800000 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-FPE-ENCRYPT-00400000 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-KEY-AGREEMENT-00000800 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-MAC-GENERATE-00000080 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-MAC-VERIFY-00000100 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-RESERVED-00000040 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-RESERVED-00000400 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-RESERVED-00004000 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-RESERVED-00008000 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-RESERVED-00010000 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-RESERVED-00020000 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-RESERVED-00040000 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-RESERVED-00080000 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-SIGN-00000001 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-UNRESTRICTED-00200000 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-UNWRAP-KEY-00000020 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-VERIFY-00000002 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-WRAP-KEY-00000010 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.1 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-CONTAINER-00000080 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-EXTENSIONS-XXXXXXX0 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-HARDWARE-00000002 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-HYPERVISOR-00000020 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-OFF-PREMISES-00000200 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-OFF-SYSTEM-00000010 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-ON-PREMISES-00000100 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-ON-PROCESSOR-00000004 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-ON-SYSTEM-00000008 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-OPERATING-SYSTEM-00000040 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-OUTSOURCED-00000800 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-SAME-JURISDICTION-00002000 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-SELF-MANAGED-00000400 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-SOFTWARE-00000001 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-VALIDATED-00001000 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.2 |
| KMIPKIT-ELEM-BITMASK-VALUE-STORAGE-STATUS-MASK-ARCHIVAL-STORAGE-00000002 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.3 |
| KMIPKIT-ELEM-BITMASK-VALUE-STORAGE-STATUS-MASK-DESTROYED-STORAGE-00000004 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.3 |
| KMIPKIT-ELEM-BITMASK-VALUE-STORAGE-STATUS-MASK-EXTENSIONS-XXXXXXX0 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.3 |
| KMIPKIT-ELEM-BITMASK-VALUE-STORAGE-STATUS-MASK-ON-LINE-STORAGE-00000001 | bitmask\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §12.3 |
| KMIPKIT-ELEM-CREDENTIAL-ATTESTATION | credential | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-CREDENTIAL-CREDENTIAL | credential | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-CREDENTIAL-DEVICE | credential | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-CREDENTIAL-HASHED-PASSWORD | credential | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-CREDENTIAL-ONE-TIME-PASSWORD | credential | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-CREDENTIAL-TICKET | credential | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-CREDENTIAL-USERNAME-AND-PASSWORD | credential | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
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
| KMIPKIT-ELEM-ENUM-VALUE-ADJUSTMENT-TYPE-DECREMENT-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.1 |
| KMIPKIT-ELEM-ENUM-VALUE-ADJUSTMENT-TYPE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.1 |
| KMIPKIT-ELEM-ENUM-VALUE-ADJUSTMENT-TYPE-INCREMENT-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.1 |
| KMIPKIT-ELEM-ENUM-VALUE-ADJUSTMENT-TYPE-NEGATE-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.1 |
| KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-DNS-NAME-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.2 |
| KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-EMAIL-ADDRESS-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.2 |
| KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.2 |
| KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-IP-ADDRESS-00000007 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.2 |
| KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-OBJECT-SERIAL-NUMBER-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.2 |
| KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-UNINTERPRETED-TEXT-STRING-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.2 |
| KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-URI-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.2 |
| KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-X-500-DISTINGUISHED-NAME-00000006 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.2 |
| KMIPKIT-ELEM-ENUM-VALUE-ASYNCHRONOUS-INDICATOR-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.3 |
| KMIPKIT-ELEM-ENUM-VALUE-ASYNCHRONOUS-INDICATOR-MANDATORY-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.3 |
| KMIPKIT-ELEM-ENUM-VALUE-ASYNCHRONOUS-INDICATOR-OPTIONAL-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.3 |
| KMIPKIT-ELEM-ENUM-VALUE-ASYNCHRONOUS-INDICATOR-PROHIBITED-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.3 |
| KMIPKIT-ELEM-ENUM-VALUE-ATTESTATION-TYPE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.4 |
| KMIPKIT-ELEM-ENUM-VALUE-ATTESTATION-TYPE-SAML-ASSERTION-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.4 |
| KMIPKIT-ELEM-ENUM-VALUE-ATTESTATION-TYPE-TCG-INTEGRITY-REPORT-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.4 |
| KMIPKIT-ELEM-ENUM-VALUE-ATTESTATION-TYPE-TPM-QUOTE-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.4 |
| KMIPKIT-ELEM-ENUM-VALUE-BATCH-ERROR-CONTINUATION-OPTION-CONTINUE-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.5 |
| KMIPKIT-ELEM-ENUM-VALUE-BATCH-ERROR-CONTINUATION-OPTION-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.5 |
| KMIPKIT-ELEM-ENUM-VALUE-BATCH-ERROR-CONTINUATION-OPTION-STOP-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.5 |
| KMIPKIT-ELEM-ENUM-VALUE-BATCH-ERROR-CONTINUATION-OPTION-UNDO-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.5 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-AEAD-00000012 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-AESKEYWRAPPADDING-0000000C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-CBC-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-CBC-MAC-0000000A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-CCM-00000008 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-CFB-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-CMAC-00000007 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-CTR-00000006 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-ECB-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-GCM-00000009 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-NISTKEYWRAP-0000000D | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-OFB-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-PCBC-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-X9-102-AESKW-0000000E | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-X9-102-AKW1-00000010 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-X9-102-AKW2-00000011 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-X9-102-TDKW-0000000F | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-XTS-0000000B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUM-VALUE-CANCELLATION-RESULT-CANCELED-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.7 |
| KMIPKIT-ELEM-ENUM-VALUE-CANCELLATION-RESULT-COMPLETED-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.7 |
| KMIPKIT-ELEM-ENUM-VALUE-CANCELLATION-RESULT-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.7 |
| KMIPKIT-ELEM-ENUM-VALUE-CANCELLATION-RESULT-FAILED-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.7 |
| KMIPKIT-ELEM-ENUM-VALUE-CANCELLATION-RESULT-UNABLE-TO-CANCEL-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.7 |
| KMIPKIT-ELEM-ENUM-VALUE-CANCELLATION-RESULT-UNAVAILABLE-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.7 |
| KMIPKIT-ELEM-ENUM-VALUE-CERTIFICATE-REQUEST-TYPE-CRMF-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.8 |
| KMIPKIT-ELEM-ENUM-VALUE-CERTIFICATE-REQUEST-TYPE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.8 |
| KMIPKIT-ELEM-ENUM-VALUE-CERTIFICATE-REQUEST-TYPE-PEM-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.8 |
| KMIPKIT-ELEM-ENUM-VALUE-CERTIFICATE-REQUEST-TYPE-PKCS-10-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.8 |
| KMIPKIT-ELEM-ENUM-VALUE-CERTIFICATE-REQUEST-TYPE-RESERVED-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.8 |
| KMIPKIT-ELEM-ENUM-VALUE-CERTIFICATE-TYPE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.9 |
| KMIPKIT-ELEM-ENUM-VALUE-CERTIFICATE-TYPE-PGP-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.9 |
| KMIPKIT-ELEM-ENUM-VALUE-CERTIFICATE-TYPE-X-509-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.9 |
| KMIPKIT-ELEM-ENUM-VALUE-CLIENT-REGISTRATION-METHOD-CLIENT-GENERATED-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.10 |
| KMIPKIT-ELEM-ENUM-VALUE-CLIENT-REGISTRATION-METHOD-CLIENT-REGISTERED-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.10 |
| KMIPKIT-ELEM-ENUM-VALUE-CLIENT-REGISTRATION-METHOD-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.10 |
| KMIPKIT-ELEM-ENUM-VALUE-CLIENT-REGISTRATION-METHOD-SERVER-ON-DEMAND-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.10 |
| KMIPKIT-ELEM-ENUM-VALUE-CLIENT-REGISTRATION-METHOD-SERVER-PRE-GENERATED-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.10 |
| KMIPKIT-ELEM-ENUM-VALUE-CLIENT-REGISTRATION-METHOD-UNSPECIFIED-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.10 |
| KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-ATTESTATION-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.11 |
| KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-DEVICE-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.11 |
| KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.11 |
| KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-HASHED-PASSWORD-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.11 |
| KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-ONE-TIME-PASSWORD-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.11 |
| KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-TICKET-00000006 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.11 |
| KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-USERNAME-AND-PASSWORD-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.11 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-3DES-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-AES-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ARIA-00000029 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-BLOWFISH-00000010 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-CAMELLIA-00000011 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-CAST5-00000012 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-CHACHA20-0000001C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-CHACHA20POLY1305-0000001E | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-DES-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-DH-0000000D | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-DSA-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-EC-0000001A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ECDH-0000000E | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ECDSA-00000006 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ECMQV-0000000F | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ED25519-00000037 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ED448-00000038 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-GOST-28147-89-00000031 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-GOST-R-34-10-2012-0000002E | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-GOST-R-34-11-2012-0000002F | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-GOST-R-34-13-2015-00000030 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-MD5-0000000C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA1-00000007 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA224-00000008 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA256-00000009 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA3-224-00000023 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA3-256-00000024 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA3-384-00000025 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA3-512-00000026 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA384-0000000A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA512-0000000B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-IDEA-00000013 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-MARS-00000014 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-MCELIECE-00000034 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-MCELIECE-6960119-00000035 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-MCELIECE-8192128-00000036 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ONE-TIME-PAD-0000001B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-POLY1305-0000001D | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-RC2-00000015 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-RC4-00000016 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-RC5-00000017 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-RSA-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SEED-0000002A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SHA3-224-0000001F | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SHA3-256-00000020 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SHA3-384-00000021 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SHA3-512-00000022 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SHAKE-128-00000027 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SHAKE-256-00000028 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SKIPJACK-00000018 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SM2-0000002B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SM3-0000002C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SM4-0000002D | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SPHINCS-256-00000033 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-TWOFISH-00000019 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-XMSS-00000032 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUM-VALUE-DATA-DECRYPT-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.13 |
| KMIPKIT-ELEM-ENUM-VALUE-DATA-ENCRYPT-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.13 |
| KMIPKIT-ELEM-ENUM-VALUE-DATA-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.13 |
| KMIPKIT-ELEM-ENUM-VALUE-DATA-HASH-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.13 |
| KMIPKIT-ELEM-ENUM-VALUE-DATA-MAC-MAC-DATA-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.13 |
| KMIPKIT-ELEM-ENUM-VALUE-DATA-RNG-RETRIEVE-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.13 |
| KMIPKIT-ELEM-ENUM-VALUE-DATA-SIGN-SIGNATURE-DATA-00000006 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.13 |
| KMIPKIT-ELEM-ENUM-VALUE-DATA-SIGNATURE-VERIFY-00000007 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.13 |
| KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-ASYMMETRIC-KEY-00000008 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.14 |
| KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-AWS-SIGNATURE-VERSION-4-00000009 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.14 |
| KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-ENCRYPT-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.14 |
| KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.14 |
| KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-HASH-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.14 |
| KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-HKDF-0000000A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.14 |
| KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-HMAC-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.14 |
| KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-NIST800-108-C-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.14 |
| KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-NIST800-108-DPI-00000007 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.14 |
| KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-NIST800-108-F-00000006 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.14 |
| KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-PBKDF2-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.14 |
| KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-DELETED-00000006 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.15 |
| KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.15 |
| KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-KEY-MATERIAL-DELETED-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.15 |
| KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-KEY-MATERIAL-SHREDDED-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.15 |
| KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-META-DATA-DELETED-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.15 |
| KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-META-DATA-SHREDDED-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.15 |
| KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-SHREDDED-00000007 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.15 |
| KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-UNSPECIFIED-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.15 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-DSA-WITH-SHA-1-00000009 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-DSA-WITH-SHA224-0000000A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-DSA-WITH-SHA256-0000000B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-ECDSA-WITH-SHA-1-0000000C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-ECDSA-WITH-SHA224-0000000D | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-ECDSA-WITH-SHA256-0000000E | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-ECDSA-WITH-SHA384-0000000F | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-ECDSA-WITH-SHA512-00000010 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-MD2-WITH-RSA-ENCRYPTION-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-MD5-WITH-RSA-ENCRYPTION-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-RSASSA-PSS-00000008 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA-1-WITH-RSA-ENCRYPTION-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA-224-WITH-RSA-ENCRYPTION-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA-256-WITH-RSA-ENCRYPTION-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA-384-WITH-RSA-ENCRYPTION-00000006 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA-512-WITH-RSA-ENCRYPTION-00000007 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA3-256-WITH-RSA-ENCRYPTION-00000011 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA3-384-WITH-RSA-ENCRYPTION-00000012 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA3-512-WITH-RSA-ENCRYPTION-00000013 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUM-VALUE-DRBG-ALGORITHM-CTR-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.17 |
| KMIPKIT-ELEM-ENUM-VALUE-DRBG-ALGORITHM-DUAL-EC-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.17 |
| KMIPKIT-ELEM-ENUM-VALUE-DRBG-ALGORITHM-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.17 |
| KMIPKIT-ELEM-ENUM-VALUE-DRBG-ALGORITHM-HASH-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.17 |
| KMIPKIT-ELEM-ENUM-VALUE-DRBG-ALGORITHM-HMAC-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.17 |
| KMIPKIT-ELEM-ENUM-VALUE-DRBG-ALGORITHM-UNSPECIFIED-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.17 |
| KMIPKIT-ELEM-ENUM-VALUE-ENCODING-OPTION-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.18 |
| KMIPKIT-ELEM-ENUM-VALUE-ENCODING-OPTION-NO-ENCODING-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.18 |
| KMIPKIT-ELEM-ENUM-VALUE-ENCODING-OPTION-TTLV-ENCODING-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.18 |
| KMIPKIT-ELEM-ENUM-VALUE-ENDPOINT-ROLE-CLIENT-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.19 |
| KMIPKIT-ELEM-ENUM-VALUE-ENDPOINT-ROLE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.19 |
| KMIPKIT-ELEM-ENUM-VALUE-ENDPOINT-ROLE-SERVER-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.19 |
| KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.20 |
| KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-GP-X-CHANGE-NOTICE-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.20 |
| KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-GP-X-ORIGINAL-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.20 |
| KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-K-CHANGE-NOTICE-00000007 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.20 |
| KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-K-ORIGINAL-00000006 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.20 |
| KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-UNSPECIFIED-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.20 |
| KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-X-CHANGE-NOTICE-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.20 |
| KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-X-ORIGINAL-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.20 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-MD2-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-MD4-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-MD5-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-RIPEMD-160-00000009 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-1-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-224-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-256-00000006 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-384-00000007 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-512-00000008 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-512-224-0000000C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-512-256-0000000D | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA3-224-0000000E | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA3-256-0000000F | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA3-384-00000010 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA3-512-00000011 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-TIGER-0000000A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-WHIRLPOOL-0000000B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUM-VALUE-INTEROP-FUNCTION-BEGIN-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.22 |
| KMIPKIT-ELEM-ENUM-VALUE-INTEROP-FUNCTION-END-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.22 |
| KMIPKIT-ELEM-ENUM-VALUE-INTEROP-FUNCTION-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.22 |
| KMIPKIT-ELEM-ENUM-VALUE-INTEROP-FUNCTION-RESET-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.22 |
| KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-BIG-INTEGER-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-BOOLEAN-00000006 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-BYTE-STRING-00000008 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-DATE-TIME-00000009 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-DATE-TIME-EXTENDED-0000000B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-ENUMERATION-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-INTEGER-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-INTERVAL-0000000A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-LONG-INTEGER-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-STRUCTURE-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-TEXT-STRING-00000007 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-COMPRESSION-TYPE-EC-PUBLIC-KEY-TYPE-UNCOMPRESSED-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.24 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-COMPRESSION-TYPE-EC-PUBLIC-KEY-TYPE-X9-62-COMPRESSED-CHAR2-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.24 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-COMPRESSION-TYPE-EC-PUBLIC-KEY-TYPE-X9-62-COMPRESSED-PRIME-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.24 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-COMPRESSION-TYPE-EC-PUBLIC-KEY-TYPE-X9-62-HYBRID-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.24 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-COMPRESSION-TYPE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.24 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-ECPRIVATEKEY-00000006 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-OPAQUE-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-PKCS-1-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-PKCS-10-00000017 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-PKCS-12-00000016 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-PKCS-8-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-RAW-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-RESERVED-0000000E | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-RESERVED-0000000F | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-RESERVED-00000010 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-RESERVED-00000011 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-RESERVED-00000012 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-RESERVED-00000013 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-DH-PRIVATE-KEY-0000000C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-DH-PUBLIC-KEY-0000000D | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-DSA-PRIVATE-KEY-00000008 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-DSA-PUBLIC-KEY-00000009 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-EC-PRIVATE-KEY-00000014 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-EC-PUBLIC-KEY-00000015 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-RSA-PRIVATE-KEY-0000000A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-RSA-PUBLIC-KEY-0000000B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-SYMMETRIC-KEY-00000007 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-X-509-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-BDK-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-CVK-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-DEK-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-DUKPT-00000016 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-IV-00000017 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-KEK-0000000B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MAC16609-0000000C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MAC97971-0000000D | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MAC97972-0000000E | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MAC97973-0000000F | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MAC97974-00000010 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MAC97975-00000011 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKAC-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKCP-00000009 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKDAC-00000007 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKDN-00000008 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKOTH-0000000A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKSMC-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKSMI-00000006 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-PVKIBM-00000013 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-PVKOTH-00000015 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-PVKPVV-00000014 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-TRKBK-00000018 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-ZPK-00000012 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-VALUE-LOCATION-TYPE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.27 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-VALUE-LOCATION-TYPE-UNINTERPRETED-TEXT-STRING-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.27 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-VALUE-LOCATION-TYPE-URI-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.27 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-WRAP-TYPE-AS-REGISTERED-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.29 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-WRAP-TYPE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.29 |
| KMIPKIT-ELEM-ENUM-VALUE-KEY-WRAP-TYPE-NOT-WRAPPED-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.29 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-CERTIFICATE-LINK-00000101 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-CHILD-LINK-00000109 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-DERIVATION-BASE-OBJECT-LINK-00000104 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-DERIVED-KEY-LINK-00000105 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-NEXT-LINK-0000010B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-PARENT-LINK-00000108 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-PKCS-12-CERTIFICATE-LINK-0000010C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-PKCS-12-PASSWORD-LINK-0000010D | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-PREVIOUS-LINK-0000010A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-PRIVATE-KEY-LINK-00000103 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-PUBLIC-KEY-LINK-00000102 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-REPLACED-OBJECT-LINK-00000107 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-REPLACEMENT-OBJECT-LINK-00000106 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-WRAPPING-KEY-LINK-0000010E | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUM-VALUE-MASK-GENERATOR-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.30 |
| KMIPKIT-ELEM-ENUM-VALUE-MASK-GENERATOR-MFG1-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.30 |
| KMIPKIT-ELEM-ENUM-VALUE-NAME-TYPE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.31 |
| KMIPKIT-ELEM-ENUM-VALUE-NAME-TYPE-UNINTERPRETED-TEXT-STRING-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.31 |
| KMIPKIT-ELEM-ENUM-VALUE-NAME-TYPE-URI-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.31 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PRIVATE-AUTHENTICATION-KEY-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PRIVATE-AUTHORIZATION-KEY-00000012 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PRIVATE-EPHEMERAL-KEY-AGREEMENT-KEY-0000000F | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PRIVATE-KEY-TRANSPORT-KEY-0000000A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PRIVATE-SIGNATURE-KEY-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PRIVATE-STATIC-KEY-AGREEMENT-KEY-0000000D | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PUBLIC-AUTHENTICATION-KEY-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PUBLIC-AUTHORIZATION-KEY-00000013 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PUBLIC-EPHEMERAL-KEY-AGREEMENT-KEY-00000010 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PUBLIC-KEY-TRANSPORT-KEY-0000000B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PUBLIC-SIGNATURE-VERIFICATION-KEY-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PUBLIC-STATIC-KEY-AGREEMENT-KEY-0000000E | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-AUTHENTICATION-KEY-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-AUTHORIZATION-KEY-00000011 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-DATA-ENCRYPTION-KEY-00000006 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-KEY-AGREEMENT-KEY-0000000C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-KEY-WRAPPING-KEY-00000007 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-MASTER-KEY-00000009 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-RANDOM-NUMBER-GENERATION-KEY-00000008 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-GROUP-MEMBER-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.33 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-GROUP-MEMBER-GROUP-MEMBER-DEFAULT-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.33 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-GROUP-MEMBER-GROUP-MEMBER-FRESH-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.33 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-CERTIFICATE-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-CERTIFICATE-REQUEST-0000000A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-OPAQUE-OBJECT-00000008 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-PGP-KEY-00000009 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-PRIVATE-KEY-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-PUBLIC-KEY-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-RESERVED-00000006 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-SECRET-DATA-00000007 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-SPLIT-KEY-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-SYMMETRIC-KEY-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-ENUM-VALUE-OPAQUE-DATA-TYPE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.35 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-ACTIVATE-00000012 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-ADD-ATTRIBUTE-0000000D | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-ADJUST-ATTRIBUTE-00000030 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-ARCHIVE-00000015 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-CANCEL-00000019 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-CERTIFY-00000006 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-CHECK-00000009 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-CREATE-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-CREATE-KEY-PAIR-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-CREATE-SPLIT-KEY-00000028 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-DECRYPT-00000020 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-DELEGATED-LOGIN-0000002F | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-DELETE-ATTRIBUTE-0000000F | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-DERIVE-KEY-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-DESTROY-00000014 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-DISCOVER-VERSIONS-0000001E | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-ENCRYPT-0000001F | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-EXPORT-0000002B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-GET-0000000A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-GET-ATTRIBUTE-LIST-0000000C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-GET-ATTRIBUTES-0000000B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-GET-CONSTRAINTS-00000038 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-GET-USAGE-ALLOCATION-00000011 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-HASH-00000027 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-IMPORT-0000002A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-INTEROP-00000034 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-JOIN-SPLIT-KEY-00000029 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-LOCATE-00000008 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-LOG-0000002C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-LOGIN-0000002D | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-LOGOUT-0000002E | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-MAC-00000023 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-MAC-VERIFY-00000024 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-MODIFY-ATTRIBUTE-0000000E | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-NOTIFY-0000001B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-OBTAIN-LEASE-00000010 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-PING-0000003B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-PKCS-11-00000033 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-POLL-0000001A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-PROCESS-0000003A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-PUT-0000001C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-QUERY-00000018 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-QUERY-ASYNCHRONOUS-REQUESTS-00000039 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RE-CERTIFY-00000007 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RE-KEY-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RE-KEY-KEY-PAIR-0000001D | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RE-PROVISION-00000035 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RECOVER-00000016 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-REGISTER-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-REVOKE-00000013 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RNG-RETRIEVE-00000025 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RNG-SEED-00000026 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SET-ATTRIBUTE-00000031 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SET-CONSTRAINTS-00000037 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SET-DEFAULTS-00000036 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SET-ENDPOINT-ROLE-00000032 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SIGN-00000021 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SIGNATURE-VERIFY-00000022 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-VALIDATE-00000017 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-ANSI-X9-23-00000006 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.37 |
| KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.37 |
| KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-ISO-10126-00000007 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.37 |
| KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-NONE-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.37 |
| KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-OAEP-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.37 |
| KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-PKCS1-V1-5-00000008 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.37 |
| KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-PKCS5-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.37 |
| KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-PSS-0000000A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.37 |
| KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-SSL3-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.37 |
| KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-X9-31-00000009 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.37 |
| KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-ZEROS-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.37 |
| KMIPKIT-ELEM-ENUM-VALUE-PROCESSING-STAGE-COMPLETED-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.40 |
| KMIPKIT-ELEM-ENUM-VALUE-PROCESSING-STAGE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.40 |
| KMIPKIT-ELEM-ENUM-VALUE-PROCESSING-STAGE-IN-PROCESS-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.40 |
| KMIPKIT-ELEM-ENUM-VALUE-PROCESSING-STAGE-SUBMITTED-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.40 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-ADVANCED-CRYPTOGRAPHIC-CLIENT-0000010E | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-ADVANCED-CRYPTOGRAPHIC-SERVER-0000010F | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-ADVANCED-SYMMETRIC-KEY-FOUNDRY-CLIENT-00000114 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-AES-XTS-CLIENT-00000124 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-AES-XTS-SERVER-00000125 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-ASYMMETRIC-KEY-LIFECYCLE-CLIENT-0000010A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-ASYMMETRIC-KEY-LIFECYCLE-SERVER-0000010B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-BASELINE-CLIENT-0000012A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-BASELINE-SERVER-0000012B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-BASIC-CRYPTOGRAPHIC-CLIENT-0000010C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-BASIC-CRYPTOGRAPHIC-SERVER-0000010D | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-BASIC-SYMMETRIC-KEY-FOUNDRY-CLIENT-00000112 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-COMPLETE-SERVER-0000012C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-COMPLETE-SERVER-BASIC-00000104 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-COMPLETE-SERVER-TLS-V1-2-00000105 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-HTTPS-CLIENT-0000011E | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-HTTPS-SERVER-0000011F | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-INTERMEDIATE-SYMMETRIC-KEY-FOUNDRY-CLIENT-00000113 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-JSON-CLIENT-00000120 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-JSON-SERVER-00000121 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-OPAQUE-MANAGED-OBJECT-STORE-CLIENT-00000116 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-OPAQUE-MANAGED-OBJECT-STORE-SERVER-00000117 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-PKCS-11-CLIENT-00000128 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-PKCS-11-SERVER-00000129 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-QUANTUM-SAFE-CLIENT-00000126 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-QUANTUM-SAFE-SERVER-00000127 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-RESERVED-00000001-00000103 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-RESERVED-00000118 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-RESERVED-00000119 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-RESERVED-0000011A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-RESERVED-0000011B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-RNG-CRYPTOGRAPHIC-CLIENT-00000110 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-RNG-CRYPTOGRAPHIC-SERVER-00000111 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-STORAGE-ARRAY-WITH-SELF-ENCRYPTING-DRIVE-CLIENT-0000011C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-STORAGE-ARRAY-WITH-SELF-ENCRYPTING-DRIVE-SERVER-0000011D | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-SYMMETRIC-KEY-FOUNDRY-SERVER-00000115 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-SYMMETRIC-KEY-LIFECYCLE-CLIENT-00000108 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-SYMMETRIC-KEY-LIFECYCLE-SERVER-00000109 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-TAPE-LIBRARY-CLIENT-00000106 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-TAPE-LIBRARY-SERVER-00000107 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-XML-CLIENT-00000122 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-XML-SERVER-00000123 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUM-VALUE-PROTECTION-LEVEL-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.42 |
| KMIPKIT-ELEM-ENUM-VALUE-PROTECTION-LEVEL-HIGH-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.42 |
| KMIPKIT-ELEM-ENUM-VALUE-PROTECTION-LEVEL-LOW-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.42 |
| KMIPKIT-ELEM-ENUM-VALUE-PUT-FUNCTION-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.43 |
| KMIPKIT-ELEM-ENUM-VALUE-PUT-FUNCTION-NEW-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.43 |
| KMIPKIT-ELEM-ENUM-VALUE-PUT-FUNCTION-REPLACE-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.43 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-APPLICATION-NAMESPACES-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-ATTESTATION-TYPES-00000007 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-CAPABILITIES-0000000B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-CLIENT-REGISTRATION-METHODS-0000000C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-DEFAULTS-INFORMATION-0000000D | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-EXTENSION-LIST-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-EXTENSION-MAP-00000006 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-OBJECTS-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-OPERATIONS-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-PROFILES-0000000A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-RNGS-00000008 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-SERVER-INFORMATION-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-STORAGE-PROTECTION-MASKS-0000000E | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-VALIDATIONS-00000009 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB163V1-00000027 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB163V2-00000028 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB163V3-00000029 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB176V1-0000002A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB208W1-0000002E | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB272W1-00000032 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB304W1-00000033 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB368W1-00000035 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB191V1-0000002B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB191V2-0000002C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB191V3-0000002D | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB239V1-0000002F | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB239V2-00000030 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB239V3-00000031 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB359V1-00000034 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB431R1-00000036 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9P192V2-00000022 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9P192V3-00000023 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9P239V1-00000024 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9P239V2-00000025 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9P239V3-00000026 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-B-163-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-B-233-00000006 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-B-283-00000009 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-B-409-0000000C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-B-571-0000000F | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP160R1-00000037 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP160T1-00000038 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP192R1-00000039 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP192T1-0000003A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP224R1-0000003B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP224T1-0000003C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP256R1-0000003D | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP256T1-0000003E | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP320R1-0000003F | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP320T1-00000040 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP384R1-00000041 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP384T1-00000042 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP512R1-00000043 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP512T1-00000044 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-CURVE25519-00000045 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-CURVE448-00000046 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-K-163-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-K-233-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-K-283-00000008 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-K-409-0000000B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-K-571-0000000E | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-P-192-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-P-224-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-P-256-00000007 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-P-384-0000000A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-P-521-0000000D | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP112R1-00000010 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP112R2-00000011 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP128R1-00000012 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP128R2-00000013 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP160K1-00000014 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP160R1-00000015 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP160R2-00000016 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP192K1-00000017 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP224K1-00000018 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP256K1-00000019 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT113R1-0000001A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT113R2-0000001B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT131R1-0000001C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT131R2-0000001D | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT163R1-0000001E | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT193R1-0000001F | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT193R2-00000020 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT239K1-00000021 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-APPLICATION-NAMESPACE-NOT-SUPPORTED-0000000F | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ATTESTATION-FAILED-00000015 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ATTESTATION-REQUIRED-00000014 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ATTRIBUTE-INSTANCE-NOT-FOUND-00000020 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ATTRIBUTE-NOT-FOUND-00000021 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ATTRIBUTE-READ-ONLY-00000022 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ATTRIBUTE-SINGLE-VALUED-00000023 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-AUTHENTICATION-NOT-SUCCESSFUL-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-BAD-CRYPTOGRAPHIC-PARAMETERS-00000024 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-BAD-PASSWORD-00000025 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-CODEC-ERROR-00000026 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-CONSTRAINT-VIOLATION-0000004B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-CRYPTOGRAPHIC-FAILURE-0000000A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-DUPLICATE-PROCESS-REQUEST-0000004C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ENCODING-OPTION-ERROR-00000012 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-FEATURE-NOT-SUPPORTED-00000008 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-GENERAL-FAILURE-00000100 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ILLEGAL-OBJECT-TYPE-00000028 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INCOMPATIBLE-CRYPTOGRAPHIC-USAGE-MASK-00000029 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INTERNAL-SERVER-ERROR-0000002A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-ASYNCHRONOUS-CORRELATION-VALUE-0000002B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-ATTRIBUTE-0000002C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-ATTRIBUTE-VALUE-0000002D | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-CORRELATION-VALUE-0000002E | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-CSR-0000002F | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-DATA-TYPE-0000001C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-FIELD-00000007 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-MESSAGE-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-OBJECT-TYPE-00000030 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-TICKET-00000019 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ITEM-NOT-FOUND-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-KEY-COMPRESSION-TYPE-NOT-SUPPORTED-00000011 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-KEY-FORMAT-TYPE-NOT-SUPPORTED-00000010 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-KEY-VALUE-NOT-PRESENT-00000013 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-KEY-WRAP-TYPE-NOT-SUPPORTED-00000032 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-MISSING-DATA-00000006 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-MISSING-INITIALIZATION-VECTOR-00000034 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-MULTI-VALUED-ATTRIBUTE-0000001E | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-NON-UNIQUE-NAME-ATTRIBUTE-00000035 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-NOT-AUTHORISED-00000039 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-NOT-EXTRACTABLE-00000017 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-NUMERIC-RANGE-0000001B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-OBJECT-ALREADY-EXISTS-00000018 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-OBJECT-ARCHIVED-0000000D | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-OBJECT-DESTROYED-00000036 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-OBJECT-NOT-FOUND-00000037 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-OPERATION-CANCELED-BY-REQUESTER-00000009 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-OPERATION-NOT-SUPPORTED-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-PERMISSION-DENIED-0000000C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-PKCS-11-CODEC-ERROR-00000045 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-PKCS-11-INVALID-FUNCTION-00000046 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-PKCS-11-INVALID-INTERFACE-00000047 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-PRIVATE-PROTECTION-STORAGE-UNAVAILABLE-00000048 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-PROTECTION-STORAGE-UNAVAILABLE-00000044 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-PUBLIC-PROTECTION-STORAGE-UNAVAILABLE-00000049 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-READ-ONLY-ATTRIBUTE-0000001D | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-RESERVED-0000000B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-RESERVED-0000000E | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-RESERVED-00000027 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-RESERVED-00000031 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-RESERVED-00000033 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-RESERVED-00000038 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-RESPONSE-TOO-LARGE-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-SENSITIVE-00000016 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-SERVER-LIMIT-EXCEEDED-0000003A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-UNKNOWN-ENUMERATION-0000003B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-UNKNOWN-MESSAGE-EXTENSION-0000003C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-UNKNOWN-OBJECT-GROUP-0000004A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-UNKNOWN-TAG-0000003D | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-UNSUPPORTED-ATTRIBUTE-0000001F | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-UNSUPPORTED-CRYPTOGRAPHIC-PARAMETERS-0000003E | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-UNSUPPORTED-PROTOCOL-VERSION-0000003F | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-USAGE-LIMIT-EXCEEDED-0000001A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-WRAPPING-OBJECT-ARCHIVED-00000040 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-WRAPPING-OBJECT-DESTROYED-00000041 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-WRAPPING-OBJECT-NOT-FOUND-00000042 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-WRONG-KEY-LIFECYCLE-STATE-00000043 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-STATUS-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.47 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-STATUS-OPERATION-FAILED-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.47 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-STATUS-OPERATION-PENDING-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.47 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-STATUS-OPERATION-UNDONE-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.47 |
| KMIPKIT-ELEM-ENUM-VALUE-RESULT-STATUS-SUCCESS-00000000 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.47 |
| KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-AFFILIATION-CHANGED-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.48 |
| KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-CA-COMPROMISE-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.48 |
| KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-CESSATION-OF-OPERATION-00000006 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.48 |
| KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.48 |
| KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-KEY-COMPROMISE-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.48 |
| KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-PRIVILEGE-WITHDRAWN-00000007 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.48 |
| KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-SUPERSEDED-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.48 |
| KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-UNSPECIFIED-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.48 |
| KMIPKIT-ELEM-ENUM-VALUE-RNG-ALGORITHM-ANSI-X9-31-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.49 |
| KMIPKIT-ELEM-ENUM-VALUE-RNG-ALGORITHM-ANSI-X9-62-00000006 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.49 |
| KMIPKIT-ELEM-ENUM-VALUE-RNG-ALGORITHM-DRBG-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.49 |
| KMIPKIT-ELEM-ENUM-VALUE-RNG-ALGORITHM-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.49 |
| KMIPKIT-ELEM-ENUM-VALUE-RNG-ALGORITHM-FIPS-186-2-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.49 |
| KMIPKIT-ELEM-ENUM-VALUE-RNG-ALGORITHM-NRBG-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.49 |
| KMIPKIT-ELEM-ENUM-VALUE-RNG-ALGORITHM-UNSPECIFIED-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.49 |
| KMIPKIT-ELEM-ENUM-VALUE-RNG-MODE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.50 |
| KMIPKIT-ELEM-ENUM-VALUE-RNG-MODE-NON-SHARED-INSTANTIATION-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.50 |
| KMIPKIT-ELEM-ENUM-VALUE-RNG-MODE-SHARED-INSTANTIATION-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.50 |
| KMIPKIT-ELEM-ENUM-VALUE-RNG-MODE-UNSPECIFIED-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.50 |
| KMIPKIT-ELEM-ENUM-VALUE-ROTATE-NAME-TYPE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.51 |
| KMIPKIT-ELEM-ENUM-VALUE-ROTATE-NAME-TYPE-UNINTERPRETED-TEXT-STRING-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.51 |
| KMIPKIT-ELEM-ENUM-VALUE-ROTATE-NAME-TYPE-URI-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.51 |
| KMIPKIT-ELEM-ENUM-VALUE-SECRET-DATA-TYPE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.52 |
| KMIPKIT-ELEM-ENUM-VALUE-SECRET-DATA-TYPE-PASSWORD-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.52 |
| KMIPKIT-ELEM-ENUM-VALUE-SECRET-DATA-TYPE-SEED-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.52 |
| KMIPKIT-ELEM-ENUM-VALUE-SHREDDING-ALGORITHM-CRYPTOGRAPHIC-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.53 |
| KMIPKIT-ELEM-ENUM-VALUE-SHREDDING-ALGORITHM-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.53 |
| KMIPKIT-ELEM-ENUM-VALUE-SHREDDING-ALGORITHM-UNSPECIFIED-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.53 |
| KMIPKIT-ELEM-ENUM-VALUE-SHREDDING-ALGORITHM-UNSUPPORTED-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.53 |
| KMIPKIT-ELEM-ENUM-VALUE-SPLIT-KEY-METHOD-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.54 |
| KMIPKIT-ELEM-ENUM-VALUE-SPLIT-KEY-METHOD-POLYNOMIAL-SHARING-GF-2-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.54 |
| KMIPKIT-ELEM-ENUM-VALUE-SPLIT-KEY-METHOD-POLYNOMIAL-SHARING-GF-2-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.54 |
| KMIPKIT-ELEM-ENUM-VALUE-SPLIT-KEY-METHOD-POLYNOMIAL-SHARING-PRIME-FIELD-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.54 |
| KMIPKIT-ELEM-ENUM-VALUE-SPLIT-KEY-METHOD-XOR-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.54 |
| KMIPKIT-ELEM-ENUM-VALUE-STATE-ACTIVE-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.55 |
| KMIPKIT-ELEM-ENUM-VALUE-STATE-COMPROMISED-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.55 |
| KMIPKIT-ELEM-ENUM-VALUE-STATE-DEACTIVATED-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.55 |
| KMIPKIT-ELEM-ENUM-VALUE-STATE-DESTROYED-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.55 |
| KMIPKIT-ELEM-ENUM-VALUE-STATE-DESTROYED-COMPROMISED-00000006 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.55 |
| KMIPKIT-ELEM-ENUM-VALUE-STATE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.55 |
| KMIPKIT-ELEM-ENUM-VALUE-STATE-PRE-ACTIVE-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.55 |
| KMIPKIT-ELEM-ENUM-VALUE-TICKET-TYPE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.57 |
| KMIPKIT-ELEM-ENUM-VALUE-TICKET-TYPE-LOGIN-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.57 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-CERTIFY-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-CREATE-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-CREATE-KEY-PAIR-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-CREATE-KEY-PAIR-PRIVATE-KEY-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-CREATE-KEY-PAIR-PUBLIC-KEY-00000006 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-CREATE-SPLIT-KEY-00000007 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-DERIVE-KEY-00000008 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-ID-PLACEHOLDER-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-IMPORT-00000009 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-JOIN-SPLIT-KEY-0000000A | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-LOCATE-0000000B | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-RE-CERTIFY-0000000E | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-RE-KEY-0000000D | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-RE-KEY-KEY-PAIR-0000000F | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-RE-KEY-KEY-PAIR-PRIVATE-KEY-00000010 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-RE-KEY-KEY-PAIR-PUBLIC-KEY-00000011 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-REGISTER-0000000C | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUM-VALUE-UNWRAP-MODE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.59 |
| KMIPKIT-ELEM-ENUM-VALUE-UNWRAP-MODE-NOT-PROCESSED-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.59 |
| KMIPKIT-ELEM-ENUM-VALUE-UNWRAP-MODE-PROCESSED-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.59 |
| KMIPKIT-ELEM-ENUM-VALUE-UNWRAP-MODE-UNSPECIFIED-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.59 |
| KMIPKIT-ELEM-ENUM-VALUE-USAGE-LIMITS-UNIT-BYTE-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.60 |
| KMIPKIT-ELEM-ENUM-VALUE-USAGE-LIMITS-UNIT-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.60 |
| KMIPKIT-ELEM-ENUM-VALUE-USAGE-LIMITS-UNIT-OBJECT-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.60 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-AUTHORITY-TYPE-COMMON-CRITERIA-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.63 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-AUTHORITY-TYPE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.63 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-AUTHORITY-TYPE-NIST-CMVP-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.63 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-AUTHORITY-TYPE-UNSPECIFIED-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.63 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-TYPE-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.64 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-TYPE-FIRMWARE-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.64 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-TYPE-HARDWARE-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.64 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-TYPE-HYBRID-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.64 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-TYPE-SOFTWARE-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.64 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-TYPE-UNSPECIFIED-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.64 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDITY-INDICATOR-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.61 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDITY-INDICATOR-INVALID-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.61 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDITY-INDICATOR-UNKNOWN-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.61 |
| KMIPKIT-ELEM-ENUM-VALUE-VALIDITY-INDICATOR-VALID-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.61 |
| KMIPKIT-ELEM-ENUM-VALUE-WRAPPING-METHOD-ENCRYPT-00000001 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.62 |
| KMIPKIT-ELEM-ENUM-VALUE-WRAPPING-METHOD-ENCRYPT-THEN-MAC-SIGN-00000003 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.62 |
| KMIPKIT-ELEM-ENUM-VALUE-WRAPPING-METHOD-EXTENSIONS-8XXXXXXX | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.62 |
| KMIPKIT-ELEM-ENUM-VALUE-WRAPPING-METHOD-MAC-SIGN-00000002 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.62 |
| KMIPKIT-ELEM-ENUM-VALUE-WRAPPING-METHOD-MAC-SIGN-THEN-ENCRYPT-00000004 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.62 |
| KMIPKIT-ELEM-ENUM-VALUE-WRAPPING-METHOD-TR-31-00000005 | enumeration\_value | both | client\_1\_0 | KMIPKIT-SRC-spec §11.62 |
| KMIPKIT-ELEM-ENUMERATION-ADJUSTMENT-TYPE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.1 |
| KMIPKIT-ELEM-ENUMERATION-ALTERNATIVE-NAME-TYPE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.2 |
| KMIPKIT-ELEM-ENUMERATION-ASYNCHRONOUS-INDICATOR | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.3 |
| KMIPKIT-ELEM-ENUMERATION-ATTESTATION-TYPE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.4 |
| KMIPKIT-ELEM-ENUMERATION-BATCH-ERROR-CONTINUATION-OPTION | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.5 |
| KMIPKIT-ELEM-ENUMERATION-BLOCK-CIPHER-MODE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.6 |
| KMIPKIT-ELEM-ENUMERATION-CANCELLATION-RESULT | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.7 |
| KMIPKIT-ELEM-ENUMERATION-CERTIFICATE-REQUEST-TYPE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.8 |
| KMIPKIT-ELEM-ENUMERATION-CERTIFICATE-TYPE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.9 |
| KMIPKIT-ELEM-ENUMERATION-CLIENT-REGISTRATION-METHOD | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.10 |
| KMIPKIT-ELEM-ENUMERATION-CREDENTIAL-TYPE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.11 |
| KMIPKIT-ELEM-ENUMERATION-CRYPTOGRAPHIC-ALGORITHM | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.12 |
| KMIPKIT-ELEM-ENUMERATION-DATA | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.13 |
| KMIPKIT-ELEM-ENUMERATION-DERIVATION-METHOD | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.14 |
| KMIPKIT-ELEM-ENUMERATION-DESTROY-ACTION | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.15 |
| KMIPKIT-ELEM-ENUMERATION-DIGITAL-SIGNATURE-ALGORITHM | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.16 |
| KMIPKIT-ELEM-ENUMERATION-DRBG-ALGORITHM | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.17 |
| KMIPKIT-ELEM-ENUMERATION-ENCODING-OPTION | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.18 |
| KMIPKIT-ELEM-ENUMERATION-ENDPOINT-ROLE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.19 |
| KMIPKIT-ELEM-ENUMERATION-FIPS186-VARIATION | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.20 |
| KMIPKIT-ELEM-ENUMERATION-HASHING-ALGORITHM | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.21 |
| KMIPKIT-ELEM-ENUMERATION-INTEROP-FUNCTION | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.22 |
| KMIPKIT-ELEM-ENUMERATION-ITEM-TYPE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.23 |
| KMIPKIT-ELEM-ENUMERATION-KEY-COMPRESSION-TYPE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.24 |
| KMIPKIT-ELEM-ENUMERATION-KEY-FORMAT-TYPE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.25 |
| KMIPKIT-ELEM-ENUMERATION-KEY-ROLE-TYPE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.26 |
| KMIPKIT-ELEM-ENUMERATION-KEY-VALUE-LOCATION-TYPE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.27 |
| KMIPKIT-ELEM-ENUMERATION-KEY-WRAP-TYPE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.29 |
| KMIPKIT-ELEM-ENUMERATION-LINK-TYPE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.28 |
| KMIPKIT-ELEM-ENUMERATION-MASK-GENERATOR | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.30 |
| KMIPKIT-ELEM-ENUMERATION-NAME-TYPE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.31 |
| KMIPKIT-ELEM-ENUMERATION-NIST-KEY-TYPE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.32 |
| KMIPKIT-ELEM-ENUMERATION-OBJECT-GROUP-MEMBER | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.33 |
| KMIPKIT-ELEM-ENUMERATION-OBJECT-TYPE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-ELEM-ENUMERATION-OPAQUE-DATA-TYPE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.35 |
| KMIPKIT-ELEM-ENUMERATION-OPERATION | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.36 |
| KMIPKIT-ELEM-ENUMERATION-PADDING-METHOD | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.37 |
| KMIPKIT-ELEM-ENUMERATION-PKCS-11-FUNCTION | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.38 |
| KMIPKIT-ELEM-ENUMERATION-PKCS-11-RETURN-CODE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.39 |
| KMIPKIT-ELEM-ENUMERATION-PROCESSING-STAGE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.40 |
| KMIPKIT-ELEM-ENUMERATION-PROFILE-NAME | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.41 |
| KMIPKIT-ELEM-ENUMERATION-PROTECTION-LEVEL | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.42 |
| KMIPKIT-ELEM-ENUMERATION-PUT-FUNCTION | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.43 |
| KMIPKIT-ELEM-ENUMERATION-QUERY-FUNCTION | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.44 |
| KMIPKIT-ELEM-ENUMERATION-RECOMMENDED-CURVE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.45 |
| KMIPKIT-ELEM-ENUMERATION-RESULT-REASON | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-ENUMERATION-RESULT-STATUS | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.47 |
| KMIPKIT-ELEM-ENUMERATION-REVOCATION-REASON-CODE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.48 |
| KMIPKIT-ELEM-ENUMERATION-RNG-ALGORITHM | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.49 |
| KMIPKIT-ELEM-ENUMERATION-RNG-MODE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.50 |
| KMIPKIT-ELEM-ENUMERATION-ROTATE-NAME-TYPE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.51 |
| KMIPKIT-ELEM-ENUMERATION-SECRET-DATA-TYPE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.52 |
| KMIPKIT-ELEM-ENUMERATION-SHREDDING-ALGORITHM | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.53 |
| KMIPKIT-ELEM-ENUMERATION-SPLIT-KEY-METHOD | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.54 |
| KMIPKIT-ELEM-ENUMERATION-STATE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.55 |
| KMIPKIT-ELEM-ENUMERATION-TAG | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-ELEM-ENUMERATION-TICKET-TYPE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.57 |
| KMIPKIT-ELEM-ENUMERATION-UNIQUE-IDENTIFIER | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.58 |
| KMIPKIT-ELEM-ENUMERATION-UNWRAP-MODE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.59 |
| KMIPKIT-ELEM-ENUMERATION-USAGE-LIMITS-UNIT | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.60 |
| KMIPKIT-ELEM-ENUMERATION-VALIDATION-AUTHORITY-TYPE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.63 |
| KMIPKIT-ELEM-ENUMERATION-VALIDATION-TYPE | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.64 |
| KMIPKIT-ELEM-ENUMERATION-VALIDITY-INDICATOR | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.61 |
| KMIPKIT-ELEM-ENUMERATION-WRAPPING-METHOD | enumeration | both | client\_1\_0 | KMIPKIT-SRC-spec §11.62 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-1-BATCH-ITEM | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §8.1 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-1-REQUEST-HEADER | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §8.1 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-1-REQUEST-MESSAGE | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §8.1 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-2-ASYNCHRONOUS-INDICATOR | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §8.2 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-2-ATTESTATION-CAPABLE-INDICATOR | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §8.2 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-2-ATTESTATION-TYPE | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §8.2 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-2-AUTHENTICATION | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §8.2 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-2-BATCH-COUNT | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §8.2 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-2-BATCH-ERROR-CONTINUATION-OPTION | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §8.2 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-2-BATCH-ORDER-OPTION | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §8.2 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-2-CLIENT-CORRELATION-VALUE | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §8.2 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-2-MAXIMUM-RESPONSE-SIZE | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §8.2 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-2-PROTOCOL-VERSION | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §8.2 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-2-REQUEST-HEADER | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §8.2 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-2-SERVER-CORRELATION-VALUE | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §8.2 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-2-TIME-STAMP | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §8.2 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-3-BATCH-ITEM | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §8.3 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-3-EPHEMERAL | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §8.3 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-3-MESSAGE-EXTENSION | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §8.3 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-3-OPERATION | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §8.3 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-3-REQUEST-PAYLOAD | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §8.3 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-3-UNIQUE-BATCH-ITEM-ID | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §8.3 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-4-BATCH-ITEM | message\_field | server\_to\_client | client\_1\_0 | KMIPKIT-SRC-spec §8.4 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-4-RESPONSE-HEADER | message\_field | server\_to\_client | client\_1\_0 | KMIPKIT-SRC-spec §8.4 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-4-RESPONSE-MESSAGE | message\_field | server\_to\_client | client\_1\_0 | KMIPKIT-SRC-spec §8.4 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-5-ATTESTATION-TYPE | message\_field | server\_to\_client | client\_1\_0 | KMIPKIT-SRC-spec §8.5 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-5-BATCH-COUNT | message\_field | server\_to\_client | client\_1\_0 | KMIPKIT-SRC-spec §8.5 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-5-CLIENT-CORRELATION-VALUE | message\_field | server\_to\_client | client\_1\_0 | KMIPKIT-SRC-spec §8.5 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-5-NONCE | message\_field | server\_to\_client | client\_1\_0 | KMIPKIT-SRC-spec §8.5 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-5-PROTOCOL-VERSION | message\_field | server\_to\_client | client\_1\_0 | KMIPKIT-SRC-spec §8.5 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-5-RESPONSE-HEADER | message\_field | server\_to\_client | client\_1\_0 | KMIPKIT-SRC-spec §8.5 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-5-SERVER-CORRELATION-VALUE | message\_field | server\_to\_client | client\_1\_0 | KMIPKIT-SRC-spec §8.5 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-5-SERVER-HASHED-PASSWORD | message\_field | server\_to\_client | client\_1\_0 | KMIPKIT-SRC-spec §8.5 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-5-TIME-STAMP | message\_field | server\_to\_client | client\_1\_0 | KMIPKIT-SRC-spec §8.5 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-6-ASYNCHRONOUS-CORRELATION-VALUE | message\_field | server\_to\_client | client\_1\_0 | KMIPKIT-SRC-spec §8.6 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-6-BATCH-ITEM | message\_field | server\_to\_client | client\_1\_0 | KMIPKIT-SRC-spec §8.6 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-6-MESSAGE-EXTENSION | message\_field | server\_to\_client | client\_1\_0 | KMIPKIT-SRC-spec §8.6 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-6-OPERATION | message\_field | server\_to\_client | client\_1\_0 | KMIPKIT-SRC-spec §8.6 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-6-RESPONSE-PAYLOAD | message\_field | server\_to\_client | client\_1\_0 | KMIPKIT-SRC-spec §8.6 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-6-RESULT-MESSAGE | message\_field | server\_to\_client | client\_1\_0 | KMIPKIT-SRC-spec §8.6 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-6-RESULT-REASON | message\_field | server\_to\_client | client\_1\_0 | KMIPKIT-SRC-spec §8.6 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-6-RESULT-STATUS | message\_field | server\_to\_client | client\_1\_0 | KMIPKIT-SRC-spec §8.6 |
| KMIPKIT-ELEM-MESSAGE-FIELD-8-6-UNIQUE-BATCH-ITEM-ID | message\_field | server\_to\_client | client\_1\_0 | KMIPKIT-SRC-spec §8.6 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-1-ASYNCHRONOUS-CORRELATION-VALUE | message\_field | server\_to\_client | client\_1\_0 | KMIPKIT-SRC-spec §9.1 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-10-SERVER-CORRELATION-VALUE | message\_field | both | client\_1\_0 | KMIPKIT-SRC-spec §9.10 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-12-MAXIMUM-RESPONSE-SIZE | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.12 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-13-CRITICALITY-INDICATOR | message\_field | both | client\_1\_0 | KMIPKIT-SRC-spec §9.13 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-13-MESSAGE-EXTENSION | message\_field | both | client\_1\_0 | KMIPKIT-SRC-spec §9.13 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-13-VENDOR-EXTENSION | message\_field | both | client\_1\_0 | KMIPKIT-SRC-spec §9.13 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-13-VENDOR-IDENTIFICATION | message\_field | both | client\_1\_0 | KMIPKIT-SRC-spec §9.13 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-14-NONCE | message\_field | both | client\_1\_0 | KMIPKIT-SRC-spec §9.14 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-14-NONCE-ID | message\_field | both | client\_1\_0 | KMIPKIT-SRC-spec §9.14 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-14-NONCE-VALUE | message\_field | both | client\_1\_0 | KMIPKIT-SRC-spec §9.14 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-15-OPERATION | message\_field | both | client\_1\_0 | KMIPKIT-SRC-spec §9.15 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-16-PROTOCOL-VERSION | message\_field | both | client\_1\_0 | KMIPKIT-SRC-spec §9.16 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-16-PROTOCOL-VERSION-MAJOR | message\_field | both | client\_1\_0 | KMIPKIT-SRC-spec §9.16 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-16-PROTOCOL-VERSION-MINOR | message\_field | both | client\_1\_0 | KMIPKIT-SRC-spec §9.16 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-17-RESULT-MESSAGE | message\_field | server\_to\_client | client\_1\_0 | KMIPKIT-SRC-spec §9.17 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-18-RESULT-REASON | message\_field | server\_to\_client | client\_1\_0 | KMIPKIT-SRC-spec §9.18 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-19-RESULT-STATUS | message\_field | server\_to\_client | client\_1\_0 | KMIPKIT-SRC-spec §9.19 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-2-ASYNCHRONOUS-INDICATOR | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.2 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-20-TIME-STAMP | message\_field | both | client\_1\_0 | KMIPKIT-SRC-spec §9.20 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-21-UNIQUE-BATCH-ITEM-ID | message\_field | both | client\_1\_0 | KMIPKIT-SRC-spec §9.21 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-3-ATTESTATION-CAPABLE-INDICATOR | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.3 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-4-AUTHENTICATION | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.4 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-4-CREDENTIAL-MAY-BE-REPEATED | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.4 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-5-BATCH-COUNT | message\_field | both | client\_1\_0 | KMIPKIT-SRC-spec §9.5 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-6-BATCH-ERROR-CONTINUATION-OPTION | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.6 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-7-BATCH-ITEM | message\_field | both | client\_1\_0 | KMIPKIT-SRC-spec §9.7 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-8-BATCH-ORDER-OPTION | message\_field | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.8 |
| KMIPKIT-ELEM-MESSAGE-FIELD-9-9-CLIENT-CORRELATION-VALUE | message\_field | both | client\_1\_0 | KMIPKIT-SRC-spec §9.9 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-3-10-KEY-MATERIAL | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §3.10 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-3-11-KEY-MATERIAL | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §3.11 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-3-12-KEY-MATERIAL | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §3.12 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-3-4-KEY-MATERIAL | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §3.4 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-3-5-KEY-MATERIAL | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §3.5 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-3-6-KEY-MATERIAL | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §3.6 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-3-7-KEY-MATERIAL | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §3.7 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-3-8-KEY-MATERIAL | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §3.8 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-3-9-KEY-MATERIAL | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §3.9 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-CERTIFICATE | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §2.1 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-CERTIFICATE-REQUEST | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §2.2 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-ENCRYPTION-KEY-INFORMATION | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-KEY-BLOCK | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-KEY-VALUE | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §3.2 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-KEY-WRAPPING-DATA | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-ELEM-OBJECT-STRUCTURE-MAC-SIGNATURE-KEY-INFORMATION | object\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §3.3 |
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
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-1-ASYNCHRONOUS-CORRELATION-VALUES | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.1 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-10-DATA-LENGTH | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.10 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-11-DEFAULTS-INFORMATION | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.11 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-12-DERIVATION-PARAMETERS | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.12 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-13-EXTENSION-INFORMATION | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.13 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-14-FINAL-INDICATOR | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.14 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-15-INTEROP-FUNCTION | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.15 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-16-INTEROP-IDENTIFIER | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.16 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-17-INIT-INDICATOR | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.17 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-18-KEY-WRAPPING-SPECIFICATION | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.18 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-19-LOG-MESSAGE | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.19 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-2-ASYNCHRONOUS-REQUEST | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.2 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-20-MAC-DATA | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.20 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-21-OBJECTS | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.21 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-22-OBJECT-DEFAULTS | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.22 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-23-OBJECT-GROUPS | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.23 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-24-OBJECT-TYPES | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.24 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-25-OPERATIONS | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.25 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-26-PKCS-11-FUNCTION | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.26 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-27-PKCS-11-INPUT-PARAMETERS | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.27 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-28-PKCS-11-INTERFACE | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.28 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-29-PKCS-11-OUTPUT-PARAMETERS | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.29 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-3-AUTHENTICATED-ENCRYPTION-ADDITIONAL-DATA | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.3 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-30-PKCS-11-RETURN-CODE | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.30 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-31-PROFILE-INFORMATION | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.31 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-32-PROFILE-VERSION | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.32 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-33-PROTECTION-STORAGE-MASKS | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.33 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-34-RIGHT | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.34 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-35-RIGHTS | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.35 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-36-RNG-PARAMETERS | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.36 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-37-SERVER-INFORMATION | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.37 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-38-SIGNATURE-DATA | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.38 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-39-TICKET | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.39 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-4-AUTHENTICATED-ENCRYPTION-TAG | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.4 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-40-USAGE-LIMITS | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.40 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-41-VALIDATION-INFORMATION | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.41 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-5-CAPABILITY-INFORMATION | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.5 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-6-CONSTRAINT | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.6 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-7-CONSTRAINTS | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.7 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-8-CORRELATION-VALUE | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.8 |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-9-DATA | operation\_structure | both | client\_1\_0 | KMIPKIT-SRC-spec §7.9 |
| KMIPKIT-ELEM-OPTION-ASYNCHRONOUS-INDICATOR | option | both | client\_1\_0 | KMIPKIT-SRC-spec §9.2, KMIPKIT-SRC-spec §11.3 |
| KMIPKIT-ELEM-OPTION-BATCH-ERROR-CONTINUATION-OPTION | option | both | client\_1\_0 | KMIPKIT-SRC-spec §9.6, KMIPKIT-SRC-spec §11.5 |
| KMIPKIT-ELEM-OPTION-BATCH-ORDER-OPTION | option | both | client\_1\_0 | KMIPKIT-SRC-spec §9.8 |
| KMIPKIT-ELEM-RESULT-CANCELLATION-RESULT | result | both | client\_1\_0 | KMIPKIT-SRC-spec §11.7 |
| KMIPKIT-ELEM-RESULT-RESULT-MESSAGE | result | both | client\_1\_0 | KMIPKIT-SRC-spec §9.17 |
| KMIPKIT-ELEM-RESULT-RESULT-REASON | result | both | client\_1\_0 | KMIPKIT-SRC-spec §9.18, KMIPKIT-SRC-spec §11.46 |
| KMIPKIT-ELEM-RESULT-RESULT-STATUS | result | both | client\_1\_0 | KMIPKIT-SRC-spec §9.19, KMIPKIT-SRC-spec §11.47 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-1-CERTIFICATE-CERTIFICATE-TYPE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §2.1 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-1-CERTIFICATE-CERTIFICATE-VALUE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §2.1 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-2-CERTIFICATE-REQUEST-CERTIFICATE-REQUEST-TYPE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §2.2 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-2-CERTIFICATE-REQUEST-CERTIFICATE-REQUEST-VALUE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §2.2 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-3-OPAQUE-OBJECT-OPAQUE-DATA-TYPE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §2.3 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-3-OPAQUE-OBJECT-OPAQUE-DATA-VALUE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §2.3 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-4-PGP-KEY-KEY-BLOCK | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §2.4 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-4-PGP-KEY-PGP-KEY-VERSION | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §2.4 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-5-PRIVATE-KEY-KEY-BLOCK | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §2.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-6-PUBLIC-KEY-KEY-BLOCK | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §2.6 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-7-SECRET-DATA-KEY-BLOCK | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §2.7 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-7-SECRET-DATA-SECRET-DATA-TYPE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §2.7 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-8-SPLIT-KEY-KEY-BLOCK | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §2.8 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-8-SPLIT-KEY-KEY-PART-IDENTIFIER | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §2.8 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-8-SPLIT-KEY-PRIME-FIELD-SIZE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §2.8 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-8-SPLIT-KEY-SPLIT-KEY-METHOD | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §2.8 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-8-SPLIT-KEY-SPLIT-KEY-PARTS | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §2.8 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-8-SPLIT-KEY-SPLIT-KEY-THRESHOLD | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §2.8 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-2-9-SYMMETRIC-KEY-KEY-BLOCK | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §2.9 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-1-KEY-BLOCK-CRYPTOGRAPHIC-ALGORITHM | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-1-KEY-BLOCK-CRYPTOGRAPHIC-LENGTH | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-1-KEY-BLOCK-KEY-COMPRESSION-TYPE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-1-KEY-BLOCK-KEY-FORMAT-TYPE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-1-KEY-BLOCK-KEY-VALUE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-1-KEY-BLOCK-KEY-WRAPPING-DATA | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.1 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-10-KEY-MATERIAL-G | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.10 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-10-KEY-MATERIAL-J | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.10 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-10-KEY-MATERIAL-P | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.10 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-10-KEY-MATERIAL-Q | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.10 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-10-KEY-MATERIAL-Y | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.10 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-11-KEY-MATERIAL-D | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-11-KEY-MATERIAL-RECOMMENDED-CURVE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-12-KEY-MATERIAL-Q-STRING | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.12 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-12-KEY-MATERIAL-RECOMMENDED-CURVE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.12 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-2-KEY-VALUE-ATTRIBUTES | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.2 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-2-KEY-VALUE-KEY-MATERIAL | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.2 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-3-ENCRYPTION-KEY-INFORMATION-CRYPTOGRAPHIC-PARAMETERS | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-3-ENCRYPTION-KEY-INFORMATION-UNIQUE-IDENTIFIER | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-3-KEY-WRAPPING-DATA-ENCODING-OPTION | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-3-KEY-WRAPPING-DATA-ENCRYPTION-KEY-INFORMATION | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-3-KEY-WRAPPING-DATA-IV-COUNTER-NONCE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-3-KEY-WRAPPING-DATA-MAC-SIGNATURE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-3-KEY-WRAPPING-DATA-MAC-SIGNATURE-KEY-INFORMATION | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-3-KEY-WRAPPING-DATA-WRAPPING-METHOD | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-3-MAC-SIGNATURE-KEY-INFORMATION-CRYPTOGRAPHIC-PARAMETERS | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-3-MAC-SIGNATURE-KEY-INFORMATION-UNIQUE-IDENTIFIER | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.3 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-4-KEY-MATERIAL-KEY | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.4 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-5-KEY-MATERIAL-G | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-5-KEY-MATERIAL-P | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-5-KEY-MATERIAL-Q | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-5-KEY-MATERIAL-X | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-6-KEY-MATERIAL-G | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.6 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-6-KEY-MATERIAL-P | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.6 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-6-KEY-MATERIAL-Q | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.6 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-6-KEY-MATERIAL-Y | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.6 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-7-KEY-MATERIAL-CRT-COEFFICIENT | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.7 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-7-KEY-MATERIAL-MODULUS | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.7 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-7-KEY-MATERIAL-P | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.7 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-7-KEY-MATERIAL-PRIME-EXPONENT-P | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.7 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-7-KEY-MATERIAL-PRIME-EXPONENT-Q | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.7 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-7-KEY-MATERIAL-PRIVATE-EXPONENT | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.7 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-7-KEY-MATERIAL-PUBLIC-EXPONENT | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.7 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-7-KEY-MATERIAL-Q | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.7 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-8-KEY-MATERIAL-MODULUS | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.8 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-8-KEY-MATERIAL-PUBLIC-EXPONENT | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.8 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-9-KEY-MATERIAL-G | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.9 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-9-KEY-MATERIAL-J | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.9 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-9-KEY-MATERIAL-P | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.9 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-9-KEY-MATERIAL-Q | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.9 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-3-9-KEY-MATERIAL-X | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §3.9 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-14-QLENGTH | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.14 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-14-RECOMMENDED-CURVE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.14 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-BLOCK-CIPHER-MODE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-COUNTER-LENGTH | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-CRYPTOGRAPHIC-ALGORITHM | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-DIGITAL-SIGNATURE-ALGORITHM | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-FIXED-FIELD-LENGTH | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-HASHING-ALGORITHM | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-INITIAL-COUNTER-VALUE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-INVOCATION-FIELD-LENGTH | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-IV-LENGTH | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-KEY-ROLE-TYPE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-MASK-GENERATOR | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-MASK-GENERATOR-HASHING-ALGORITHM | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-P-SOURCE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-PADDING-METHOD | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-RANDOM-IV | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-SALT-LENGTH | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-TAG-LENGTH | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-TRAILER-FIELD | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.16 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-2-ALTERNATIVE-NAME-TYPE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.2 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-2-ALTERNATIVE-NAME-VALUE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.2 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-21-DIGEST-VALUE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.21 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-21-HASHING-ALGORITHM | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.21 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-21-KEY-FORMAT-TYPE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.21 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-27-KEY-VALUE-LOCATION-TYPE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.27 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-27-KEY-VALUE-LOCATION-VALUE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.27 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-31-LINK-TYPE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.31 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-31-LINKED-OBJECT-IDENTIFIER | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.31 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-32-NAME-TYPE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.32 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-32-NAME-VALUE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.32 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-4-APPLICATION-DATA | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.4 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-4-APPLICATION-NAMESPACE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.4 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-47-REVOCATION-MESSAGE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.47 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-47-REVOCATION-REASON-CODE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.47 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-53-ROTATE-NAME-TYPE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.53 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-53-ROTATE-NAME-VALUE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.53 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-60-ATTRIBUTE-NAME | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.60 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-60-ATTRIBUTE-VALUE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.60 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-60-VENDOR-IDENTIFICATION | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.60 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-61-CERTIFICATE-SERIAL-NUMBER | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.61 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-61-ISSUER-DISTINGUISHED-NAME | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.61 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-62-ISSUER-ALTERNATIVE-NAME | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.62 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-62-ISSUER-DISTINGUISHED-NAME | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.62 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-63-SUBJECT-ALTERNATIVE-NAME | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.63 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-4-63-SUBJECT-DISTINGUISHED-NAME | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §4.63 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-5-1-ANY-ATTRIBUTE-IN-4-OBJECT-ATTRIBUTES | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §5.1 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-5-2-ANY-ATTRIBUTE-IN-4-OBJECT-ATTRIBUTES | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §5.2 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-5-3-ANY-ATTRIBUTE-IN-4-OBJECT-ATTRIBUTES | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §5.3 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-5-4-ANY-ATTRIBUTE-IN-4-OBJECT-ATTRIBUTES | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §5.4 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-5-5-ATTRIBUTE-NAME | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §5.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-5-5-ATTRIBUTE-REFERENCE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §5.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-5-5-VENDOR-IDENTIFICATION | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §5.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-5-6-ANY-ATTRIBUTE-IN-4-OBJECT-ATTRIBUTES | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §5.6 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-5-7-ANY-ATTRIBUTE-IN-4-OBJECT-ATTRIBUTES | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §5.7 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-1-ASYNCHRONOUS-CORRELATION-VALUE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.1 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-11-OBJECT-DEFAULTS | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-12-CRYPTOGRAPHIC-PARAMETERS | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.12 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-12-DERIVATION-DATA | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.12 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-12-INITIALIZATION-VECTOR | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.12 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-12-ITERATION-COUNT | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.12 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-12-SALT | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.12 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-13-EXTENSION-ATTRIBUTE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.13 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-13-EXTENSION-DESCRIPTION | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.13 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-13-EXTENSION-ENUMERATION | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.13 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-13-EXTENSION-NAME | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.13 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-13-EXTENSION-PARENT-STRUCTURE-TAG | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.13 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-13-EXTENSION-TAG | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.13 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-13-EXTENSION-TYPE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.13 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-18-ATTRIBUTE-NAME | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.18 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-18-ENCODING-OPTION | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.18 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-18-ENCRYPTION-KEY-INFORMATION | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.18 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-18-MAC-SIGNATURE-KEY-INFORMATION | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.18 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-18-WRAPPING-METHOD | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.18 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-2-ASYNCHRONOUS-CORRELATION-VALUE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.2 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-2-OPERATION | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.2 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-2-PROCESSING-STAGE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.2 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-2-SUBMISSION-DATE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.2 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-21-UNIQUE-IDENTIFIER | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.21 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-22-ATTRIBUTES | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.22 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-22-OBJECT-GROUPS | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.22 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-22-OBJECT-TYPE-OBJECTTYPES | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.22 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-23-OBJECT-GROUP | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.23 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-24-OBJECT-TYPE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.24 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-25-OPERATION | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.25 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-26-PKCS-11-FUNCTION | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.26 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-27-PKCS-11-INPUT-PARAMETERS | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.27 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-28-PKCS-11-INTERFACE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.28 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-29-PKCS-11-OUTPUT-PARAMETERS | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.29 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-30-PKCS-11-RETURN-CODE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.30 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-31-PROFILE-NAME | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.31 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-31-PROFILE-VERSION | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.31 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-31-SERVER-PORT | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.31 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-31-SERVER-URI | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.31 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-32-PROFILE-VERSION-MAJOR | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.32 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-32-PROFILE-VERSION-MINOR | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.32 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-34-OBJECT-GROUPS | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.34 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-34-OBJECTS | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.34 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-34-OPERATIONS | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.34 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-34-USAGE-LIMITS | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.34 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-35-RIGHT | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.35 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-36-CRYPTOGRAPHIC-ALGORITHM | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.36 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-36-CRYPTOGRAPHIC-LENGTH | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.36 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-36-DRBG-ALGORITHM | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.36 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-36-FIPS186-VARIATION | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.36 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-36-HASHING-ALGORITHM | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.36 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-36-PREDICTION-RESISTANCE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.36 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-36-RECOMMENDED-CURVE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.36 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-36-RNG-ALGORITHM | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.36 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-37-ALTERNATIVE-FAILOVER-ENDPOINTS | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.37 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-37-BUILD-DATE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.37 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-37-BUILD-LEVEL | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.37 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-37-CLUSTER-INFO | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.37 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-37-PRODUCT-NAME | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.37 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-37-SERVER-LOAD | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.37 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-37-SERVER-NAME | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.37 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-37-SERVER-SERIAL-NUMBER | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.37 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-37-SERVER-VERSION | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.37 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-37-VENDOR-SPECIFIC | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.37 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-39-TICKET-TYPE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.39 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-39-TICKET-VALUE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.39 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-40-USAGE-LIMITS-COUNT | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.40 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-40-USAGE-LIMITS-TOTAL | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.40 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-40-USAGE-LIMITS-UNIT | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.40 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-41-VALIDATION-AUTHORITY-COUNTRY | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.41 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-41-VALIDATION-AUTHORITY-TYPE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.41 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-41-VALIDATION-AUTHORITY-URI | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.41 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-41-VALIDATION-CERTIFICATE-IDENTIFIER | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.41 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-41-VALIDATION-CERTIFICATE-URI | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.41 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-41-VALIDATION-LEVEL | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.41 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-41-VALIDATION-PROFILE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.41 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-41-VALIDATION-TYPE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.41 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-41-VALIDATION-VENDOR-URI | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.41 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-41-VALIDATION-VERSION-MAJOR | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.41 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-41-VALIDATION-VERSION-MINOR | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.41 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-5-ASYNCHRONOUS-CAPABILITY | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-5-ATTESTATION-CAPABILITY | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-5-BATCH-CONTINUE-CAPABILITY | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-5-BATCH-UNDO-CAPABILITY | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-5-DESTROY-ACTION | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-5-QUANTUM-SAFE-CAPABILITY | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-5-RNG-MODE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-5-SHREDDING-ALGORITHM | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-5-STREAMING-CAPABILITY | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-5-UNWRAP-MODE | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.5 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-6-ATTRIBUTES | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.6 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-6-OBJECT-GROUPS | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.6 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-6-OBJECT-TYPES | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.6 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-7-7-CONSTRAINT | structure\_member | both | client\_1\_0 | KMIPKIT-SRC-spec §7.7 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ATTESTATION-ATTESTATION-ASSERTION | structure\_member | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ATTESTATION-ATTESTATION-MEASUREMENT | structure\_member | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ATTESTATION-ATTESTATION-TYPE | structure\_member | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ATTESTATION-NONCE | structure\_member | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-CREDENTIAL-CREDENTIAL-TYPE | structure\_member | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-CREDENTIAL-CREDENTIAL-VALUE | structure\_member | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-DEVICE-DEVICE-IDENTIFIER | structure\_member | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-DEVICE-DEVICE-SERIAL-NUMBER | structure\_member | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-DEVICE-MACHINE-IDENTIFIER | structure\_member | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-DEVICE-MEDIA-IDENTIFIER | structure\_member | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-DEVICE-NETWORK-IDENTIFIER | structure\_member | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-DEVICE-PASSWORD | structure\_member | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-HASHED-PASSWORD-HASHED-PASSWORD | structure\_member | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-HASHED-PASSWORD-HASHING-ALGORITHM | structure\_member | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-HASHED-PASSWORD-TIMESTAMP | structure\_member | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-HASHED-PASSWORD-USERNAME | structure\_member | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ONE-TIME-PASSWORD-ONE-TIME-PASSWORD | structure\_member | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ONE-TIME-PASSWORD-PASSWORD | structure\_member | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ONE-TIME-PASSWORD-USERNAME | structure\_member | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-TICKET-TICKET | structure\_member | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-USERNAME-AND-PASSWORD-PASSWORD | structure\_member | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
| KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-USERNAME-AND-PASSWORD-USERNAME | structure\_member | client\_to\_server | client\_1\_0 | KMIPKIT-SRC-spec §9.11 |
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
| KMIPKIT-PROFILE-AES-XTS-CLIENT | AES XTS Client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.13.1, KMIPKIT-SRC-profiles §5.13.3, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.30 |
| KMIPKIT-PROFILE-AES-XTS-SERVER | AES XTS Server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.13.2, KMIPKIT-SRC-profiles §5.13.3, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.31 |
| KMIPKIT-PROFILE-ASYMMETRIC-KEY-LIFECYCLE-CLIENT | Asymmetric Key Lifecycle Client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.8.1, KMIPKIT-SRC-profiles §5.8.3, KMIPKIT-SRC-profiles §5.8.4, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.16 |
| KMIPKIT-PROFILE-ASYMMETRIC-KEY-LIFECYCLE-SERVER | Asymmetric Key Lifecycle Server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.8.2, KMIPKIT-SRC-profiles §5.8.3, KMIPKIT-SRC-profiles §5.8.4, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.17 |
| KMIPKIT-PROFILE-BASELINE-CLIENT | Baseline Client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.1.3, KMIPKIT-SRC-profiles §5.6.3, KMIPKIT-SRC-profiles §6.1 |
| KMIPKIT-PROFILE-BASELINE-SERVER | Baseline Server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.1.3, KMIPKIT-SRC-profiles §5.6.3, KMIPKIT-SRC-profiles §6.2 |
| KMIPKIT-PROFILE-COMPLETE-SERVER | Complete Server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.2, KMIPKIT-SRC-profiles §5.3.2, KMIPKIT-SRC-profiles §5.4.3, KMIPKIT-SRC-profiles §5.5.3, KMIPKIT-SRC-profiles §5.6.2, KMIPKIT-SRC-profiles §5.7.4, KMIPKIT-SRC-profiles §5.8.2, KMIPKIT-SRC-profiles §5.9.4, KMIPKIT-SRC-profiles §5.9.5, KMIPKIT-SRC-profiles §5.9.6, KMIPKIT-SRC-profiles §5.10.2, KMIPKIT-SRC-profiles §5.11.2, KMIPKIT-SRC-profiles §5.12.5, KMIPKIT-SRC-profiles §5.13.2, KMIPKIT-SRC-profiles §5.16, KMIPKIT-SRC-profiles §5.18.4, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.3, KMIPKIT-SRC-profiles §6.5, KMIPKIT-SRC-profiles §6.7, KMIPKIT-SRC-profiles §6.9, KMIPKIT-SRC-profiles §6.11, KMIPKIT-SRC-profiles §6.15, KMIPKIT-SRC-profiles §6.17, KMIPKIT-SRC-profiles §6.21, KMIPKIT-SRC-profiles §6.22, KMIPKIT-SRC-profiles §6.23, KMIPKIT-SRC-profiles §6.25, KMIPKIT-SRC-profiles §6.27, KMIPKIT-SRC-profiles §6.29, KMIPKIT-SRC-profiles §6.31, KMIPKIT-SRC-profiles §6.33, KMIPKIT-SRC-profiles §6.35 |
| KMIPKIT-PROFILE-CRYPTOGRAPHIC-ADVANCED-CLIENT | Advanced Cryptographic Client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.9.2, KMIPKIT-SRC-profiles §5.9.8, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.19 |
| KMIPKIT-PROFILE-CRYPTOGRAPHIC-ADVANCED-SERVER | Advanced Cryptographic Server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.9.5, KMIPKIT-SRC-profiles §5.9.8, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.22 |
| KMIPKIT-PROFILE-CRYPTOGRAPHIC-BASIC-CLIENT | Basic Cryptographic Client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.9.1, KMIPKIT-SRC-profiles §5.9.7, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.18 |
| KMIPKIT-PROFILE-CRYPTOGRAPHIC-BASIC-SERVER | Basic Cryptographic Server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.9.4, KMIPKIT-SRC-profiles §5.9.7, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.21 |
| KMIPKIT-PROFILE-CRYPTOGRAPHIC-RNG-CLIENT | RNG Cryptographic Client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.9.3, KMIPKIT-SRC-profiles §5.9.9, KMIPKIT-SRC-profiles §5.9.10, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.20 |
| KMIPKIT-PROFILE-CRYPTOGRAPHIC-RNG-SERVER | RNG Cryptographic Server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.9.6, KMIPKIT-SRC-profiles §5.9.9, KMIPKIT-SRC-profiles §5.9.10, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.23 |
| KMIPKIT-PROFILE-HTTPS-CLIENT | HTTPS Client | conditional | not\_claimed | KMIPKIT-SRC-profiles §3.2, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.3.1, KMIPKIT-SRC-profiles §5.3.3, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.4 |
| KMIPKIT-PROFILE-HTTPS-SERVER | HTTPS Server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.2, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.3.2, KMIPKIT-SRC-profiles §5.3.3, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.5 |
| KMIPKIT-PROFILE-JSON-CLIENT | JSON Client | out\_of\_scope | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.5.2, KMIPKIT-SRC-profiles §5.5.4, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.8 |
| KMIPKIT-PROFILE-JSON-SERVER | JSON Server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.5.3, KMIPKIT-SRC-profiles §5.5.4, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.9 |
| KMIPKIT-PROFILE-OPAQUE-MANAGED-OBJECT-STORE-CLIENT | Opaque Managed Object Store Client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.10.1, KMIPKIT-SRC-profiles §5.10.3, KMIPKIT-SRC-profiles §5.10.4, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.24 |
| KMIPKIT-PROFILE-OPAQUE-MANAGED-OBJECT-STORE-SERVER | Opaque Managed Object Store Server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.10.2, KMIPKIT-SRC-profiles §5.10.3, KMIPKIT-SRC-profiles §5.10.4, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.25 |
| KMIPKIT-PROFILE-PKCS11-CLIENT | PKCS#11 Client | out\_of\_scope | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.18.3, KMIPKIT-SRC-profiles §5.18.5, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.34 |
| KMIPKIT-PROFILE-PKCS11-SERVER | PKCS#11 Server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.18.4, KMIPKIT-SRC-profiles §5.18.5, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.35 |
| KMIPKIT-PROFILE-QUANTUM-SAFE-CLIENT | Quantum Safe Client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.15, KMIPKIT-SRC-profiles §5.17, KMIPKIT-SRC-profiles §5.17.1, KMIPKIT-SRC-profiles §5.17.2, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.32 |
| KMIPKIT-PROFILE-QUANTUM-SAFE-SERVER | Quantum Safe Server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.16, KMIPKIT-SRC-profiles §5.17.1, KMIPKIT-SRC-profiles §5.17.2, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.33 |
| KMIPKIT-PROFILE-STORAGE-ARRAY-SELF-ENCRYPTING-DRIVES-CLIENT | Storage Array with Self-Encrypting Drives Client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.11.1, KMIPKIT-SRC-profiles §5.11.3, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.26 |
| KMIPKIT-PROFILE-STORAGE-ARRAY-SELF-ENCRYPTING-DRIVES-SERVER | Storage Array with Self-Encrypting Drives Server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.11.2, KMIPKIT-SRC-profiles §5.11.3, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.27 |
| KMIPKIT-PROFILE-SYMMETRIC-KEY-FOUNDRY-ADVANCED-CLIENT | Advanced Symmetric Key Foundry Client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.7.3, KMIPKIT-SRC-profiles §5.7.7, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.14 |
| KMIPKIT-PROFILE-SYMMETRIC-KEY-FOUNDRY-BASIC-CLIENT | Basic Symmetric Key Foundry Client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.7.1, KMIPKIT-SRC-profiles §5.7.5, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.12 |
| KMIPKIT-PROFILE-SYMMETRIC-KEY-FOUNDRY-INTERMEDIATE-CLIENT | Intermediate Symmetric Key Foundry Client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.7.2, KMIPKIT-SRC-profiles §5.7.6, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.13 |
| KMIPKIT-PROFILE-SYMMETRIC-KEY-FOUNDRY-SERVER | Symmetric Key Foundry Server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.7.4, KMIPKIT-SRC-profiles §5.7.5, KMIPKIT-SRC-profiles §5.7.6, KMIPKIT-SRC-profiles §5.7.7, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.15 |
| KMIPKIT-PROFILE-SYMMETRIC-KEY-LIFECYCLE-CLIENT | Symmetric Key Lifecycle Client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.6.1, KMIPKIT-SRC-profiles §5.6.3, KMIPKIT-SRC-profiles §5.6.4, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.10 |
| KMIPKIT-PROFILE-SYMMETRIC-KEY-LIFECYCLE-SERVER | Symmetric Key Lifecycle Server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.6.2, KMIPKIT-SRC-profiles §5.6.3, KMIPKIT-SRC-profiles §5.6.4, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.11 |
| KMIPKIT-PROFILE-TAPE-LIBRARY-CLIENT | Tape Library Client | client\_1\_0 | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.12.2, KMIPKIT-SRC-profiles §5.12.3, KMIPKIT-SRC-profiles §5.12.4, KMIPKIT-SRC-profiles §5.12.6, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.28 |
| KMIPKIT-PROFILE-TAPE-LIBRARY-SERVER | Tape Library Server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.12.5, KMIPKIT-SRC-profiles §5.12.6, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.29 |
| KMIPKIT-PROFILE-XML-CLIENT | XML Client | out\_of\_scope | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.1, KMIPKIT-SRC-profiles §5.4.2, KMIPKIT-SRC-profiles §5.4.4, KMIPKIT-SRC-profiles §6.1, KMIPKIT-SRC-profiles §6.6 |
| KMIPKIT-PROFILE-XML-SERVER | XML Server | server\_only | not\_claimed | KMIPKIT-SRC-profiles §3.1, KMIPKIT-SRC-profiles §5.1.2, KMIPKIT-SRC-profiles §5.4.3, KMIPKIT-SRC-profiles §5.4.4, KMIPKIT-SRC-profiles §6.2, KMIPKIT-SRC-profiles §6.7 |

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
| KMIPKIT-TEST-CN01-2-12 | TC-CREATE-SD-1-21 | unavailable | — |
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
| unavailable | 203 |

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
| KMIPKIT-DISC-037 | open | blocked for affected records | 1 requirements, 436 elements, 1 policies | Reserved-tag receipt and lossless preservation behavior is not specified | KMIPKIT-SRC-spec §11.56 |
| KMIPKIT-DISC-038 | open | blocked for affected records | 3 elements, 2 profiles | JSON profile example uses Template for a reserved Object Type value | KMIPKIT-SRC-profiles §5.5.4.1, KMIPKIT-SRC-spec §11.34 |
| KMIPKIT-DISC-039 | open | blocked for affected records | 1 elements | Query Asynchronous Requests response table is labeled as a PKCS#11 response | KMIPKIT-SRC-spec §6.1.41 |

## Project policies

| Policy | Provenance | Summary |
| --- | --- | --- |
| KMIPKIT-POLICY-EXTENSION-PRESERVATION | AGENTS.md | Preserve unknown KMIP extension data losslessly and expose it without interpreting it as a standardized value. |
| KMIPKIT-POLICY-UNKNOWN-FUTURE-VALUE-PRESERVATION | AGENTS.md | Preserve unknown or future tags, enumeration values, and bitmask bits without assigning them standardized KMIP semantics. |
| KMIPKIT-POLICY-VENDOR-VALUE-PRESERVATION | AGENTS.md | Preserve vendor-defined values distinctly from OASIS-assigned, reserved, unused, and unknown-to-KMIPKit values. |
