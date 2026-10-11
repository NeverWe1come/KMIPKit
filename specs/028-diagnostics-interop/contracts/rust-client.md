# Rust Client Contract: Interop, Log, and Validate

Additive typed client surface following existing operation conventions. OASIS source tables are named in the approved spec. Cross-language mappings are owned by KMIPKIT-0043.

## Types and methods

- InteropRequest requires Interop Function and Interop Identifier; InteropResponse has no operation-specific success fields; InteropError is sanitized. Provide Client::interop.
- LogRequest requires Log Message; LogResponse has no operation-specific success fields; LogError is sanitized. Provide Client::log.
- ValidateRequest represents one chain with repeated Certificate and/or Unique Identifier values and optional Validity Date; ValidateResponse exposes required open Validity Indicator; ValidateError is sanitized. Provide Client::validate.
- Admit corresponding ClientRequest and ClientOperation variants.
- Each call performs one explicit exchange and preserves common result, Pending, delivery, generic TTLV, and unknown values. No retry or implicit Poll.
- Interop server availability is not checked by this client. Log Message and certificate material are redacted. Validate does not perform local certificate validation or trust decisions.

Use these Rust names as inputs to the API manifest once 0043 is integrated. Do not hand-edit generated C, Java, or Python output.
