package org.kmipkit.validator;

import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.protocol.KmipValues;
import org.junit.jupiter.api.Test;

import java.util.Map;
import java.util.function.LongSupplier;

import static org.junit.jupiter.api.Assertions.assertEquals;

class TypedRequestFactoryTest {
    @Test
    void buildsEachDriverRouteWithTheExpectedKmipOperation() {
        Map<String, LongSupplier> expected = Map.of(
                "Discover Versions", () -> 0x1e,
                "Ping", () -> 0x3b,
                "Query", () -> 0x18,
                "Log", () -> 0x2c,
                "Get Constraints", () -> KmipValues.V2_1_OPERATION_GET_CONSTRAINTS,
                "Get Attribute List", () -> 0x0c);

        expected.forEach((operation, expectedCode) -> {
            try (ClientBatchItem request = TypedRequestFactory.create(operation)) {
                assertEquals(expectedCode.getAsLong(), request.operation(), operation);
            }
        });
    }
}
