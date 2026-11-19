package org.kmipkit.validator;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.jupiter.api.Test;
import org.kmipkit.protocol.KmipValues;

import java.nio.file.Path;
import java.util.HashSet;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;

class EndpointRoleRouteContractTest {
    private static final Path ROUTE_CAPABILITY_MANIFEST =
            Path.of("../shared/coverage/route-capabilities.json");
    private static final String ENDPOINT_ROLE_ENUMERATION_ROUTE =
            "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.enumeration";

    @Test
    void endpointRoleValueTargetsDoNotClaimUnrelatedResponseAccessors() throws Exception {
        JsonNode manifest = new ObjectMapper().readTree(ROUTE_CAPABILITY_MANIFEST.toFile());
        assertEquals(Set.of(ENDPOINT_ROLE_ENUMERATION_ROUTE), routeIdsFor(manifest,
                "KMIPKIT-ELEM-ENUM-VALUE-ENDPOINT-ROLE-CLIENT-00000001"));
        assertEquals(Set.of(ENDPOINT_ROLE_ENUMERATION_ROUTE), routeIdsFor(manifest,
                "KMIPKIT-ELEM-ENUM-VALUE-ENDPOINT-ROLE-SERVER-00000002"));
    }

    @Test
    void installedEndpointRoleNamedConstantsMatchTheirCatalogValues() throws Exception {
        JsonNode manifest = new ObjectMapper().readTree(ROUTE_CAPABILITY_MANIFEST.toFile());
        assertNamedConstant(manifest, "KMIPKIT-ELEM-ENUM-VALUE-ENDPOINT-ROLE-CLIENT-00000001",
                "org.kmipkit.protocol.KmipValues.EndpointRole.CLIENT", KmipValues.EndpointRole.CLIENT,
                "Client");
        assertNamedConstant(manifest, "KMIPKIT-ELEM-ENUM-VALUE-ENDPOINT-ROLE-SERVER-00000002",
                "org.kmipkit.protocol.KmipValues.EndpointRole.SERVER", KmipValues.EndpointRole.SERVER,
                "Server");
    }

    private static Set<String> routeIdsFor(JsonNode manifest, String targetId) {
        Set<String> routeIds = new HashSet<>();
        for (JsonNode route : manifest.path("routes")) {
            if (!"java".equals(route.path("language").asText())) continue;
            for (JsonNode target : route.path("normative_target_ids")) {
                if (targetId.equals(target.asText())) routeIds.add(route.path("route_id").asText());
            }
        }
        return Set.copyOf(routeIds);
    }

    private static void assertNamedConstant(JsonNode manifest, String targetId,
            String expectedApiSymbol, long installedValue, String expectedName) {
        for (JsonNode value : manifest.path("value_symbols")) {
            if (!"java".equals(value.path("language").asText())
                    || !targetId.equals(value.path("target_id").asText())) continue;
            assertEquals("named-constant", value.path("availability").asText());
            assertEquals(expectedApiSymbol, value.path("public_api_symbol").asText());
            assertEquals(value.path("wire_value").asLong(), installedValue);
            assertEquals(expectedName, KmipValues.EndpointRole.name(installedValue));
            return;
        }
        throw new AssertionError("The Java route manifest is missing a named Endpoint Role constant.");
    }
}
