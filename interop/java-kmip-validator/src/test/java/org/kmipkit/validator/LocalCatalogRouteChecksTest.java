package org.kmipkit.validator;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.kmipkit.ttlv.TtlvCodec;
import org.kmipkit.ttlv.TtlvItemType;
import org.junit.jupiter.api.Test;

import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertArrayEquals;

class LocalCatalogRouteChecksTest {
    private static final Path CATALOG = Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json");

    @Test
    void executesExactBooleanFieldTagAcrossGenericTtlvRoutes() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        String caseId = "test-norm-message-field-message-field-9-13-criticality-indicator";
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        LocalCatalogRouteChecks.executeBooleanFields(catalog, "2.1", Set.of(caseId), invocations::add);

        assertEquals(List.of(
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.boolean",
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::routeId).toList());
        assertEquals(Set.of("KMIPKIT-ELEM-MESSAGE-FIELD-9-13-CRITICALITY-INDICATOR"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::targetId)
                        .collect(java.util.stream.Collectors.toSet()));
        assertEquals(Set.of(caseId), invocations.stream().map(LocalCatalogRouteChecks.Invocation::caseId)
                .collect(java.util.stream.Collectors.toSet()));
        assertEquals(Set.of("construct", "decode"), invocations.stream()
                .map(LocalCatalogRouteChecks.Invocation::action)
                .collect(java.util.stream.Collectors.toSet()));
        assertEquals(1, invocations.stream().map(LocalCatalogRouteChecks.Invocation::wire)
                .mapToInt(wire -> wire.length).distinct().count());
        byte[] expected = new byte[] {0x42, 0x00, 0x26, 0x06, 0, 0, 0, 8,
                0, 0, 0, 0, 0, 0, 0, 1};
        invocations.forEach(invocation -> assertArrayEquals(expected, invocation.wire()));
        try (var decoded = TtlvCodec.decode(expected); var view = decoded.view();
                var tag = view.tag(); var value = view.value()) {
            assertEquals(0x420026, tag.raw());
            assertEquals(TtlvItemType.Boolean, view.itemType());
            assertEquals(true, value.booleanValue());
        }
    }

    @Test
    void executesRandomIvThroughTheTypedCryptographicParametersBuilder() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        String caseId = "test-norm-structure-member-structure-member-4-16-random-iv";
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        LocalCatalogRouteChecks.executeBooleanFields(catalog, "2.1", Set.of(caseId), invocations::add);

        assertEquals(Set.of(
                "java.typed-protocol.model.org.kmipkit.protocol.cryptographicparameters.builder",
                "java.typed-protocol.model.org.kmipkit.protocol.cryptographicparameters.builder.randomiv",
                "java.typed-protocol.model.org.kmipkit.protocol.cryptographicparameters.builder.build",
                "java.typed-protocol.model.org.kmipkit.protocol.cryptographicparameters.fromttlv"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::routeId)
                        .filter(routeId -> routeId.contains("cryptographicparameters"))
                        .collect(java.util.stream.Collectors.toSet()));
        assertEquals(Set.of(
                "java.typed-protocol.model.org.kmipkit.ttlv.typedattribute.create",
                "java.typed-protocol.model.org.kmipkit.ttlv.typedattribute.fromitem",
                "java.typed-protocol.model.org.kmipkit.extensions.attributeset.trynew"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::routeId)
                        .filter(routeId -> routeId.contains("typedattribute")
                                || routeId.contains("attributeset"))
                        .collect(java.util.stream.Collectors.toSet()));
    }

    @Test
    void executesEveryItemTypeEnumerationThroughGenericTtlvRoutes() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        LocalCatalogRouteChecks.executeItemTypeEnumerations(catalog, "2.1", invocations::add);

        assertEquals(44, invocations.size());
        assertEquals(Set.of(
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.enumeration",
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::routeId)
                        .collect(java.util.stream.Collectors.toSet()));
        assertEquals(11, invocations.stream().map(LocalCatalogRouteChecks.Invocation::targetId)
                .distinct().count());
        assertEquals(Set.of("test-enum-item-type"), invocations.stream()
                .map(LocalCatalogRouteChecks.Invocation::caseId)
                .collect(java.util.stream.Collectors.toSet()));
        assertEquals(11, invocations.stream()
                .filter(invocation -> invocation.routeId().endsWith("ttlvvalue.enumeration")).count());
    }
}
