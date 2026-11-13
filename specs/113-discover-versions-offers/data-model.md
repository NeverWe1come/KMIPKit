# Data Model: Discover Versions Offers

## Offer State

| State | Meaning | Request payload |
|---|---|---|
| Default | Existing no-argument API behavior | One Protocol Version `(2,1)` |
| Explicit offers | Caller-supplied non-empty, catalogued preference list | One repeated Protocol Version structure per value, preserving order and duplicates |
| Omitted offers | Caller deliberately asks the server to report its supported versions | No Protocol Version child |

The omitted state is distinct from default. An explicit empty list is invalid;
the caller selects the omitted state directly.

## Protocol Version Value

- `major`: signed KMIP integer value.
- `minor`: signed KMIP integer value.
- Client offers: each pair must be present in the compiled KMIPKit version
  catalog.
- Server response values: each pair must be structurally valid; if the request
  had explicit offers, each value must also appear in that list. If the offer
  field was omitted, raw well-formed values are preserved even if not in the
  current catalog.

## Response

- Preserve the server's preference order and repeated values.
- A successful empty response is distinct from a non-success KMIP result.
- Non-success result handling remains the existing typed result behavior.
- Discover Versions results do not alter client configuration or select a
  version for subsequent requests.
