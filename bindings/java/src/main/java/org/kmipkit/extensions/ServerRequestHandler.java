package org.kmipkit.extensions;

import java.util.List;
import org.jspecify.annotations.Nullable;
import org.kmipkit.protocol.VersionGate.ProtocolVersion;

/** Callback handler for incoming server-to-client operations. */
public interface ServerRequestHandler {
    public record ItemView(long operation, byte[] uniqueBatchItemId) {}

    S2cOperationResult handleDiscoverVersions(
            ItemView item,
            List<ProtocolVersion> clientVersions,
            List<ProtocolVersion> supportedVersionsOut);

    default S2cOperationResult handleNotify(
            ItemView item, byte[] uniqueIdentifier, byte[] attributesTtlv) {
        return S2cOperationResult.failure(0x05L);
    }

    /** Deleted references are an ordered Request Payload TTLV structure, possibly empty. */
    default S2cOperationResult handleNotify(
            ItemView item, byte[] uniqueIdentifier, byte[] attributesTtlv,
            byte[] deletedAttributeReferencesTtlv) {
        return handleNotify(item, uniqueIdentifier, attributesTtlv);
    }

    S2cOperationResult handlePut(
            ItemView item,
            long putFunction,
            byte[] uniqueIdentifier,
            byte[] replacedUniqueIdentifier,
            byte[] objectTtlv,
            byte[] attributesTtlv);

    S2cOperationResult handleQuery(
            ItemView item,
            long queryFunctionFlags,
            S2cCapabilities outCapabilities);

    default S2cOperationResult handleSetEndpointRole(ItemView item, long targetRole) {
        return S2cOperationResult.failure(0x05L);
    }

    /** Receives optional vendor text and fills the successful response's role and vendor. */
    default S2cOperationResult handleSetEndpointRole(
            ItemView item, long targetRole, @Nullable String vendorIdentification,
            S2cEndpointRoleResponse response) {
        return handleSetEndpointRole(item, targetRole);
    }
}
