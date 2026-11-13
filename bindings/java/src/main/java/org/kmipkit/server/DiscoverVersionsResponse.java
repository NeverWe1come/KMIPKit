package org.kmipkit.server;

import java.util.List;
import org.kmipkit.InvalidInputException;
import org.kmipkit.extensions.ProtocolVersion;

/** Typed Discover Versions response payload. */
public final class DiscoverVersionsResponse extends ServerResponse {
    private DiscoverVersionsResponse(long handle, DiscoverVersionsRequest request) {
        super(handle, request);
    }

    public static DiscoverVersionsResponse forRequest(
            DiscoverVersionsRequest request, List<ProtocolVersion> versions) {
        if (request == null || versions == null) throw new InvalidInputException("request and versions are required");
        int[][] values = new int[versions.size()][2];
        for (int index = 0; index < values.length; index++) {
            ProtocolVersion version = versions.get(index);
            if (version == null || version.major() < 0 || version.major() > 255
                    || version.minor() < 0 || version.minor() > 255) {
                throw new InvalidInputException("protocol version is outside the valid range");
            }
            values[index][0] = version.major();
            values[index][1] = version.minor();
        }
        return new DiscoverVersionsResponse(
                ServerNative.responseDiscoverVersions(request.nativeHandle(), values), request);
    }
}
