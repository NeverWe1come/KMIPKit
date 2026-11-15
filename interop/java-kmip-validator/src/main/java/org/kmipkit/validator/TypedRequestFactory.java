package org.kmipkit.validator;

import org.kmipkit.extensions.ClientBatchItem;

import java.nio.charset.StandardCharsets;

/** Constructs deterministic public Java requests for the operations currently executed by the driver. */
final class TypedRequestFactory {
    private static final String LOG_MESSAGE = "KMIPKit interoperability diagnostic";

    private TypedRequestFactory() { }

    static ClientBatchItem create(String operation) {
        ClientBatchItem request = switch (operation) {
            case "Discover Versions" -> ClientBatchItem.discoverVersions();
            case "Ping" -> ClientBatchItem.ping();
            case "Query" -> ClientBatchItem.query(new int[] {1}, null);
            case "Log" -> ClientBatchItem.log(LOG_MESSAGE.getBytes(StandardCharsets.UTF_8));
            case "Get Constraints" -> ClientBatchItem.getConstraints();
            case "Get Attribute List" -> ClientBatchItem.getAttributeList();
            default -> null;
        };
        return request == null ? null : request.withUniqueBatchItemId(
                "item-0".getBytes(StandardCharsets.US_ASCII));
    }
}
