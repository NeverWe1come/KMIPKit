# KMIP 2.1 Client Operation Dependency Matrix

This matrix inventories the 57 catalog elements with `scope_state: client_1_0`. Operation sections refer to **Key Management Interoperability Protocol Specification Version 2.1**, OASIS Standard, 14 December 2020, §§6.1.1–6.1.57. A linked owner/candidate reference does not imply that the specification is approved, or that its API bindings are implemented or verified.

The status snapshot was read from the active `release/1.0.0` base at `fd6b7782` on 2026-10-10. Revalidate owner approval, scope, implementation, and open gates from the active release branch when implementation begins; never promote a row based on a draft worktree or a merged implementation alone.

| OASIS section | Operation | Linked owner specification | Current status / API gate |
|---|---|---|---|
| §6.1.1 | Activate | KMIPKIT-0018 | Draft for independent review; blocked until formally approved and Rust API exists |
| §6.1.2 | Add Attribute | KMIPKIT-0016 | Draft for independent review; blocked until formally approved and Rust API exists |
| §6.1.3 | Adjust Attribute | KMIPKIT-0016 | Draft for independent review; blocked until formally approved and Rust API exists |
| §6.1.4 | Archive | KMIPKIT-0018 | Draft for independent review; blocked until formally approved and Rust API exists |
| §6.1.5 | Cancel | KMIPKIT-0009 | Draft; blocked until formally approved and Rust API exists |
| §6.1.6 | Certify | — | Blocked: approved operation specification required |
| §6.1.7 | Check | — | Blocked: approved operation specification required |
| §6.1.8 | Create | KMIPKIT-0014 | Accepted for implementation; bind after canonical Rust API exists |
| §6.1.9 | Create Key Pair | KMIPKIT-0014 | Accepted for implementation; bind after canonical Rust API exists |
| §6.1.10 | Create Split Key | KMIPKIT-0014 | Accepted for implementation; bind after canonical Rust API exists |
| §6.1.11 | Decrypt | KMIPKIT-0019 | Autonomous implementation authorized; spec not approved; blocked |
| §6.1.12 | Delegated Login | — | Blocked: approved operation specification required |
| §6.1.13 | Delete Attribute | KMIPKIT-0016 | Draft for independent review; blocked until formally approved and Rust API exists |
| §6.1.14 | Derive Key | — | Blocked: approved operation specification required |
| §6.1.15 | Destroy | KMIPKIT-0018 | Draft for independent review; blocked until formally approved and Rust API exists |
| §6.1.16 | Discover Versions | KMIPKIT-0007 | Approved contract; bind after canonical Rust API exists |
| §6.1.17 | Encrypt | KMIPKIT-0019 | Autonomous implementation authorized; spec not approved; blocked |
| §6.1.18 | Export | — | Blocked: approved operation specification required |
| §6.1.19 | Get | KMIPKIT-0017 | Draft; refresh pending review; blocked |
| §6.1.20 | Get Attributes | KMIPKIT-0016 | Draft for independent review; blocked until formally approved and Rust API exists |
| §6.1.21 | Get Attribute List | KMIPKIT-0016 | Draft for independent review; blocked until formally approved and Rust API exists |
| §6.1.22 | Get Constraints | — | Blocked: approved operation specification required |
| §6.1.23 | Get Usage Allocation | — | Blocked: approved operation specification required |
| §6.1.24 | Hash | KMIPKIT-0021 | Approved (PR #70 merged); bind after canonical Rust API exists |
| §6.1.25 | Import | — | Blocked: approved operation specification required |
| §6.1.26 | Interop | — | Blocked: approved operation specification required |
| §6.1.27 | Join Split Key | — | Blocked: approved operation specification required |
| §6.1.28 | Locate | KMIPKIT-0017 | Draft; refresh pending review; blocked |
| §6.1.29 | Log | — | Blocked: approved operation specification required |
| §6.1.30 | Login | — | Blocked: approved operation specification required |
| §6.1.31 | Logout | — | Blocked: approved operation specification required |
| §6.1.32 | MAC | KMIPKIT-0021 | Approved (PR #70 merged); bind after canonical Rust API exists |
| §6.1.33 | MAC Verify | KMIPKIT-0021 | Approved (PR #70 merged); bind after canonical Rust API exists |
| §6.1.34 | Modify Attribute | KMIPKIT-0016 | Draft for independent review; blocked until formally approved and Rust API exists |
| §6.1.35 | Obtain Lease | — | Blocked: approved operation specification required |
| §6.1.36 | Ping | KMIPKIT-0020 | Autonomous implementation authorized and PR merged; spec not approved; blocked |
| §6.1.37 | PKCS#11 | — | Blocked: approved operation specification required |
| §6.1.38 | Poll | KMIPKIT-0009 | Draft; blocked until formally approved; normative/query-response issue remains tracked there |
| §6.1.39 | Process | KMIPKIT-0009 | Draft; blocked until formally approved and Rust API exists |
| §6.1.40 | Query | KMIPKIT-0020 | Autonomous implementation authorized and PR merged; spec not approved; blocked |
| §6.1.41 | Query Asynchronous Requests | KMIPKIT-0009 | Draft; blocked until formally approved; `KMIPKIT-DISC-039` remains a gate for typed response mapping |
| §6.1.42 | Recover | KMIPKIT-0018 | Draft for independent review; blocked until formally approved and Rust API exists |
| §6.1.43 | Register | — | Blocked: approved operation specification required |
| §6.1.44 | Revoke | — | Blocked: approved operation specification required |
| §6.1.45 | Re-certify | — | Blocked: approved operation specification required |
| §6.1.46 | Re-key | — | Blocked: approved operation specification required |
| §6.1.47 | Re-key Key Pair | — | Blocked: approved operation specification required |
| §6.1.48 | Re-Provision | — | Blocked: approved operation specification required |
| §6.1.49 | RNG Retrieve | — | Blocked: approved operation specification required |
| §6.1.50 | RNG Seed | — | Blocked: approved operation specification required |
| §6.1.51 | Set Attribute | KMIPKIT-0016 | Draft for independent review; blocked until formally approved and Rust API exists |
| §6.1.52 | Set Constraints | — | Blocked: approved operation specification required |
| §6.1.53 | Set Defaults | — | Blocked: approved operation specification required |
| §6.1.54 | Set Endpoint Role | — | Blocked: approved operation specification required |
| §6.1.55 | Sign | KMIPKIT-0021 | Approved (PR #70 merged); bind after canonical Rust API exists |
| §6.1.56 | Signature Verify | KMIPKIT-0021 | Approved (PR #70 merged); bind after canonical Rust API exists |
| §6.1.57 | Validate | — | Blocked: approved operation specification required |

## Reconciliation

- Total client_1_0 operations: **57**.
- Linked to a feature specification in the catalog or specs 0007/0009: **30**.
- Formally approved/accepted owners currently in the active release line (0007, 0014, 0021): **9 operations**.
- Linked to a draft or implementation-authorized-but-unapproved owner: **21 operations**.
- No owner specification linked: **27 operations**.
- Total operations blocked pending formal owner approval: **48**.
- Conflicting or post-1.0 drafts in other worktrees are not owners unless their scope is accepted and linked on the active release line. No future feature number is reserved or invented by this matrix.

## Advancement rule

When an owner draft becomes formally approved/accepted on the active release line, update that row from candidate owner to approved owner and add concrete implementation tasks for its Rust/C/Java/Python surfaces. For an unowned operation, first create and approve a separate operation specification, then link it here and in the normative catalog through its reviewed update. Keep every row blocked until owner approval, canonical Rust support, all four mappings, and verification evidence exist. Do not use a draft worktree as the source of an API signature.
