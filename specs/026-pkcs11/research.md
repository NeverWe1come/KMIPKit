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
