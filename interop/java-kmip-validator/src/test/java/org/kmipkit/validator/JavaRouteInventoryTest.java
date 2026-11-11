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
                .filter(testCase -> "server_exchange".equals(testCase.execution().mode()))
                .map(CatalogLoader.CatalogCase::operation)
                .collect(Collectors.toSet());

        assertEquals(catalogOperations, inventory.keySet());
        assertEquals(catalogOperations.size(), inventory.size());
        assertEquals(JavaRouteInventory.Status.EXECUTABLE_TYPED, inventory.get("Query").status());
        assertEquals(JavaRouteInventory.Status.EXECUTABLE_TYPED, inventory.get("Log").status());
        for (String operation : TypedRequestFactory.availableFactories().keySet()) {
            assertEquals(JavaRouteInventory.Status.EXECUTABLE_TYPED,
                    inventory.get(operation).status(), operation);
        }
        assertTrue(inventory.get("Log").reason().contains("server log"));
        assertTrue(inventory.get("Log").reason().contains("mutation"));
        assertEquals(JavaRouteInventory.Status.EXECUTABLE_TYPED, inventory.get("Get Attributes").status());
        assertEquals(JavaRouteInventory.Status.EXECUTABLE_TYPED, inventory.get("Get Attribute List").status());
        assertEquals(JavaRouteInventory.Status.EXECUTABLE_TYPED, inventory.get("Get Constraints").status());
        assertEquals(JavaRouteInventory.Status.EXECUTABLE_TYPED, inventory.get("Create").status());
        assertEquals(JavaRouteInventory.Status.PAYLOAD_ONLY, inventory.get("Cancel").status());
        assertEquals(JavaRouteInventory.Status.NO_PUBLIC_ROUTE, inventory.get("Set Endpoint Role").status());

        var clientQuery = catalog.cases().stream()
                .filter(testCase -> testCase.caseId().equals("test-op-query"))
                .findFirst().orElseThrow();
        var serverQuery = catalog.cases().stream()
                .filter(testCase -> testCase.caseId().equals("test-s2c-query"))
                .findFirst().orElseThrow();
        assertEquals(JavaRouteInventory.Status.EXECUTABLE_TYPED,
                JavaRouteInventory.routeFor(clientQuery).status());
        assertEquals(JavaRouteInventory.Status.WORKFLOW_UNAVAILABLE,
                JavaRouteInventory.routeFor(serverQuery).status());
        assertTrue(JavaRouteInventory.routeFor(serverQuery).reason().contains("server-initiated"));

        for (String caseId : Set.of("test-op-get-attribute-list", "test-op-get-constraints")) {
            var statefulCase = catalog.cases().stream()
                    .filter(testCase -> testCase.caseId().equals(caseId))
                    .findFirst().orElseThrow();
            assertEquals(JavaRouteInventory.Status.EXECUTABLE_TYPED,
                    JavaRouteInventory.routeFor(statefulCase).status(), caseId);
            assertTrue(JavaRouteInventory.routeFor(statefulCase).reason().contains("typed response"), caseId);
        }

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
