# Research: KMIP 2.1 Query and Ping

## Sources reviewed

1. Pinned OASIS KMIP Specification v2.1 HTML, immutable copy at `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`.
2. Repository source hierarchy in `docs/compliance/document-hierarchy.md`.
3. Catalog entries for §6.1.36, §6.1.40, §11.44, Query/Ping payload tables, requirements, test mappings, and discrepancies.
4. Existing request, batch, typed operation, fake-transport, result, and TTLV contracts in the release branch.

## Decisions

- Include Ping and ordinary Query only. Query Asynchronous Requests is a separate operation (§6.1.41) and stays in its existing asynchronous-operation scope.
- Model Ping with empty operation request and response payloads. Use shared KMIP Result and transport delivery behavior.
- Model Query with one or more repeatable Query Function values and optional Object Groups. Object Groups is the §7.23 structure with zero or more repeated Object Group attributes; the attribute value is Text String (§4.35, Tables 99–100). Expose the 14 assigned Query Function values and allow valid extension/future values. The Object Group Member enum in §11.33 is not part of this structure.
- Model every response member from Table 283 with its source cardinality. For complex or future nested values, retain structurally valid generic TTLV Items rather than inventing server-side semantics. Accept both response forms described by §6.1.40 and Table 283 despite their conflict; track it as open `KMIPKIT-DISC-045` and do not claim server conformance for that case.
- Preserve Query result values exactly; the API does not certify the server's support or authorization.
- Implement Query Extension List/Map request construction exactly as requested by the caller. Their response precedence and support conditions are server obligations in §6.1.40.
- Use a focused fake transport as the deterministic integration boundary. Do not require a live server to execute unit or contract tests.
- Keep spec-preparation and implementation separate. This PR contains no production API or code change; the implementation follows only after its specification gate.

## Source conflict review

The pinned v2.1 source says the Query response payload is empty if there are no values to return, but Table 283 also marks Protection Storage Masks as required and only explicitly permits its list to be empty. The catalog records this as open `KMIPKIT-DISC-045`. No approved erratum or decision is pinned in the repository. The client accepts both forms so its behavior is deterministic without inventing a server-conformance interpretation.

## Test evidence limitations

- `TC-PING-1-21` (catalog ID `KMIPKIT-TEST-CN01-2-67`, source Test Cases §2.67) is linked to Ping but its XML fixture is unavailable.
- Query is linked to three profile test cases and one quantum-safe test mapping. Their fixtures are unavailable. The quantum-safe mapping is marked weak in the catalog, and the source label/case naming discrepancy is already recorded there.
- HTTPS, XML, and JSON profile cases do not prove this TTLV-only feature. No official fixture pass or profile-conformance claim is made.
- Existing catalog discrepancy `KMIPKIT-DISC-002` concerns HTTPS profile evidence. `KMIPKIT-DISC-039` concerns Query Asynchronous Requests, not ordinary Query. Neither changes this feature's TTLV operation contract.
- The feature's executable tests must therefore use source-derived TTLV vectors, direct structural expectations, malformed/negative inputs, and fake-transport scenarios, with exact OASIS citations. Such tests are not labelled official OASIS fixture passes.