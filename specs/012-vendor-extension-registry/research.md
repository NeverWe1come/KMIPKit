# Research: KMIP 2.1 Vendor Extension Registry

## Scope and source hierarchy

The decision basis is the accepted KMIPKit roadmap, ADR-0007, ADR-0006, ADR-0013, ADR-0012, ADR-0014, the project constitution, and the pinned OASIS KMIP Specification v2.1. The normative review uses checked-in immutable sources and the catalog only; no build step downloads OASIS content.

- OASIS §9.13 and Table 418 define Message Extension fields and the Vendor Identification character set.
- OASIS §7.13 and Table 365 define Extension Information metadata.
- OASIS §11.44 and Table 476 define Query Extension List and Query Extension Map functions.
- The existing generic message model and KMIPKIT-0007 own generic preservation and common criticality behavior.
- ADR-0013 assigns the per-client registry, typed validation, and adapter work to KMIPKIT-0012.

## Decisions

### Registry and dependency direction

Keep extension schema/value types in the existing protocol crate, which already owns protocol models and depends on the TTLV crate. Keep the immutable registry attached to client configuration, where typed requests and responses are evaluated. Preserve the existing dependency direction; protocol models and the registry gain no transport or TLS access. Expose required opaque handles only through the FFI crate, then provide Java 17/JNI and Python 3.12/CFFI facades under ADR-0006.

The current workspace has no Java/Python binding projects and no public API manifest. Since parity and generated adapters are explicit 1.0 requirements, this feature establishes the initial checked-in API manifest at specification/api/public-api.json. Later API specifications extend this manifest rather than creating competing sources. The manifest describes mechanical public declarations. Rust protocol/validation logic and idiomatic Java/Python facades remain reviewed source, not generated business logic.

### Registry definitions and validation

Use typed, data-only definition builders; loading external manifest files and the declarative extension SDK remain post-1.0 roadmap work. This adds no runtime schema-language dependency. A definition contains stable identity (vendor identifier, extension name, extension version), compatibility metadata, one finite discriminator, and a recursive TTLV schema.

The finite schema vocabulary contains expected TTLV type; child tag and cardinality; optional child-order constraints where the vendor schema declares order; text/octet length bounds; signed/unsigned numeric bounds; allowed enumeration values; allowed/required bitmask bits; and whether undeclared children are accepted and preserved. It contains no expression evaluator, regex engine, callback, or arbitrary code. Constraints outside this vocabulary cannot be registered as recognized under this feature.

The discriminator is a path of structure-child tags plus one exact scalar type/value. Registry construction rejects a duplicate (vendor identifier, discriminator path, scalar type/value) key. Distinct keys may use different paths that are simultaneously present in one payload. Inbound lookup first collects matching discriminator keys for the received vendor: zero matches is unrecognized, more than one is ambiguous and unrecognized without schema selection, and exactly one proceeds to complete schema validation. The multiple-match fixture contains two distinct paths and exact values from registered keys in one payload; no registration-order winner is allowed. This conservative rule bounds schema work to at most one full validation after the bounded TTLV discriminator scan. A schema-invalid critical extension continues through KMIPKIT-0007 rejection; a schema-invalid non-critical extension remains losslessly inspectable and is not exposed as a partial typed value.

An unvalidated generic TTLV subtree may be submitted to the registry validator to create a sealed validated value. Client::execute accepts only the protocol's closed typed request set and sealed validated values; it never accepts a generic Item, raw body, or caller conversion trait as a bypass.

### Security and compatibility

Apply the existing 16 MiB message, depth-64, and 100,000-element decoder limits to extension-value validation. Additionally apply every ExtensionRegistryLimits field from FR-012, including discriminator scalar bytes, allowed-enumeration and ordering-constraint members, payload-index records, and lookup comparisons. Count registry inputs before clone/reserve. For inbound lookup, count before allocation and build one shared bounded temporary index with at most 200,000 records: one descriptor per Structure plus one `(24-bit tag, original child index)` pair per child. This is bounded by the 100,000-item TTLV cap; fixed-pass radix ordering preserves duplicate tags and the generic subtree. Resolve each path by lower/upper-bound searches, where absent or repeated tags are non-matches. Bound work to at most 35 tag comparisons per path step, 1,024 definitions × 64 path steps, and the configured aggregate comparison ceiling; a lower configured budget returns a stable redacted resource-limit error without partial recognition. Do not log or format extension payload values. Reuse KMIPKit secret ownership/zeroization rules for owned secret-bearing values and document C/Java/Python runtime copies. Data-only schema evaluation is bounded by value traversal limits and does not get transport/TLS access.

Schema membership work is also indexed: allowed Enumeration sets are sorted once and binary-searched (at most 13 numeric comparisons per input Enumeration), while ordering constraints are unique acyclic directed tag edges compiled to child-rule indexes. Validation records each declared child's first/last position during one input traversal, then evaluates every edge once; all occurrences of the `before_tag` must precede all occurrences of `after_tag`, while an absent endpoint is left to cardinality validation. This avoids work proportional to repeated-input count multiplied by constraint-list length. Callers may raise or lower defaults within each hard maximum; above-maximum configuration is rejected consistently across adapters.

Because vendor payloads may contain secret-bearing values, KMIPKIT-0012 satisfies KMIPKIT-0007-OD-006 with an operation-specific test through the existing `Client::execute`: test partial writes and transport success/error, zeroizing-owner lifetime through exchange return, initialized-range cleanup immediately before drop, delivery-state classification, redaction, and absence of retries. This is required test evidence, not authorization for a new generic serialization path.

Do not add external runtime dependencies. This feature pins Python 3.12 for the standard-library code generator in repository automation; generated files are committed and CI runs the generator in check mode. Any later dependency proposal must pass the accepted dependency policy.

### Alternatives rejected

- Matching only on Vendor Identification: rejected by ADR-0013 because one vendor may define multiple extensions and the identifier does not prove semantic understanding.
- Arbitrary user validators or runtime plugins: rejected by ADR-0007/0013 due code-execution, panic, trust, ABI, and isolation risks.
- Global registry: rejected because client configuration and recognition must be isolated.
- External generic-schema runtime: rejected for 1.0 because a new schema engine adds runtime dependency and validation/security surface; the finite vocabulary covers this feature's contract.
- Deferring all cross-language surfaces: rejected because the accepted 1.0 plan and ADR-0006 require language equivalence and the roadmap says 0012 must complete before API parity is frozen.
- Implementing Query transport operations here: rejected because this feature owns registry metadata, while Query operation execution belongs to the all-operations specification family.

## Remaining design constraint

Before code generation is implemented, the public API manifest version and generator output paths must be frozen in the feature contracts. This is the first increment of the later full 1.0 API inventory; it must not claim the overall API is complete.
