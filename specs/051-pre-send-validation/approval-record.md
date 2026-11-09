# KMIPKIT-0051 Approval Record

The maintainer approved the Phase 5 plan on 2026-11-05, including the KMIPKIT-0051 deliverable: schema-aware `validate()` for requests and known structures, and rejection of locally checkable normative violations before transport writes with `NotSent` delivery evidence.

The approved plan authorizes KMIPKIT-0051 to be developed on a branch based on the pushed KMIPKIT-0050 branch while PR #118 remains open. Its integration PR is stacked on KMIPKIT-0050 and remains a draft until required checks are available. This record does not authorize changing the OASIS source, accepted architecture boundaries, server-side behavior, or the human review/merge rules.

The maintainer's 2026-11-09 correction plan explicitly extends the existing 0051 branch to apply the approved multi-version `VersionGate` using the effective version. KMIP 2.1 remains the default and the source of truth for 0051 operation schemas; the 2.0 and 3.0 pinned sources are used only for their approved version-specific operation, tag, and mutation gates under KMIPKIT-0069–0071. This extension does not change transport, TLS, encoding, or server-side behavior.

The approved Phase 5 plan also establishes the ambiguity rule: the pinned OASIS source for the effective version is authoritative; when it does not define an API or implementation choice, the coordinator records and applies the recommended project behavior without expanding protocol scope.

Scope clarification from the approved 1.0 roadmap: KMIP v2.1 §6.1 defines 57 client-initiated operations. KMIPKit 1.0 includes the 56 operations present in the KMIPKIT-0050 client API. Set Endpoint Role (§6.1.54, Table 331) is client-initiated and is deferred to 1.1; it is not classified as server-initiated. §6.2 server-initiated operations remain outside the feature.
