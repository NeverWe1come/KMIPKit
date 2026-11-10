package org.kmipkit.validator;

import org.junit.jupiter.api.Test;

import java.nio.file.Path;
import java.util.Set;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class JavaRouteInventoryTest {
    @Test
    void classifiesEveryCatalogOperationAndExplainsPendingRoutes() throws Exception {
        var catalog = CatalogLoader.load(Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        var inventory = JavaRouteInventory.classify(catalog);
        var catalogOperations = catalog.cases().stream()
                .map(CatalogLoader.CatalogCase::operation)
                .collect(Collectors.toSet());

        assertEquals(catalogOperations, inventory.keySet());
        assertEquals(59, inventory.size());
        assertEquals(JavaRouteInventory.Status.EXECUTABLE_TYPED, inventory.get("Query").status());
        assertEquals(JavaRouteInventory.Status.TYPED_API_PENDING, inventory.get("Get Attributes").status());
        assertEquals(JavaRouteInventory.Status.PAYLOAD_ONLY, inventory.get("Cancel").status());
        assertEquals(JavaRouteInventory.Status.NO_PUBLIC_ROUTE, inventory.get("Set Endpoint Role").status());

        for (var route : inventory.values()) {
            assertFalse(route.reason().isBlank(), route.operation());
            assertTrue(route.status() != JavaRouteInventory.Status.UNCLASSIFIED, route.operation());
        }
        assertEquals(Set.of("Cancel", "Poll", "Process", "Query Asynchronous Requests"),
                inventory.values().stream()
                        .filter(route -> route.status() == JavaRouteInventory.Status.PAYLOAD_ONLY)
                        .map(JavaRouteInventory.Route::operation)
                        .collect(Collectors.toSet()));
    }
}
