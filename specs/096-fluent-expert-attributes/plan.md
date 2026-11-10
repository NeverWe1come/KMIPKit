# KMIPKIT-0096 Implementation Plan

## Design

Represent the only supported parent as a typed `ObjectAttributes` path, so a
caller cannot point into headers, batch metadata, or transport structures.
Positions are absolute, zero-based indices in the final ordered list after
semantic attributes have been assembled. The builder stores explicit
position/item pairs, rejects duplicate indices, then inserts through the
existing `AttributeSet` validation and invokes `CreateRequest::validate()`.

## Verification

1. Add failing tests for accepted insertion and malformed positions.
2. Record Red evidence in a test-only commit.
3. Implement the typed path and validated insertion; record Green separately.
4. Run focused and crate checks, generated API validation, traceability, and
   repository-required checks before pushing the draft PR.
