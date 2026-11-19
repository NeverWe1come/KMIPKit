package org.kmipkit.validator;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.jupiter.api.Test;

import java.nio.file.Path;
import java.util.HashSet;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;

class BatchResponseRouteContractTest {
    private static final Path ROUTE_CAPABILITY_MANIFEST =
            Path.of("../shared/coverage/route-capabilities.json");

    @Test
    void responseBatchCountAccessorTargetsOnlyTheNormativeCountFields() throws Exception {
        JsonNode manifest = new ObjectMapper().readTree(ROUTE_CAPABILITY_MANIFEST.toFile());

        assertEquals(Set.of(
                        "KMIPKIT-ELEM-MESSAGE-FIELD-8-5-BATCH-COUNT",
                        "KMIPKIT-ELEM-MESSAGE-FIELD-9-5-BATCH-COUNT"),
                targetsFor(manifest,
                        "java.typed-protocol.function.client_batch_response_item_count"));
    }

    @Test
    void responseBatchItemAccessorTargetsOnlyTheNormativeItemFields() throws Exception {
        JsonNode manifest = new ObjectMapper().readTree(ROUTE_CAPABILITY_MANIFEST.toFile());

        assertEquals(Set.of(
                        "KMIPKIT-ELEM-MESSAGE-FIELD-8-6-BATCH-ITEM",
                        "KMIPKIT-ELEM-MESSAGE-FIELD-9-7-BATCH-ITEM"),
                targetsFor(manifest,
                        "java.typed-protocol.function.client_batch_response_item_at"));
    }

    private static Set<String> targetsFor(JsonNode manifest, String routeId) {
        for (JsonNode route : manifest.path("routes")) {
            if (!"java".equals(route.path("language").asText())
                    || !routeId.equals(route.path("route_id").asText())) continue;
            Set<String> targets = new HashSet<>();
            route.path("normative_target_ids").forEach(target -> targets.add(target.asText()));
            return Set.copyOf(targets);
        }
        throw new AssertionError("The Java route manifest is missing " + routeId);
    }
}
