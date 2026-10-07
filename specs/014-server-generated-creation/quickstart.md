# Quickstart: Server-Generated Object Creation

This guide becomes runnable when implementation tasks are complete. It assumes a client configured by the separately approved KMIPKIT-0013 production transport.

## Create one object

1. Construct a client with explicit TLS trust, client identity, endpoint, and timeouts.
2. Build a CreateRequest with an explicit Object Type and caller-selected attributes.
3. Invoke the typed Create method or place its closed request variant in a ClientBatch.
4. Check the KMIP result and use the returned Unique Identifier only with a successful typed payload.

## Create a key pair

1. Select algorithm, length, domain parameters, usage, and protection policy explicitly.
2. Supply common, private-only, and public-only attributes in separate ordered sets.
3. For each Table 191 value, apply key-specific-over-common precedence per key and ensure both effective values are either absent or equal.
4. Read both returned identifiers; do not infer one from the other.

## Create split-key parts

1. Select the split method, part count, threshold, and optional source object identifier explicitly.
2. Invoke one request. If the result is Pending, use the separate KMIPKIT-0009 APIs explicitly.
3. The client advertises its configured local response-byte limit as Maximum Response Size, capped to the largest KMIP Integer; the local cap is still enforced.
4. Retain all returned identifiers in their response order.

## Verify

Run focused protocol and client tests, including the separately identified TC-CREATE-SD-1-21 fixture and table-derived cases, protocol/client coverage gates, formatting, Clippy, documentation build, and the cross-platform CI matrix. Table-derived tests are not described as official OASIS cases.
