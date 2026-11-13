# Research: Discover Versions Offers

## Decision: Extend the typed Discover Versions request path

The feature will add explicit offers and explicit omission to the existing
typed `DiscoverVersionsRequest`/`ClientBatchItem` path. It will not change
client construction or add automatic protocol negotiation.

**Rationale**: Discover Versions payload choices are per-operation. ADR-0022
separately assigns Message Header version to client configuration or request
override and prohibits implicit downgrade/retransmission. Keeping these
choices separate prevents an offered server version from silently changing
the wire version of later requests.

**Alternatives considered**:

- Change the no-argument builder to offer both 2.1 and 2.0: rejected because
  existing wire behavior must stay KMIP 2.1-only by default.
- Store the offer list in client configuration: rejected because it describes
  one Discover Versions request, not all client operations.
- Automatically negotiate and retry: rejected by ADR-0022 and AGENTS.md §8.

## Decision: Preserve a distinct absent-field state

The existing empty internal Rust vector means “use the default 2.1 offer”; it
cannot also represent an omitted Protocol Version field. The model must
distinguish default, explicit non-empty list, and omitted field. Explicit
empty lists remain invalid; callers use the dedicated omitted-field builder.

**Rationale**: OASIS v2.0 §6.1.16 Table 196 and v2.1 §6.1.16 Table 211 mark
the request field optional. A no-argument API already has a compatibility
contract to send 2.1, so it cannot change to omission.

## Decision: Validate offered values but preserve the preference ranking

Explicit offers must be catalogued and non-empty. The exact caller-supplied
ordering and repetitions are retained. Numeric version ordering is not used
to infer preference; the caller owns the preference order required by OASIS.
When no list is supplied, valid server-announced raw version pairs are
preserved without an offered-intersection check. When a list is supplied,
successful response versions must belong to that list.

**Rationale**: OASIS describes values as ordered by client preference, not by
numeric version. Its repeated-field designation does not create a uniqueness
rule. The no-list response has no client-offer intersection to validate.

## Existing implementation evidence

- `crates/kmipkit-protocol/src/discover_versions.rs`: `with_versions` accepts
  non-empty catalogued values; `new()` uses a default `(2,1)` offer. Response
  conversion currently requires returned versions to be in the supplied
  offer slice.
- `crates/kmipkit-client/src/execute.rs`: `ClientRequest::discover_versions`
  and `ClientBatchItem::discover_versions` expose only the default.
- C ABI, Java `ClientBatchItem`, and Python `client_batch_item_discover_versions`
  expose only no-argument constructors.
- `docs/adr/0022-multi-version-modern-kmip-architecture.md`: the client
  default remains 2.1, multi-version preference lists are already part of the
  architecture, and four-language parity is required.

## Normative source details

Pinned OASIS v2.0 §6.1.16 states the client's optional request list is ordered
by decreasing preference; the server returns only the supported intersection
when that list is supplied. Table 196 marks Protocol Version as optional and
repeatable; Table 197 describes the server's ordered response list.

Pinned OASIS v2.1 §6.1.16 states the same behavior; Table 211 is the request
payload and Table 212 the response payload. Pinned hashes and paths are
recorded in `spec.md`.
