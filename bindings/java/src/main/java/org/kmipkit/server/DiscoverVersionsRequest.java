package org.kmipkit.server;

import java.util.ArrayList;
import java.util.List;
import org.kmipkit.extensions.ProtocolVersion;

/** Received Discover Versions request. */
public final class DiscoverVersionsRequest extends ServerRequest {
    DiscoverVersionsRequest(long handle, ServerRequestBatch owner) { super(handle, owner, 30); }

    public List<ProtocolVersion> requestedVersions() {
        long count = ServerNative.discoverVersionsCount(nativeHandle());
        List<ProtocolVersion> versions = new ArrayList<>((int) count);
        for (long index = 0; index < count; index++) {
            int[] value = ServerNative.discoverVersionAt(nativeHandle(), index);
            versions.add(new ProtocolVersion(value[0], value[1]));
        }
        return List.copyOf(versions);
    }
}
