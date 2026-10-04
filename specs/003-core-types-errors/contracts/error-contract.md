# Contract: KMIP Result and Client Error Surface

This contract fixes externally visible behavior; identifiers may follow established crate naming if semantics remain exact.

## KMIP result

- ResultStatus and ResultReason retain their original 32-bit values and provide generated known-value interpretation.
- KmipOperationResult contains status, optional reason, and optional message.
- Construction enforces the §9.18 invariant: Failure has a reason and Success does not. Pending, undone, and unknown statuses are retained without additional reason-presence assumptions here.
- Result Message is untrusted server text. It is available to explicit caller inspection and is never formatted or logged automatically.

## Request delivery

RequestDeliveryState has exactly NotSent, PossiblySent, and ResponseStarted values:
- NotSent: no request bytes were sent.
- PossiblySent: transmission began but no response byte has been received. A read attempt returning zero bytes leaves this state unchanged.
- ResponseStarted: at least one response byte has been received but no complete KMIP operation result is available.

Local validation before send is NotSent. An incomplete write after transmission starts is PossiblySent. A read or response-processing failure after bytes arrive is ResponseStarted. A complete KMIP server result does not receive a local delivery state.

## Client error

The public client error distinguishes local validation, protocol-processing, and transport failures from a KMIP server result. A server result is surfaced through a dedicated server-result case containing KmipOperationResult. Local failures retain safe cause categories and report delivery state when applicable. Arbitrary source text and payloads are discarded before retention; sanitization consumes and drops the original source, which cannot be reached through the public error/source chain. Default Display and Debug for ResultMessage, KmipOperationResult, and public client errors omit cause text, Result Message, secrets, and raw bodies. This feature adds no automatic serialization for these values. Delivery state is evidence only and does not itself signal that retrying is safe. The separate client-execution feature retains the project rule against automatic retries.

## Compatibility

Known values derive from generated catalog data; unknown numeric values remain intact. Public enums are future-extensible. Foreign-language mappings and stable ABI codes are outside this feature.
