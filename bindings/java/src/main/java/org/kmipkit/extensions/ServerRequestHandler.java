package org.kmipkit.extensions;

import java.util.List;
import org.kmipkit.protocol.VersionGate.ProtocolVersion;

/** Callback handler for incoming server-to-client operations. */
public interface ServerRequestHandler {
    public record ItemView(long operation, byte[] uniqueBatchItemId) {}

    long handleDiscoverVersions(
            ItemView item,
            ProtocolVersion clientVersion,
            List<ProtocolVersion> supportedVersionsOut);

    long handleNotify(
            ItemView item,
            byte[] uniqueIdentifier,
            byte[] attributesTtlv);

    long handlePut(
            ItemView item,
            long putFunction,
            byte[] uniqueIdentifier,
            byte[] replacedUniqueIdentifier,
            byte[] objectTtlv,
            byte[] attributesTtlv);

    long handleQuery(
            ItemView item,
            long queryFunctionFlags,
            S2cCapabilities outCapabilities);

    long handleSetEndpointRole(
            ItemView item,
            long targetRole);
}
