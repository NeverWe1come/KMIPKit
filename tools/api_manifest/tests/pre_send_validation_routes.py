"""Exact 0051 trait routes kept separate from historical standard-trait slices."""

from functools import cache
import json
from pathlib import Path

REQUEST_TYPES = frozenset("""
ActivateRequest AddAttributeRequest AdjustAttributeRequest ArchiveRequest
CancelRequest CertifyRequest CheckRequest CreateKeyPairRequest CreateRequest
CreateSplitKeyRequest DecryptRequest DelegatedLoginRequest DeleteAttributeRequest
DeriveKeyRequest DestroyRequest DiscoverVersionsRequest EncryptRequest ExportRequest
GetAttributeListRequest GetAttributesRequest GetConstraintsRequest GetRequest
GetUsageAllocationRequest HashRequest ImportRequest InteropRequest JoinSplitKeyRequest
LocateRequest LogRequest LoginRequest LogoutRequest MacRequest MacVerifyRequest
ModifyAttributeRequest ObtainLeaseRequest PingRequest Pkcs11Request PollRequest
ProcessRequest QueryAsyncRequestsRequest QueryRequest ReProvisionRequest
RecertifyRequest RecoverRequest RegisterRequest RekeyKeyPairRequest RekeyRequest
RevokeRequest RngRetrieveRequest RngSeedRequest SetAttributeRequest SetConstraintsRequest
SetDefaultsRequest SignRequest SignatureVerifyRequest ValidateRequest
""".split())
DIRECT_ROOT_REQUEST_TYPES = frozenset("""
CancelRequest DelegatedLoginRequest GetConstraintsRequest GetUsageAllocationRequest
LoginRequest LogoutRequest PollRequest ProcessRequest QueryAsyncRequestsRequest
RngRetrieveRequest RngSeedRequest SetConstraintsRequest SetDefaultsRequest
""".split())
TRAIT_PATH = "kmipkit_protocol::validation::typed_requests::ValidateKmipRequest"
TRAIT_MEMBERS = ("SCHEMA", "validate")
VALIDATION_TRAIT_ROUTES = frozenset(
    (package, f"<{prefix}{request} as {TRAIT_PATH}>::{member}")
    for package, prefix, requests in (
        ("rust-kmipkit-protocol", "kmipkit_protocol::", REQUEST_TYPES),
        ("rust-kmipkit", "kmipkit::protocol::", REQUEST_TYPES),
        ("rust-kmipkit", "kmipkit::", DIRECT_ROOT_REQUEST_TYPES),
        ("rust-kmipkit-client", "kmipkit_client::", ("Pkcs11Request",)),
    )
    for request in requests
    for member in TRAIT_MEMBERS
)


def is_pre_send_validation_trait(row: dict) -> bool:
    """Recognize only the 252 existing package/path pairs pinned by 0051."""
    return (row.get("packageId"), row.get("exportName")) in VALIDATION_TRAIT_ROUTES


@cache
def validation_stack_routes() -> frozenset[tuple[str, str]]:
    """Read the exact added routes from the two independently audited snapshots."""
    root = Path(__file__).resolve().parents[3]
    reports = (
        "specs/051-pre-send-validation/reports/rust-export-report-2026-11-07.json",
        "specs/050-parameters-auth-client/reports/rust-export-report-post-sync-2026-11-07.json",
    )
    inventories = []
    for path in reports:
        report = json.loads((root / path).read_text(encoding="utf-8"))
        inventories.append({
            (row["packageId"], row["exportName"])
            for row in report["declarations"] + report["conditionalDeclarations"]
        })
    return frozenset(inventories[0] - inventories[1])
