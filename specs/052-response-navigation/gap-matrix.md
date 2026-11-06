# KMIPKIT-0052 Response API Gap Matrix

Baseline: KMIPKIT-0051 head `8da402286` (2026-11-06). Normative tables are verified against the immutable `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`.

| ID | Client operation | OASIS response source | Existing typed reader | Baseline response-tree access | Planned evidence |
|---|---|---|---|---|---|
| KMIPKIT-0052-R001 | Activate | §6.1.1 Table 165 | `ClientResponseView::activate()` | No uniform raw view | typed accessor and raw tree fixture |
| KMIPKIT-0052-R002 | Add Attribute | §6.1.2 Table 168 | `ClientResponseView::add_attribute()` | No uniform raw view | typed accessor and raw tree fixture |
| KMIPKIT-0052-R003 | Adjust Attribute | §6.1.3 Table 171 | `ClientResponseView::adjust_attribute()` | No uniform raw view | typed accessor and raw tree fixture |
| KMIPKIT-0052-R004 | Archive | §6.1.4 Table 174 | `ClientResponseView::archive()` | No uniform raw view | typed accessor and raw tree fixture |
| KMIPKIT-0052-R005 | Cancel | §6.1.5 Table 177 | `CancelResponse` model exists; no typed reader on `ClientOperationOutcome` | callback-only payload access | typed outcome accessor and payload-presence fixture |
| KMIPKIT-0052-R006 | Certify | §6.1.6 Table 180 | `ClientResponseView::certify()` | No uniform raw view | typed accessor and raw tree fixture |
| KMIPKIT-0052-R007 | Check | §6.1.7 Table 184 | `ClientResponseView::check()` | No uniform raw view | typed accessor and raw tree fixture |
| KMIPKIT-0052-R008 | Create | §6.1.8 Table 187 | `ClientResponseView::create()` | No uniform raw view | typed accessor and raw tree fixture |
| KMIPKIT-0052-R009 | Create Key Pair | §6.1.9 Table 190 | `ClientResponseView::create_key_pair()` | No uniform raw view | typed accessor and raw tree fixture |
| KMIPKIT-0052-R010 | Create Split Key | §6.1.10 Table 194 | `ClientResponseView::create_split_key()` | No uniform raw view | typed accessor and raw tree fixture |
| KMIPKIT-0052-R011 | Decrypt | §6.1.11 Table 197 | `ClientResponseView::decrypt()` | No uniform raw view | typed accessor and raw tree fixture |
| KMIPKIT-0052-R012 | Delegated Login | §6.1.12 Table 200 | `ClientResponseView::delegated_login()` | No uniform raw view | typed accessor and raw tree fixture |
| KMIPKIT-0052-R013 | Delete Attribute | §6.1.13 Table 203 | `ClientResponseView::delete_attribute()` | No uniform raw view | typed accessor and raw tree fixture |
| KMIPKIT-0052-R014 | Derive Key | §6.1.14 Table 206 | `ClientResponseView::derive_key()` | Operation-specific retained TTLV only | typed accessor and raw tree fixture |
| KMIPKIT-0052-R015 | Destroy | §6.1.15 Table 209 | `ClientResponseView::destroy()` | No uniform raw view | typed accessor and raw tree fixture |
| KMIPKIT-0052-R016 | Discover Versions | §6.1.16 Table 212 | `ClientResponseView::discover_versions()` | No uniform raw view | typed accessor and raw tree fixture |
| KMIPKIT-0052-R017 | Encrypt | §6.1.17 Table 215 | `ClientResponseView::encrypt()` | Operation-specific retained TTLV only | typed accessor and raw tree fixture |
| KMIPKIT-0052-R018 | Export | §6.1.18 Table 218 | `ClientResponseView::export()` | Operation-specific retained TTLV only | typed accessor and raw tree fixture |
| KMIPKIT-0052-R019 | Get | §6.1.19 Table 221 | `ClientResponseView::get()` | Owned Any Object only; no whole payload view | typed accessor plus KeyBlock/vendor descendants |
| KMIPKIT-0052-R020 | Get Attributes | §6.1.20 Table 224 | `ClientResponseView::get_attributes()` | No uniform raw view | typed accessor and repeated/unknown attribute fixture |
| KMIPKIT-0052-R021 | Get Attribute List | §6.1.21 Table 227 | `ClientResponseView::get_attribute_list()` | No uniform raw view | typed accessor and repeated/unknown attribute fixture |
| KMIPKIT-0052-R022 | Get Constraints | §6.1.22 Table 230 | `ClientResponseView::get_constraints()` | No uniform raw view | typed accessor and raw tree fixture |
| KMIPKIT-0052-R023 | Get Usage Allocation | §6.1.23 Table 233 | `ClientResponseView::get_usage_allocation()` | Operation-specific retained TTLV only | typed accessor and raw tree fixture |
| KMIPKIT-0052-R024 | Hash | §6.1.24 Table 236 | `ClientResponseView::hash()` | Operation-specific retained TTLV only | typed accessor and raw tree fixture |
| KMIPKIT-0052-R025 | Import | §6.1.25 Table 239 | `ClientResponseView::import()` | Operation-specific retained TTLV only | typed accessor and secret-redaction fixture |
| KMIPKIT-0052-R026 | Interop | §6.1.26 Table 242 | `ClientResponseView::interop()` | Operation-specific retained TTLV only | typed accessor and raw tree fixture |
| KMIPKIT-0052-R027 | Join Split Key | §6.1.27 Table 245 | `ClientResponseView::join_split_key()` | Operation-specific retained TTLV only | typed accessor and raw tree fixture |
| KMIPKIT-0052-R028 | Locate | §6.1.28 Table 248 | `ClientResponseView::locate()` | No uniform raw view | typed accessor and raw tree fixture |
| KMIPKIT-0052-R029 | Log | §6.1.29 Table 251 | `ClientResponseView::log()` | Operation-specific retained TTLV only | typed accessor and raw tree fixture |
| KMIPKIT-0052-R030 | Login | §6.1.30 Table 254 | `ClientResponseView::login()` | No uniform raw view | typed accessor and credential-redaction fixture |
| KMIPKIT-0052-R031 | Logout | §6.1.31 Table 257 | `ClientResponseView::logout()` | No uniform raw view | typed accessor and raw tree fixture |
| KMIPKIT-0052-R032 | MAC | §6.1.32 Table 260 | `ClientResponseView::mac()` | Operation-specific retained TTLV only | typed accessor and raw tree fixture |
| KMIPKIT-0052-R033 | MAC Verify | §6.1.33 Table 263 | `ClientResponseView::mac_verify()` | Operation-specific retained TTLV only | typed accessor and raw tree fixture |
| KMIPKIT-0052-R034 | Modify Attribute | §6.1.34 Table 266 | `ClientResponseView::modify_attribute()` | No uniform raw view | typed accessor and repeated/unknown attribute fixture |
| KMIPKIT-0052-R035 | Obtain Lease | §6.1.35 Table 269 | `ClientResponseView::obtain_lease()` | No uniform raw view | typed accessor and raw tree fixture |
| KMIPKIT-0052-R036 | Ping | §6.1.36 Table 272 | `ClientResponseView::ping()` | No uniform raw view | typed accessor and empty payload fixture |
| KMIPKIT-0052-R037 | PKCS#11 | §6.1.37 Table 274 | `ClientResponseView::pkcs11()` | No uniform raw view | typed accessor and raw tree fixture |
| KMIPKIT-0052-R038 | Poll | §6.1.38 Table 276 and §8.6 Table 399; no dedicated Poll Response Payload table | `PollResponse` model exists; no typed reader on `ClientOperationOutcome` | callback-only payload access | Pending/no-payload and terminal/original-payload fixtures |
| KMIPKIT-0052-R039 | Process | §6.1.39 Table 279 | `ProcessResponse` model exists; no typed reader on `ClientOperationOutcome` | callback-only payload access | typed outcome accessor and raw tree fixture |
| KMIPKIT-0052-R040 | Query | §6.1.40 Table 283 | `ClientResponseView::query()` | No uniform raw view | typed accessor and raw tree fixture |
| KMIPKIT-0052-R041 | Query Asynchronous Requests | §6.1.41 Table 286 (caption says PKCS#11; table body describes Asynchronous Request) | `QueryAsyncRequestsResponse` model exists; no typed reader on `ClientOperationOutcome` | callback-only payload access | generic typed outcome and raw tree fixture; no semantic interpretation |
| KMIPKIT-0052-R042 | Recover | §6.1.42 Table 289 | `ClientResponseView::recover()` | No uniform raw view | typed accessor and raw tree fixture |
| KMIPKIT-0052-R043 | Register | §6.1.43 Table 292 | `ClientResponseView::register()` | No uniform raw view | typed accessor and KeyBlock/vendor descendants |
| KMIPKIT-0052-R044 | Revoke | §6.1.44 Table 296 | `ClientResponseView::revoke()` | No uniform raw view | typed accessor and raw tree fixture |
| KMIPKIT-0052-R045 | Re-certify | §6.1.45 Table 301 | `ClientResponseView::recertify()` | No uniform raw view | typed accessor and raw tree fixture |
| KMIPKIT-0052-R046 | Re-key | §6.1.46 Table 306 | `ClientResponseView::rekey()` | No uniform raw view | typed accessor and KeyBlock/vendor descendants |
| KMIPKIT-0052-R047 | Re-key Key Pair | §6.1.47 Table 311 | `ClientResponseView::rekey_key_pair()` | No uniform raw view | typed accessor and KeyBlock/vendor descendants |
| KMIPKIT-0052-R048 | Re-Provision | §6.1.48 Table 314 | `ClientResponseView::reprovision()` | No uniform raw view | typed accessor and raw tree fixture |
| KMIPKIT-0052-R049 | RNG Retrieve | §6.1.49 Table 317 | `ClientResponseView::rng_retrieve()` | No uniform raw view | typed accessor and secret-redaction fixture |
| KMIPKIT-0052-R050 | RNG Seed | §6.1.50 Table 320 | `ClientResponseView::rng_seed()` | No uniform raw view | typed accessor and secret-redaction fixture |
| KMIPKIT-0052-R051 | Set Attribute | §6.1.51 Table 323 | `ClientResponseView::set_attribute()` | No uniform raw view | typed accessor and repeated/unknown attribute fixture |
| KMIPKIT-0052-R052 | Set Constraints | §6.1.52 Table 326 | `ClientResponseView::set_constraints()` | No uniform raw view | typed accessor and raw tree fixture |
| KMIPKIT-0052-R053 | Set Defaults | §6.1.53 Table 329 | `ClientResponseView::set_defaults()` | No uniform raw view | typed accessor and raw tree fixture |
| KMIPKIT-0052-R054 | Sign | §6.1.55 Table 335 | `ClientResponseView::sign()` | Operation-specific retained TTLV only | typed accessor and raw tree fixture |
| KMIPKIT-0052-R055 | Signature Verify | §6.1.56 Table 338 | `ClientResponseView::signature_verify()` | Operation-specific retained TTLV only | typed accessor and raw tree fixture |
| KMIPKIT-0052-R056 | Validate | §6.1.57 Table 341 | `ClientResponseView::validate()` | No uniform raw view | typed accessor and raw tree fixture |

## Cross-cutting baseline gaps

1. `ClientResponseView` has typed getters for 52 operation response models. The four asynchronous flows Cancel, Poll, Process, and Query Asynchronous Requests have protocol response models and generic execution handling, but no typed response accessor on `ClientOperationOutcome`.
2. `ClientResponseView` has no uniform raw Response Payload accessor. Individual models retain different subsets of response data; this is not a complete substitute for the source payload tree.
3. `ResponseHeaderView::with_ttlv` and `ResponseBatchItemView::with_response_payload` are callback-based views; high-level typed outcomes do not expose a direct uniform raw-node accessor.
4. Existing Rust, C, Java, and Python high-level response paths do not provide equivalent arbitrary-depth generic Response Payload traversal for every operation.
5. Get exposes its Any Object as an owned generic Item, but other typed nested structures do not consistently expose their complete source node. The response-level raw tree will provide a uniform path to every descendant.
6. No table or API promises byte-exact received wire reproduction: the TTLV decoder discards padding bytes.

## Explicit exclusions and source notes

- Set Endpoint Role (§6.1.54, Table 332) is client-initiated but deferred to 1.1 under the approved product scope. It is not part of these 56 rows.
- Table 286 is preserved under its exact pinned caption and table content discrepancy; the response stays generic until the approved `KMIPKIT-DISC-039` resolution governs semantic interpretation.
- Poll uses asynchronous result semantics. A terminal Poll response may carry the original operation's Response Payload; a Pending Poll response has no payload and carries correlation data under §8.6 Table 399.
