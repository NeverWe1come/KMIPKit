package org.kmipkit.extensions;

import java.util.List;
import org.kmipkit.protocol.VersionGate.ProtocolVersion;

/** Callback handler for incoming server-to-client operations. */
public interface ServerRequestHandler {
    public record ItemView(long operation, byte[] uniqueBatchItemId) {}

    S2cOperationResult handleDiscoverVersions(
            ItemView item,
            List<ProtocolVersion> clientVersions,
            List<ProtocolVersion> supportedVersionsOut);

    S2cOperationResult handleNotify(
            ItemView item,
            byte[] uniqueIdentifier,
            byte[] attributesTtlv);

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

    S2cOperationResult handleSetEndpointRole(
            ItemView item,
            long targetRole);
}
