package org.kmipkit.server;

import java.util.List;
import org.kmipkit.InvalidInputException;

/** Typed Query response with explicitly selected capabilities. */
public final class QueryResponse extends ServerResponse {
    private QueryResponse(long handle, QueryRequest request) { super(handle, request); }

    public static QueryResponse forRequest(QueryRequest request, List<Integer> operations,
            List<Integer> objectTypes, List<Integer> extensionTags, String vendorIdentification) {
        if (request == null) throw new InvalidInputException("request is required");
        return new QueryResponse(ServerNative.responseQuery(request.nativeHandle(),
                values(operations), values(objectTypes), values(extensionTags), vendorIdentification), request);
    }

    private static int[] values(List<Integer> values) {
        if (values == null) return new int[0];
        int[] result = new int[values.size()];
        for (int index = 0; index < result.length; index++) {
            Integer value = values.get(index);
            if (value == null) throw new InvalidInputException("capability value is null");
            result[index] = value;
        }
        return result;
    }
}
