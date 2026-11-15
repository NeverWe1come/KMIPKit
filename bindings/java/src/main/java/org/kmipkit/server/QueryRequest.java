package org.kmipkit.server;

import java.util.ArrayList;
import java.util.List;

/** Received Query request. */
public final class QueryRequest extends ServerRequest {
    QueryRequest(long handle, ServerRequestBatch owner) { super(handle, owner, 24); }

    public List<Integer> functions() {
        long count = ServerNative.queryFunctionCount(nativeHandle());
        List<Integer> result = new ArrayList<>((int) count);
        for (long index = 0; index < count; index++) {
            result.add(ServerNative.queryFunctionAt(nativeHandle(), index));
        }
        return List.copyOf(result);
    }
}
