# KMIPKIT-0048 Approval Record

The maintainer's direct approval of the Phase 5 plan on 2026-11-05 authorizes
KMIPKIT-0048 and the ordered 0047→0053 implementation sequence. This record
captures the approved scope for typed Rust access to the catalogued direct
KMIP 2.1 attributes. Language-binding parity remains assigned to KMIPKIT-0053.

The maintained plan's stated count of 63 catalogued attribute records was
checked against the normative catalog and pinned OASIS KMIP 2.1 source. The
source shows that “Certificate Attributes” in §4.6/Table 39 has no direct TTLV
tag or encoded value category. Therefore 62 records are applicable to direct
`AttributeSet`, `CurrentAttribute`, and `NewAttribute` Items. This exclusion
follows the standard and does not change the approved product scope.
