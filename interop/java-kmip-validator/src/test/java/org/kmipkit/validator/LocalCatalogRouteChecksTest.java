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
    void executesClientPayloadCorrelationFieldThroughGenericTtlvRoutes() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        String caseId = "test-norm-payload-client-cancel-request-21-001";
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        LocalCatalogRouteChecks.executeCorrelationPayloadScalarFields(
                catalog, "2.1", Set.of(caseId), invocations::add);

        assertEquals(List.of(
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode",
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::routeId).toList());
        assertEquals(Set.of(
                "KMIPKIT-PAYLOAD-CLIENT-CANCEL-REQUEST-21-001",
                "KMIPKIT-ELEM-OP-C2S-CANCEL"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::targetId)
                        .collect(java.util.stream.Collectors.toSet()));
        assertEquals(Set.of(caseId), invocations.stream().map(LocalCatalogRouteChecks.Invocation::caseId)
                .collect(java.util.stream.Collectors.toSet()));
        assertEquals(Set.of("construct", "decode"), invocations.stream()
                .map(LocalCatalogRouteChecks.Invocation::action)
                .collect(java.util.stream.Collectors.toSet()));
        assertEquals(1, invocations.stream().map(LocalCatalogRouteChecks.Invocation::wire)
                .mapToInt(wire -> wire.length).distinct().count());
    }

    @Test
    void executesClientPayloadCorrelationResponseThroughGenericTtlvRoutes() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        String caseId = "test-norm-payload-client-cancel-response-21-001";
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        LocalCatalogRouteChecks.executeCorrelationPayloadScalarFields(
                catalog, "2.1", Set.of(caseId), invocations::add);

        assertEquals(6, invocations.size());
        assertEquals(Set.of(
                "KMIPKIT-PAYLOAD-CLIENT-CANCEL-RESPONSE-21-001",
                "KMIPKIT-ELEM-OP-C2S-CANCEL"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::targetId)
                        .collect(java.util.stream.Collectors.toSet()));
        assertEquals(Set.of("java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item", "java.generic-ttlv.ttlv_codec.decode"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::routeId)
                        .collect(java.util.stream.Collectors.toSet()));
    }

    @Test
    void executesAttributesStructureThroughGenericTtlvBuilderAndItemRoutes() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        String caseId = "test-norm-attribute-structure-attribute-structure-5-1-attributes";
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        LocalCatalogRouteChecks.executeAttributeStructures(catalog, "2.1", Set.of(caseId), invocations::add);

        assertEquals(List.of(
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvstructure.create",
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvstructure.withitem",
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::routeId).toList());
        assertEquals(Set.of(caseId), invocations.stream().map(LocalCatalogRouteChecks.Invocation::caseId)
                .collect(java.util.stream.Collectors.toSet()));
        assertEquals(Set.of("KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-1-ATTRIBUTES"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::targetId)
                        .collect(java.util.stream.Collectors.toSet()));
        assertEquals(1, invocations.stream().map(LocalCatalogRouteChecks.Invocation::wire)
                .mapToInt(wire -> wire.length).distinct().count());
    }

    @Test
    void executesSevenStructuredAttributeValuesThroughGenericTtlvRoutes() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        Set<String> caseIds = Set.of(
                "test-attr-alternative-name",
                "test-attr-application-specific-information",
                "test-attr-key-value-location",
                "test-attr-link",
                "test-attr-name",
                "test-attr-revocation-reason",
                "test-attr-rotate-name");
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        for (String version : List.of("2.0", "2.1")) {
            LocalCatalogRouteChecks.executeStructureAttributeValues(catalog, version, caseIds, invocations::add);
        }

        assertEquals((caseIds.size() * 2 - 1) * 4, invocations.size());
        assertEquals(Set.of(
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.structure",
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::routeId)
                        .collect(java.util.stream.Collectors.toSet()));
        assertEquals(caseIds, invocations.stream().map(LocalCatalogRouteChecks.Invocation::caseId)
                .collect(java.util.stream.Collectors.toSet()));
        assertEquals(Set.of("construct", "decode"), invocations.stream()
                .map(LocalCatalogRouteChecks.Invocation::action)
                .collect(java.util.stream.Collectors.toSet()));
    }

    @Test
    void executesFiveTextAttributesThroughGenericTtlvRoutes() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        Set<String> caseIds = Set.of(
                "test-attr-comment",
                "test-attr-contact-information",
                "test-attr-description",
                "test-attr-object-group",
                "test-attr-pkcs#12-friendly-name");
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        for (String version : List.of("2.0", "2.1")) {
            LocalCatalogRouteChecks.executeTextAttributeValues(catalog, version, caseIds, invocations::add);
        }

        assertEquals(caseIds.size() * 2 * 4, invocations.size());
        assertEquals(Set.of(
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.textstring",
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::routeId)
                        .collect(java.util.stream.Collectors.toSet()));
        assertEquals(caseIds, invocations.stream().map(LocalCatalogRouteChecks.Invocation::caseId)
                .collect(java.util.stream.Collectors.toSet()));
        assertEquals(Set.of("construct", "decode"), invocations.stream()
                .map(LocalCatalogRouteChecks.Invocation::action)
                .collect(java.util.stream.Collectors.toSet()));
    }

    @Test
    void executesBooleanAttributesThroughGenericTtlvRoutes() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        Set<String> caseIds = Set.of(
                "test-attr-always-sensitive",
                "test-attr-extractable",
                "test-attr-fresh",
                "test-attr-key-value-present",
                "test-attr-never-extractable",
                "test-attr-quantum-safe",
                "test-attr-rotate-automatic",
                "test-attr-rotate-latest",
                "test-attr-sensitive");
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        for (String version : List.of("2.0", "2.1")) {
            LocalCatalogRouteChecks.executeBooleanAttributeValues(catalog, version, caseIds, invocations::add);
        }

        assertEquals(caseIds, invocations.stream().map(LocalCatalogRouteChecks.Invocation::caseId)
                .collect(java.util.stream.Collectors.toSet()));
        assertEquals(Set.of(
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.boolean",
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::routeId)
                        .collect(java.util.stream.Collectors.toSet()));
        assertEquals(Set.of("construct", "decode"), invocations.stream()
                .map(LocalCatalogRouteChecks.Invocation::action)
                .collect(java.util.stream.Collectors.toSet()));
        assertTrue(invocations.stream().allMatch(invocation -> invocation.wire().length == 16));
    }

    @Test
    void executesDateTimeAttributesThroughGenericTtlvRoutes() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        Set<String> caseIds = Set.of(
                "test-attr-activation-date",
                "test-attr-archive-date",
                "test-attr-compromise-date",
                "test-attr-compromise-occurrence-date",
                "test-attr-deactivation-date",
                "test-attr-destroy-date",
                "test-attr-initial-date",
                "test-attr-last-change-date",
                "test-attr-original-creation-date",
                "test-attr-process-start-date",
                "test-attr-protect-stop-date",
                "test-attr-rotate-date");
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        for (String version : List.of("2.0", "2.1")) {
            LocalCatalogRouteChecks.executeDateTimeAttributeValues(catalog, version, caseIds, invocations::add);
        }

        assertEquals(caseIds, invocations.stream().map(LocalCatalogRouteChecks.Invocation::caseId)
                .collect(java.util.stream.Collectors.toSet()));
        assertEquals(Set.of(
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.datetime",
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::routeId)
                        .collect(java.util.stream.Collectors.toSet()));
        assertEquals(Set.of("construct", "decode"), invocations.stream()
                .map(LocalCatalogRouteChecks.Invocation::action)
                .collect(java.util.stream.Collectors.toSet()));
        assertTrue(invocations.stream().allMatch(invocation -> invocation.wire().length == 16));
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

    @Test
    void executesCertificateTypeStructureMemberWithAssignedEnumerationValue() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        String caseId = "test-norm-structure-member-structure-member-2-1-certificate-certificate-type";
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        LocalCatalogRouteChecks.executeScalarStructureMembers(catalog, "2.1", Set.of(caseId), invocations::add);

        assertEquals(4, invocations.size());
        try (var decoded = TtlvCodec.decode(invocations.get(0).wire());
                var view = decoded.view(); var value = view.value()) {
            assertEquals(TtlvItemType.Enumeration, view.itemType());
            assertEquals(1, value.enumeration(), "X.509 is the first assigned Certificate Type value.");
        }
    }

    @Test
    void executesServerQueryFunctionPayloadThroughGenericTtlvRoutes() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        String caseId = "test-norm-payload-server-query-request-20-001";
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        LocalCatalogRouteChecks.executeServerPayloadScalarFields(catalog, "2.0", Set.of(caseId), invocations::add);

        assertEquals(List.of(
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode",
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::routeId).toList());
        assertEquals(Set.of("KMIPKIT-PAYLOAD-SERVER-QUERY-REQUEST-20-001", "KMIPKIT-ELEM-OP-S2C-QUERY"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::targetId)
                        .collect(java.util.stream.Collectors.toSet()));
        byte[] expected = new byte[] {0x42, 0x00, 0x74, 0x05, 0, 0, 0, 4,
                0, 0, 0, 1, 0, 0, 0, 0};
        invocations.forEach(invocation -> assertArrayEquals(expected, invocation.wire()));
    }

    @Test
    void executesServerNotifyUniqueIdentifierThroughGenericTtlvRoutes() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        String caseId = "test-norm-payload-server-notify-request-20-001";
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        LocalCatalogRouteChecks.executeServerPayloadScalarFields(catalog, "2.0", Set.of(caseId),
                invocations::add);

        assertEquals(6, invocations.size());
        assertEquals(Set.of("KMIPKIT-PAYLOAD-SERVER-NOTIFY-REQUEST-20-001", "KMIPKIT-ELEM-OP-S2C-NOTIFY"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::targetId)
                        .collect(java.util.stream.Collectors.toSet()));
        assertEquals(Set.of(caseId), invocations.stream().map(LocalCatalogRouteChecks.Invocation::caseId)
                .collect(java.util.stream.Collectors.toSet()));
        assertEquals(1, invocations.stream().map(LocalCatalogRouteChecks.Invocation::wire)
                .mapToInt(wire -> wire.length).distinct().count());
    }
}
