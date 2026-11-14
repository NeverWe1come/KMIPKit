package org.kmipkit.validator;

import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.protocol.KmipValues;
import org.junit.jupiter.api.Test;

import java.util.Map;
import java.nio.file.Path;
import java.lang.reflect.Modifier;
import java.util.Arrays;
import java.util.Set;
import java.util.function.LongSupplier;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

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

    @Test
    void constructsEveryPublicClientOperationFactoryInTheSharedCatalog() throws Exception {
        var catalog = CatalogLoader.load(Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        var routes = JavaRouteInventory.classify(catalog);
        var unavailable = java.util.Set.of("Cancel", "Notify", "Poll", "Process",
                "Query Asynchronous Requests", "Put", "Set Endpoint Role");

        for (String operation : routes.keySet()) {
            if (unavailable.contains(operation)) continue;
            try {
                try (ClientBatchItem request = TypedRequestFactory.create(operation)) {
                    assertEquals(normalize(operation), normalize(KmipValues.Operation.name(request.operation())),
                            operation);
                }
            } catch (Throwable failure) {
                throw new AssertionError("Request construction failed for " + operation, failure);
            }
        }
    }

    @Test
    void exercisesEveryPublicClientBatchItemFactoryOverload() throws Exception {
        var catalog = CatalogLoader.load(Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        Set<String> catalogOperations = JavaRouteInventory.classify(catalog).keySet().stream()
                .map(TypedRequestFactoryTest::normalize).collect(java.util.stream.Collectors.toSet());
        Set<String> publicFactories = Arrays.stream(ClientBatchItem.class.getMethods())
                .filter(method -> method.getReturnType() == ClientBatchItem.class
                        && Modifier.isStatic(method.getModifiers())
                        && !Set.of("createGroup", "obliterate", "createUser", "createCredential", "deactivate",
                                "encapsulate", "decapsulate").contains(method.getName()))
                .map(java.lang.reflect.Method::toGenericString)
                .collect(java.util.stream.Collectors.toSet());

        Map<String, String> exercised = TypedRequestFactory.constructEveryFactoryVariant();

        assertEquals(publicFactories, exercised.keySet());
        for (String operationName : exercised.values()) {
            assertTrue(catalogOperations.contains(normalize(operationName)), operationName);
        }
    }

    @Test
    void everyExecutableOperationHasAnInstalledTypedResponseReader() throws Exception {
        for (String operation : TypedRequestFactory.availableFactories().keySet()) {
            String accessor = TypedRequestFactory.responseAccessor(operation);
            assertTrue(Arrays.stream(org.kmipkit.extensions.ClientBatchItemResult.class.getMethods())
                    .anyMatch(method -> method.getName().equals(accessor)
                            && method.getParameterCount() == 0), operation + " -> " + accessor);
        }
    }

    private static String normalize(String value) {
        return value.toLowerCase(java.util.Locale.ROOT).replaceAll("[^a-z0-9]", "");
    }
}
