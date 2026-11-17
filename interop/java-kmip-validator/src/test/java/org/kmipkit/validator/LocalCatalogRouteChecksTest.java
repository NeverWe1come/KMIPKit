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
import static org.junit.jupiter.api.Assertions.assertTrue;

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
        Set<String> expectedIndividualCaseIds = new java.util.HashSet<>();
        catalog.path("test_cases").forEach(testCase -> {
            String targetId = testCase.path("requires").path("normative_target_id").asText();
            if (targetId.startsWith("KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-")) {
                expectedIndividualCaseIds.add(testCase.path("case_id").asText());
            }
        });

        LocalCatalogRouteChecks.executeItemTypeEnumerations(catalog, "2.1", invocations::add);

        assertEquals(88, invocations.size());
        assertEquals(Set.of(
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.enumeration",
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::routeId)
                        .collect(java.util.stream.Collectors.toSet()));
        assertEquals(11, invocations.stream().map(LocalCatalogRouteChecks.Invocation::targetId)
                .distinct().count());
        assertEquals(expectedIndividualCaseIds, invocations.stream()
                .map(LocalCatalogRouteChecks.Invocation::caseId)
                .filter(caseId -> !"test-enum-item-type".equals(caseId))
                .collect(java.util.stream.Collectors.toSet()));
        assertEquals(44, invocations.stream()
                .filter(invocation -> "test-enum-item-type".equals(invocation.caseId())).count());
        assertEquals(11, invocations.stream()
                .filter(invocation -> invocation.routeId().endsWith("ttlvvalue.enumeration")
                        && "test-enum-item-type".equals(invocation.caseId())).count());
        for (String caseId : expectedIndividualCaseIds) {
            assertEquals(4, invocations.stream().filter(invocation -> caseId.equals(invocation.caseId())).count());
        }
    }

    @Test
    void executesRepeatableObjectAttributeMemberThroughGenericTtlvRoutes() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        String caseId = "test-norm-structure-member-structure-member-5-1-any-attribute-in-4-object-attributes";
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        LocalCatalogRouteChecks.executeAnyObjectAttributeMembers(catalog, "2.1", Set.of(caseId),
                invocations::add);

        assertEquals(List.of(
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::routeId).toList());
        assertEquals(Set.of("KMIPKIT-ELEM-STRUCTURE-MEMBER-5-1-ANY-ATTRIBUTE-IN-4-OBJECT-ATTRIBUTES"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::targetId)
                        .collect(java.util.stream.Collectors.toSet()));
        invocations.forEach(invocation -> assertTrue(invocation.wire().length > 16));
    }

    @Test
    void executesScalarStructureMemberThroughGenericTtlvRoutes() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        String caseId = "test-norm-structure-member-structure-member-5-5-attribute-name";
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        LocalCatalogRouteChecks.executeScalarStructureMembers(catalog, "2.1", Set.of(caseId), invocations::add);

        assertEquals(List.of(
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.textstring",
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::routeId).toList());
        assertEquals(Set.of("KMIPKIT-ELEM-STRUCTURE-MEMBER-5-5-ATTRIBUTE-NAME"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::targetId)
                        .collect(java.util.stream.Collectors.toSet()));
        assertEquals(Set.of(caseId), invocations.stream().map(LocalCatalogRouteChecks.Invocation::caseId)
                .collect(java.util.stream.Collectors.toSet()));
        assertEquals(1, invocations.stream().map(LocalCatalogRouteChecks.Invocation::wire)
                .mapToInt(wire -> wire.length).distinct().count());
    }
}
