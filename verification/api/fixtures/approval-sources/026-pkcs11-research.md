# Research: KMIP 2.1 PKCS#11 Forwarding

## Decision 1: Use the typed Rust pipeline

Add a protocol request/response and admit it to the existing closed client pipeline. Each explicit call uses the common one-exchange path. This retains TTLV validation, response association, transport security, Pending behavior, and delivery evidence. Raw bytes or a separate transport would bypass that boundary.

## Decision 2: Preserve structures rather than parse PKCS#11 profile data

Use the catalog-defined Structure form for Function, Interface, Input Parameters, Output Parameters, and Return Code. Preserve Function's Enumeration-or-Integer choice and unknown values. Keep profile-specific contents as generic TTLV.

**Rationale**: §6.1.37 Tables 273–274 define KMIP fields; detailed parameters are delegated to the PKCS#11 Profile. A profile codec is outside approved scope.

## Decision 3: Keep absence and correlation explicit

Do not synthesize the default V3.0 interface. Preserve optional absence and forward supplied previous Correlation Value exactly. Approved KMIPKIT-DISC-018 treats mixed-case “Must” as descriptive and permits caller-controlled correlation.

## Decision 4: Redact opaque values and retain common result behavior

Use shared sanitized errors and secret storage. Preserve Success, Failure, Pending, Result Reason, permitted Result Message, unknown values, and delivery state. Never format opaque parameters or correlation bytes.

## Decision 5: Gate only binding work on KMIPKIT-0043

Rust models and client operation proceed independently. Add manifest mapping, generated C/Java/Python signatures, adapters, and parity tests when the approved 0043 capability mapping is integrated. Never edit generated output by hand.

## Evidence reviewed

Pinned OASIS Specification v2.1 §6.1.37 Tables 273–275 and §11.36; approved spec 026; existing protocol/client conventions and catalog entries. No direct catalog Test Case is linked to PKCS#11.


## Source interpretation KMIPKIT-026-SOURCE-001 — vendor Function Integer

**Resolved context**: client-initiated PKCS#11 forwarding requests and successful
responses in §6.1.37 only. The maintainer explicitly authorized selecting KMIP
2.1 behavior for normative review disagreements in this implementation task.
This is a traced implementation interpretation, not an official OASIS erratum.

The same primary normative document, *Key Management Interoperability Protocol
Specification Version 2.1*, OASIS Standard, 14 December 2020, contains both:

- §6.1.37 Tables 273/274 Function descriptions expressly distinguish Enumeration
  for PKCS#11-defined functions and Integer for vendor-defined functions.
- §7.26 Table 378 describes an outer Function Structure and one required nested
  Function Enumeration. It does not list the vendor Integer alternative.

There is no separately declared normative machine-readable definition or
approved pinned erratum resolving this difference. Profiles and Test Cases
cannot override the base specification under `docs/compliance/document-hierarchy.md`;
the Usage Guide and catalog are not used to create the Integer allowance.

The observable alternatives are to apply Table 378's nested Enumeration
without a vendor exception, rejecting all vendor Integer requests/responses,
or to retain its outer Structure while applying the explicit operation-context
vendor Integer alternative in both Tables 273 and 274. The approved feature
specification already selects the latter. In this forwarding API the specific
operation description governs its Function value alternatives: one nested
Enumeration for standard functions or one nested Integer for caller-selected
vendor functions. Required cardinality and outer Structure remain Table 378's
contract; Integer is not accepted for Return Code or any other Enumeration.
Unknown children and open raw values remain lossless. The caller selects the
function/type; KMIPKit does not infer that a remote server supports it.

**Compatibility effects**: a server/profile that implements only standard
functions may reject a vendor function. Its Table 275 reason is preserved;
no success, official fixture pass, PKCS#11 profile support, or complete
conformance claim is inferred from local forwarding tests.

**Security effects**: Function values remain redacted and owned generic TTLV
remains zeroizing. No local PKCS#11 implementation, profile codec, implicit
function choice, retry, Poll or transport is added.

**Verification**: `table_273_function_preserves_enumeration_or_integer` covers
request alternatives; `table_274_success_retains_required_optional_and_unknown_fields`
and `pkcs11_sends_table_273_once_and_exposes_table_274_losslessly` preserve the
vendor Integer response and encoded request. Wrong Function type and duplicate
Function negatives remain in `table_273_rejects_missing_duplicate_or_wrong_function`
and `table_274_rejects_missing_duplicate_wrong_type_and_out_of_order_fields`.

## Known nested members remain structural contracts

Opaque profile contents do not waive the base structure tables. §7.27 Table
379 Input Parameters has a 0..1 known ByteString member; §7.28 Table 380
Interface a 0..1 known TextString member; §7.29 Table 381 Output Parameters a
0..1 known ByteString member. Empty/unknown-only structures remain valid;
duplicate known children or incompatible known scalar types are rejected.
Unknown and repeated extension children remain preserved and uninterpreted.
