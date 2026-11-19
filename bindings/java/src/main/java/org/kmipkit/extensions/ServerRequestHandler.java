package org.kmipkit.extensions;

import java.util.List;
import org.kmipkit.protocol.VersionGate.ProtocolVersion;

/** Callback handler for incoming server-to-client operations. */
public interface ServerRequestHandler {
    public record ItemView(long operation, byte[] uniqueBatchItemId) {}

    default long handleDiscoverVersions(
            ItemView item,
            ProtocolVersion clientVersion,
            List<ProtocolVersion> supportedVersionsOut) {
        supportedVersionsOut.add(new ProtocolVersion(2, 1));
        return 0; // success (KMIP ResultStatus.Success)
    }

    default long handleNotify(
            ItemView item,
            byte[] uniqueIdentifier,
            byte[] attributesTtlv) {
        return 0;
    }

    default long handlePut(
            ItemView item,
            long putFunction,
            byte[] uniqueIdentifier,
            byte[] replacedUniqueIdentifier,
            byte[] objectTtlv,
            byte[] attributesTtlv) {
        return 0;
    }

    default long handleQuery(
            ItemView item,
            long queryFunctionFlags,
            S2cCapabilities outCapabilities) {
        return 0;
    }

    default long handleSetEndpointRole(
            ItemView item,
            long targetRole) {
        return 0;
    }
}
