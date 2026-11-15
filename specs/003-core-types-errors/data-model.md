# Data Model: KMIP Shared Error Contract

## ResultStatus

- Meaning: The server-reported KMIP Result Status.
- Representation: Preserve the 32-bit KMIP Enumeration value exactly; known names are resolved from generated catalog data.
- Validation: No unknown value is coerced to a known status. Only values explicitly known to mean Success or Failure drive the §9.18 reason-presence invariant.

## ResultReason

- Meaning: The server-reported KMIP Result Reason.
- Representation: Preserve the 32-bit KMIP Enumeration value exactly; known names are resolved from generated catalog data.
- Validation: Preserve unknown values. Do not infer operation-specific meaning.

## ResultMessage

- Meaning: Optional, server-controlled explanatory text.
- Representation: Distinguish absent from present, including present empty text.
- Security: Untrusted; available for explicit caller inspection; omitted from default Display and KMIPKit-generated logs.

## KmipOperationResult

Fields: status, optional reason, optional message.

Invariants:
- Failure requires a reason.
- Success forbids a reason.
- Pending, undone, and unknown statuses are preserved without extra reason-presence assumptions in this feature.
- A malformed combination is rejected; no reason is synthesized.

## RequestDeliveryState

Values:
- NotSent: no request bytes were sent.
- PossiblySent: request transmission began, but no response byte has been received. A read attempt returning zero bytes does not advance this state.
- ResponseStarted: at least one response byte has been received, but a complete KMIP operation result is unavailable.

A complete server result is represented as KmipOperationResult and is not classified as a local request-handling failure.

## TransportError

Represents a local transport failure with one RequestDeliveryState and a safe cause category. Construction consumes and drops any arbitrary source error; neither its text nor payload remains retained or reachable through the public source chain.

## ProtocolError

Represents local protocol-processing or value-validation failure. It retains a safe cause category. Arbitrary source text and payloads are discarded before retention, the original source is dropped during construction, and it cannot be reached through the public `source()` chain. Display/Debug output is safe.

## ClientFailure

Contains:
- Safe category: validation, protocol processing, or transport.
- Exactly one request delivery state for every local failure. Validation before transmission is `NotSent`; transmission or response-processing failures use the strongest observed state.
- Safe cause category retained for explicit inspection.
- Unsafe original source strings and payloads are discarded before retention, the original source is dropped during construction, and it is not reachable through the public error/source chain.
- Safe default display that does not format cause text or protocol message text.

A KMIP server operation result is surfaced through a dedicated client-error case containing the complete KmipOperationResult. It is distinct from local ClientFailure and has no local request delivery state because a complete response was received.
