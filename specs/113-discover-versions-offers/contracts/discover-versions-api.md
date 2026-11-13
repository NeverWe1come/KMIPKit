# Discover Versions API Contract

The exact symbols are proposed for maintainer review. Implementations may
follow the established naming conventions of each public language while
preserving the common semantics below.

| Language | Explicit offers | Omit offer field |
|---|---|---|
| Rust | `ClientBatchItem::discover_versions_with_versions(Vec<ProtocolVersion>) -> Result<_, _>` | `ClientBatchItem::discover_versions_without_versions()` |
| C | `kmipkit_client_batch_item_discover_versions_with_versions(versions, count, out_item)` | `kmipkit_client_batch_item_discover_versions_without_versions(out_item)` |
| Java | `ClientBatchItem.discoverVersionsWithVersions(List<ProtocolVersion>)` | `ClientBatchItem.discoverVersionsWithoutVersions()` |
| Python | `client_batch_item_discover_versions_with_versions(versions)` | `client_batch_item_discover_versions_without_versions()` |

The C explicit-offers function receives a pointer to an array of
`kmipkit_protocol_version_v1_t` and its count. It rejects a null pointer with
non-zero count, rejects a zero count, and never dereferences the pointer when
the count is zero. It rejects any uncatalogued version. No caller memory is
retained after construction.

All existing no-argument methods remain present and continue to offer only
KMIP 2.1. Explicit offers preserve order and repetition; callers supply the
list in decreasing preference order as OASIS requires, and an empty list is
an input error. The omitted-field method encodes no Protocol Version child.
The request Message Header version is selected independently by existing
client configuration/request options.

Response handling preserves returned values and order. For explicit offers,
it rejects a successful value absent from the offer list. For omitted offers,
it does not apply an offer-intersection check. Malformed response structures
remain errors in both cases.
