# KMIP 2.1 Client Operation Dependency Matrix

This matrix inventories the 56 catalog elements with `scope_state: client_1_0`. KMIP 2.1 defines 57 client-initiated operation types in §§6.1.1–6.1.57; §6.1.54 Set Endpoint Role is explicitly deferred to `client_1_1` by the catalog because its role transition requires bidirectional channel behavior outside the 1.0 boundary. Operation sections refer to **Key Management Interoperability Protocol Specification Version 2.1**, OASIS Standard, 14 December 2020, §§6.1.1–6.1.57. A linked owner/candidate reference does not imply that API bindings are implemented or verified.

The owner states below were revalidated from active `release/1.0.0` at `7a430d5f` on 2026-10-12. Approved/accepted is derived only from the linked spec's checked-in status; implementation authorization for a draft does not count as approval. Draft worktrees are not sources for owner links.

| OASIS section | Operation | Linked owner specification | Current status / API gate |
|---|---|---|---|
| §6.1.1 | Activate | KMIPKIT-0018 | Candidate owner (draft, not formally approved); blocked until approval and four-language parity evidence |
| §6.1.2 | Add Attribute | KMIPKIT-0016 | Candidate owner (draft, not formally approved); blocked until approval and four-language parity evidence |
| §6.1.3 | Adjust Attribute | KMIPKIT-0016 | Candidate owner (draft, not formally approved); blocked until approval and four-language parity evidence |
| §6.1.4 | Archive | KMIPKIT-0018 | Candidate owner (draft, not formally approved); blocked until approval and four-language parity evidence |
| §6.1.5 | Cancel | KMIPKIT-0009 | Candidate owner (draft, not formally approved); blocked until approval and four-language parity evidence; `KMIPKIT-DISC-039` remains a typed asynchronous response gate |
| §6.1.6 | Certify | KMIPKIT-0036 | Approved owner; KMIPKIT-0036 approval recorded 2026-10-11; binding remains blocked until four-language parity evidence exists |
| §6.1.7 | Check | KMIPKIT-0036 | Approved owner; KMIPKIT-0036 approval recorded 2026-10-11; binding remains blocked until four-language parity evidence exists |
| §6.1.8 | Create | KMIPKIT-0014 | Approved/accepted owner; binding remains blocked until Rust and four-language parity evidence exist |
| §6.1.9 | Create Key Pair | KMIPKIT-0014 | Approved/accepted owner; binding remains blocked until Rust and four-language parity evidence exist |
| §6.1.10 | Create Split Key | KMIPKIT-0014 | Approved/accepted owner; binding remains blocked until Rust and four-language parity evidence exist |
| §6.1.11 | Decrypt | KMIPKIT-0019 | Candidate owner (autonomous implementation authorized, specification not formally approved); blocked until approval and four-language parity evidence; `KMIPKIT-DISC-045` and `KMIPKIT-DISC-046` remain tracked |
| §6.1.12 | Delegated Login | KMIPKIT-0038 | Approved owner; KMIPKIT-0038 approval recorded 2026-10-11; binding remains blocked until four-language parity evidence exists |
| §6.1.13 | Delete Attribute | KMIPKIT-0016 | Candidate owner (draft, not formally approved); blocked until approval and four-language parity evidence |
| §6.1.14 | Derive Key | KMIPKIT-0037 | Approved owner; KMIPKIT-0037 approval recorded 2026-10-11; binding remains blocked until four-language parity evidence exists |
| §6.1.15 | Destroy | KMIPKIT-0018 | Candidate owner (draft, not formally approved); blocked until approval and four-language parity evidence |
| §6.1.16 | Discover Versions | KMIPKIT-0007 | Approved/accepted owner; binding remains blocked until Rust and four-language parity evidence exist |
| §6.1.17 | Encrypt | KMIPKIT-0019 | Candidate owner (autonomous implementation authorized, specification not formally approved); blocked until approval and four-language parity evidence; `KMIPKIT-DISC-045` and `KMIPKIT-DISC-046` remain tracked |
| §6.1.18 | Export | KMIPKIT-0037 | Approved owner; KMIPKIT-0037 approval recorded 2026-10-11; binding remains blocked until four-language parity evidence exists |
| §6.1.19 | Get | KMIPKIT-0017 | Approved/accepted owner; binding remains blocked until Rust and four-language parity evidence exist |
| §6.1.20 | Get Attributes | KMIPKIT-0016 | Candidate owner (draft, not formally approved); blocked until approval and four-language parity evidence |
| §6.1.21 | Get Attribute List | KMIPKIT-0016 | Candidate owner (draft, not formally approved); blocked until approval and four-language parity evidence |
| §6.1.22 | Get Constraints | KMIPKIT-0039 | Approved owner; KMIPKIT-0039 approval recorded 2026-10-11; binding remains blocked until four-language parity evidence exists |
| §6.1.23 | Get Usage Allocation | KMIPKIT-0039 | Approved owner; KMIPKIT-0039 approval recorded 2026-10-11; binding remains blocked until four-language parity evidence exists |
| §6.1.24 | Hash | KMIPKIT-0021 | Approved/accepted owner; binding remains blocked until Rust and four-language parity evidence exist |
| §6.1.25 | Import | KMIPKIT-0037 | Approved owner; KMIPKIT-0037 approval recorded 2026-10-11; binding remains blocked until four-language parity evidence exists |
| §6.1.26 | Interop | KMIPKIT-0042 | Approved owner; KMIPKIT-0042 approval recorded 2026-10-11; binding remains blocked until four-language parity evidence exists |
| §6.1.27 | Join Split Key | KMIPKIT-0037 | Approved owner; KMIPKIT-0037 approval recorded 2026-10-11; binding remains blocked until four-language parity evidence exists |
| §6.1.28 | Locate | KMIPKIT-0017 | Approved/accepted owner; binding remains blocked until Rust and four-language parity evidence exist |
| §6.1.29 | Log | KMIPKIT-0042 | Approved owner; KMIPKIT-0042 approval recorded 2026-10-11; binding remains blocked until four-language parity evidence exists |
| §6.1.30 | Login | KMIPKIT-0038 | Approved owner; KMIPKIT-0038 approval recorded 2026-10-11; binding remains blocked until four-language parity evidence exists |
| §6.1.31 | Logout | KMIPKIT-0038 | Approved owner; KMIPKIT-0038 approval recorded 2026-10-11; binding remains blocked until four-language parity evidence exists |
| §6.1.32 | MAC | KMIPKIT-0021 | Approved/accepted owner; binding remains blocked until Rust and four-language parity evidence exist |
| §6.1.33 | MAC Verify | KMIPKIT-0021 | Approved/accepted owner; binding remains blocked until Rust and four-language parity evidence exist |
| §6.1.34 | Modify Attribute | KMIPKIT-0016 | Candidate owner (draft, not formally approved); blocked until approval and four-language parity evidence |
| §6.1.35 | Obtain Lease | KMIPKIT-0036 | Approved owner; KMIPKIT-0036 approval recorded 2026-10-11; binding remains blocked until four-language parity evidence exists |
| §6.1.36 | Ping | KMIPKIT-0020 | Candidate owner (autonomous implementation authorized, specification not formally approved); blocked until approval and four-language parity evidence |
| §6.1.37 | PKCS#11 | KMIPKIT-0040 | Approved owner; KMIPKIT-0040 approval recorded 2026-10-11; binding remains blocked until four-language parity evidence exists |
| §6.1.38 | Poll | KMIPKIT-0009 | Candidate owner (draft, not formally approved); blocked until approval and four-language parity evidence; `KMIPKIT-DISC-039` remains a typed asynchronous response gate |
| §6.1.39 | Process | KMIPKIT-0009 | Candidate owner (draft, not formally approved); blocked until approval and four-language parity evidence; `KMIPKIT-DISC-039` remains a typed asynchronous response gate |
| §6.1.40 | Query | KMIPKIT-0020 | Candidate owner (autonomous implementation authorized, specification not formally approved); blocked until approval and four-language parity evidence |
| §6.1.41 | Query Asynchronous Requests | KMIPKIT-0009 | Candidate owner (draft, not formally approved); blocked until approval and four-language parity evidence; `KMIPKIT-DISC-039` remains a typed asynchronous response gate |
| §6.1.42 | Recover | KMIPKIT-0018 | Candidate owner (draft, not formally approved); blocked until approval and four-language parity evidence |
| §6.1.43 | Register | KMIPKIT-0036 | Approved owner; KMIPKIT-0036 approval recorded 2026-10-11; binding remains blocked until four-language parity evidence exists |
| §6.1.44 | Revoke | KMIPKIT-0036 | Approved owner; KMIPKIT-0036 approval recorded 2026-10-11; binding remains blocked until four-language parity evidence exists |
| §6.1.45 | Re-certify | KMIPKIT-0036 | Approved owner; KMIPKIT-0036 approval recorded 2026-10-11; binding remains blocked until four-language parity evidence exists |
| §6.1.46 | Re-key | KMIPKIT-0036 | Approved owner; KMIPKIT-0036 approval recorded 2026-10-11; binding remains blocked until four-language parity evidence exists |
| §6.1.47 | Re-key Key Pair | KMIPKIT-0036 | Approved owner; KMIPKIT-0036 approval recorded 2026-10-11; binding remains blocked until four-language parity evidence exists |
| §6.1.48 | Re-Provision | KMIPKIT-0036 | Approved owner; KMIPKIT-0036 approval recorded 2026-10-11; binding remains blocked until four-language parity evidence exists |
| §6.1.49 | RNG Retrieve | KMIPKIT-0041 | Approved owner; KMIPKIT-0041 approval recorded 2026-10-11; binding remains blocked until four-language parity evidence exists |
| §6.1.50 | RNG Seed | KMIPKIT-0041 | Approved owner; KMIPKIT-0041 approval recorded 2026-10-11; binding remains blocked until four-language parity evidence exists |
| §6.1.51 | Set Attribute | KMIPKIT-0016 | Candidate owner (draft, not formally approved); blocked until approval and four-language parity evidence |
| §6.1.52 | Set Constraints | KMIPKIT-0039 | Approved owner; KMIPKIT-0039 approval recorded 2026-10-11; binding remains blocked until four-language parity evidence exists |
| §6.1.53 | Set Defaults | KMIPKIT-0039 | Approved owner; KMIPKIT-0039 approval recorded 2026-10-11; binding remains blocked until four-language parity evidence exists |
| §6.1.55 | Sign | KMIPKIT-0021 | Approved/accepted owner; binding remains blocked until Rust and four-language parity evidence exist |
| §6.1.56 | Signature Verify | KMIPKIT-0021 | Approved/accepted owner; binding remains blocked until Rust and four-language parity evidence exist |
| §6.1.57 | Validate | KMIPKIT-0042 | Approved owner; KMIPKIT-0042 approval recorded 2026-10-11; binding remains blocked until four-language parity evidence exists |

## Reconciliation

- Total KMIP 2.1 client-initiated operation types: **57**.
- In-scope `client_1_0` operations: **56**.
- Linked to an owner specification in the active release line: **56**.
- Linked to an approved/accepted owner: **37 operations**.
- Linked to an unapproved candidate owner: **19 operations**.
- No owner specification linked: **0 operations**.
- API bindings remain blocked until the approved owner contract, canonical Rust API, four-language mapping, and verification evidence are present.

## Advancement rule

When an owner draft becomes formally approved/accepted on the active release line, update the row and manifest dependency state from the checked-in source. For an unowned operation, first create and approve a separate operation specification, then link it here and in the normative catalog through its reviewed update. Keep every API row blocked until owner approval, canonical Rust support, all four mappings, and verification evidence exist. Do not use a draft worktree as the source of an API signature.
