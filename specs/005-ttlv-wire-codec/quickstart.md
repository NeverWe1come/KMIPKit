# Quickstart: TTLV Codec Review Scenarios

This file defines the end-to-end scenarios the 1.0 Rust example must demonstrate once KMIPKIT-0004 is merged and the contract names are reconciled. The example is not yet executable because the model API and Reserved-tag decision are not available on this release-base worktree.

## Encode a request item

1. Construct a generic Structure in the exact field order required by its KMIP 2.1 structure definition.
2. Add at least one leaf value from each supported Item Type in separate reviewed vectors.
3. Encode one root Item with default limits.
4. Compare Tag, Type, Length, Item Value, and padding against the exact expected bytes.
5. Confirm a deliberately oversized model fails before output grows beyond the configured byte limit.

## Decode a response item

1. Load a reviewed KMIP 2.1 TTLV vector as bytes.
2. Decode exactly one item using default limits.
3. Inspect the generic tag, item type, value bits, and ordered Structure children through the 004 model API.
4. Re-encode and compare with the vector's canonical form. For nonzero padding values, expect canonical zero padding on output while retaining the same represented Item Value.
5. Confirm unknown extension Tags, Enumeration values, mask bits, repeated child Tags, and Structure order remain available to the caller.

## Reject malformed input safely

Run a byte-vector case for each malformed category in `spec.md`: truncated fields, unsupported Type, invalid fixed length, invalid Boolean/UTF-8, invalid parent boundary, arithmetic overflow, excess limits, and trailing bytes. Every case must return a structured error, no partial Item, and no payload bytes in formatted diagnostics.

## Implementation readiness

Turn the scenarios above into executable integration tests and an example under the final public contract after PR #14 lands. Do not publish an example using guessed model or codec APIs.
