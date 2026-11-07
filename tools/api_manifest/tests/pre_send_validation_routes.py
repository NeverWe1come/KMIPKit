"""Exact 0051 trait routes kept separate from historical standard-trait slices."""

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
