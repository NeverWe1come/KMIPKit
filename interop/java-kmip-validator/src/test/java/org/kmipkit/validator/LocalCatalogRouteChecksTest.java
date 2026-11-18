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
import static org.junit.jupiter.api.Assertions.assertFalse;
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
    void executesOperationTargetThroughGenericTtlvItemRoutes() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        String caseId = "test-op-create";
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        LocalCatalogRouteChecks.executeGenericOperationTargets(
                catalog, "2.1", Set.of(caseId), new ObjectMapper().createArrayNode(), invocations::add);

        assertEquals(List.of(
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::routeId).toList());
        assertEquals(Set.of("KMIPKIT-ELEM-OP-C2S-CREATE"), invocations.stream()
                .map(LocalCatalogRouteChecks.Invocation::targetId)
                .collect(java.util.stream.Collectors.toSet()));
        assertEquals(Set.of(caseId), invocations.stream().map(LocalCatalogRouteChecks.Invocation::caseId)
                .collect(java.util.stream.Collectors.toSet()));
        assertEquals(Set.of("construct", "decode"), invocations.stream()
                .map(LocalCatalogRouteChecks.Invocation::action)
                .collect(java.util.stream.Collectors.toSet()));
        assertEquals(1, invocations.stream().map(LocalCatalogRouteChecks.Invocation::wire)
                .mapToInt(wire -> wire.length).distinct().count());
        assertArrayEquals(new byte[] {0x42, 0x00, 0x5c, 0x05, 0, 0, 0, 4,
                0, 0, 0, 1, 0, 0, 0, 0}, invocations.get(0).wire());
    }

    @Test
    void selectsOperationCasesMissingGenericTtlvItemEvidence() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());

        Set<String> missing = LocalCatalogRouteChecks.missingGenericOperationTargetCaseIds(
                catalog, new ObjectMapper().createArrayNode(), "2.1");

        assertTrue(missing.contains("test-op-create"));
        assertTrue(missing.contains("test-obj-certificate"));
        assertFalse(missing.contains("test-enum-adjustment-type"),
                "A local enumeration's primary operation is context, not a separate Operation obligation.");
    }

    @Test
    void executesManagedObjectCaseThroughItsDeclaredOperationItemRoutes() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        LocalCatalogRouteChecks.executeGenericOperationTargets(catalog, "2.1",
                Set.of("test-obj-certificate"), new ObjectMapper().createArrayNode(), invocations::add);

        assertEquals(List.of(
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::routeId).toList());
        assertEquals(Set.of("KMIPKIT-ELEM-OP-C2S-REGISTER"), invocations.stream()
                .map(LocalCatalogRouteChecks.Invocation::targetId)
                .collect(java.util.stream.Collectors.toSet()));
        assertArrayEquals(new byte[] {0x42, 0x00, 0x5c, 0x05, 0, 0, 0, 4,
                0, 0, 0, 3, 0, 0, 0, 0}, invocations.get(0).wire());
    }

    @Test
    void executesBatchOperationObligationsWithTheirWorkflowIdentity() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        String caseId = "test-batch-create-activate-destroy";
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        LocalCatalogRouteChecks.executeGenericOperationTargets(catalog, "2.1", Set.of(caseId),
                new ObjectMapper().createArrayNode(), invocations::add);

        assertEquals(List.of(
                "KMIPKIT-ELEM-OP-C2S-CREATE",
                "KMIPKIT-ELEM-OP-C2S-ACTIVATE",
                "KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST",
                "KMIPKIT-ELEM-OP-C2S-REVOKE",
                "KMIPKIT-ELEM-OP-C2S-DESTROY"), invocations.stream()
                .filter(invocation -> "java.generic-ttlv.ttlv_item.create".equals(invocation.routeId()))
                .map(LocalCatalogRouteChecks.Invocation::targetId).toList());
        assertEquals(List.of(0, 1, 2, 3, 4), invocations.stream()
                .filter(invocation -> "java.generic-ttlv.ttlv_item.create".equals(invocation.routeId()))
                .map(LocalCatalogRouteChecks.Invocation::workflowStepIndex).toList());
        assertEquals(List.of("Create", "Activate", "Get Attribute List", "Revoke", "Destroy"),
                invocations.stream()
                        .filter(invocation -> "java.generic-ttlv.ttlv_item.create".equals(invocation.routeId()))
                        .map(LocalCatalogRouteChecks.Invocation::workflowOperation).toList());
        assertEquals(15, invocations.size());
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
    void executesCataloguedObjectStructureThroughGenericTtlvRoutes() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        String caseId = "test-norm-object-structure-object-structure-certificate";
        String targetId = "KMIPKIT-ELEM-OBJECT-STRUCTURE-CERTIFICATE";
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        LocalCatalogRouteChecks.executeGenericStructureTargets(
                catalog, "2.1", Set.of(caseId), Set.of(targetId), new ObjectMapper().createArrayNode(),
                invocations::add);

        assertEquals(List.of(
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvstructure.create",
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvstructure.withitem",
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.structure",
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::routeId).toList());
        assertEquals(Set.of(targetId), invocations.stream().map(LocalCatalogRouteChecks.Invocation::targetId)
                .collect(java.util.stream.Collectors.toSet()));
        assertEquals(Set.of(caseId), invocations.stream().map(LocalCatalogRouteChecks.Invocation::caseId)
                .collect(java.util.stream.Collectors.toSet()));
        byte[] wire = invocations.get(0).wire();
        assertEquals(1, invocations.stream().map(LocalCatalogRouteChecks.Invocation::wire)
                .mapToInt(value -> value.length).distinct().count());
        try (var decoded = TtlvCodec.decode(wire); var view = decoded.view();
                var tag = view.tag(); var value = view.value(); var structure = value.structure()) {
            assertEquals(0x420013, tag.raw());
            assertEquals(TtlvItemType.Structure, view.itemType());
            assertEquals(1, structure.itemCount());
        }
    }

    @Test
    void structureRouteSelectorIgnoresCatalogCasesWithoutNormativeTargets() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        JsonNode events = new ObjectMapper().createArrayNode();

        Set<String> missing = LocalCatalogRouteChecks.missingGenericStructureTargetCaseIds(
                catalog, events, "2.1", Set.of("KMIPKIT-ELEM-OBJECT-STRUCTURE-CERTIFICATE"));

        assertTrue(missing.contains("test-norm-object-structure-object-structure-certificate"));
    }

    @Test
    void structureRouteSelectorReusesIndependentGenericItemEvidence() throws Exception {
        ObjectMapper mapper = new ObjectMapper();
        JsonNode catalog = mapper.readTree(CATALOG.toFile());
        var events = mapper.createArrayNode();
        String caseId = "test-norm-operation-structure-operation-structure-7-15-interop-function";
        String targetId = "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-15-INTEROP-FUNCTION";
        for (String route : List.of("java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item", "java.generic-ttlv.ttlv_codec.decode")) {
            var event = events.addObject();
            event.put("case_id", caseId);
            event.put("target_id", targetId);
            event.put("route_id", route);
            event.put("protocol_version", "2.1");
            event.put("encoding", "TTLV");
            event.putNull("transport");
        }

        Set<String> missing = LocalCatalogRouteChecks.missingGenericStructureTargetCaseIds(
                catalog, events, "2.1", Set.of(targetId));

        assertTrue(missing.contains(caseId));
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();
        LocalCatalogRouteChecks.executeGenericStructureTargets(
                catalog, "2.1", Set.of(caseId), Set.of(targetId), events, invocations::add);
        assertEquals(List.of(
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvstructure.create",
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvstructure.withitem"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::routeId).toList());
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
    void executesEnumerationAttributesThroughGenericTtlvRoutes() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        Set<String> caseIds = Set.of(
                "test-attr-certificate-type",
                "test-attr-cryptographic-algorithm",
                "test-attr-digital-signature-algorithm",
                "test-attr-key-format-type",
                "test-attr-nist-key-type",
                "test-attr-object-type",
                "test-attr-opaque-data-type",
                "test-attr-protection-level",
                "test-attr-state");
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        for (String version : List.of("2.0", "2.1")) {
            LocalCatalogRouteChecks.executeEnumerationAttributeValues(catalog, version, caseIds, invocations::add);
        }

        assertEquals(caseIds, invocations.stream().map(LocalCatalogRouteChecks.Invocation::caseId)
                .collect(java.util.stream.Collectors.toSet()));
        assertEquals(Set.of(
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.enumeration",
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
    void executesScalarAttributesThroughGenericTtlvRoutes() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        Set<String> caseIds = Set.of(
                "test-attr-certificate-length",
                "test-attr-cryptographic-length",
                "test-attr-cryptographic-usage-mask",
                "test-attr-protection-storage-mask",
                "test-attr-rotate-generation",
                "test-attr-lease-time",
                "test-attr-protection-period",
                "test-attr-rotate-interval",
                "test-attr-rotate-offset",
                "test-attr-short-unique-identifier");
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        for (String version : List.of("2.0", "2.1")) {
            LocalCatalogRouteChecks.executeScalarAttributeValues(catalog, version, caseIds, invocations::add);
        }

        assertEquals(caseIds, invocations.stream().map(LocalCatalogRouteChecks.Invocation::caseId)
                .collect(java.util.stream.Collectors.toSet()));
        assertEquals(68, invocations.size(), "Every applicable Attribute/version pair must use four routes.");
        assertEquals(Set.of(
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.integer",
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.interval",
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.bytestring",
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::routeId)
                        .collect(java.util.stream.Collectors.toSet()));
        assertEquals(Set.of("construct", "decode"), invocations.stream()
                .map(LocalCatalogRouteChecks.Invocation::action)
                .collect(java.util.stream.Collectors.toSet()));
        assertTrue(invocations.stream().allMatch(invocation -> invocation.wire().length >= 16));
    }

    @Test
    void executesRemainingTextStringTargetsThroughGenericTtlvRoutes() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        Set<String> caseIds = Set.of(
                "test-attr-unique-identifier",
                "test-norm-structure-member-structure-member-3-3-encryption-key-information-unique-identifier",
                "test-norm-structure-member-structure-member-3-3-mac-signature-key-information-unique-identifier",
                "test-norm-structure-member-structure-member-7-18-attribute-name",
                "test-norm-structure-member-structure-member-7-21-unique-identifier",
                "test-norm-structure-member-structure-member-7-28-pkcs-11-interface",
                "test-norm-structure-member-structure-member-7-41-validation-profile");
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        for (String version : List.of("2.0", "2.1")) {
            LocalCatalogRouteChecks.executeTextStringTargets(catalog, version, caseIds, invocations::add);
        }

        assertEquals(caseIds, invocations.stream().map(LocalCatalogRouteChecks.Invocation::caseId)
                .collect(java.util.stream.Collectors.toSet()));
        assertEquals(56, invocations.size(), "Every applicable target/version pair must use four routes.");
        assertEquals(Set.of(
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.textstring",
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::routeId)
                        .collect(java.util.stream.Collectors.toSet()));
        assertEquals(Set.of("construct", "decode"), invocations.stream()
                .map(LocalCatalogRouteChecks.Invocation::action)
                .collect(java.util.stream.Collectors.toSet()));
        assertTrue(invocations.stream().allMatch(invocation -> invocation.wire().length >= 16));
    }

    @Test
    void executesRemainingByteStringAndDateTimeExtendedTargetsThroughGenericTtlvRoutes() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        Set<String> caseIds = Set.of(
                "test-norm-operation-structure-operation-structure-7-9-data",
                "test-norm-structure-member-structure-member-4-62-issuer-alternative-name",
                "test-norm-structure-member-structure-member-4-63-subject-alternative-name",
                "test-norm-structure-member-structure-member-7-27-pkcs-11-input-parameters",
                "test-norm-structure-member-structure-member-7-29-pkcs-11-output-parameters",
                "test-norm-structure-member-structure-member-9-11-hashed-password-timestamp");
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        for (String version : List.of("2.0", "2.1")) {
            LocalCatalogRouteChecks.executeRemainingFixedScalarTargets(catalog, version, caseIds, invocations::add);
        }

        assertEquals(caseIds, invocations.stream().map(LocalCatalogRouteChecks.Invocation::caseId)
                .collect(java.util.stream.Collectors.toSet()));
        assertEquals(48, invocations.size(), "Every applicable target/version pair must use four routes.");
        assertEquals(Set.of(
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.bytestring",
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.datetimeextended",
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::routeId)
                        .collect(java.util.stream.Collectors.toSet()));
        assertEquals(Set.of("construct", "decode"), invocations.stream()
                .map(LocalCatalogRouteChecks.Invocation::action)
                .collect(java.util.stream.Collectors.toSet()));
        assertTrue(invocations.stream().allMatch(invocation -> invocation.wire().length >= 16));
    }

    @Test
    void executesBigIntegerKeyMaterialTargetsThroughGenericTtlvRoutes() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        Set<String> caseIds = Set.of(
                "test-norm-structure-member-structure-member-2-8-split-key-prime-field-size",
                "test-norm-structure-member-structure-member-3-10-key-material-g",
                "test-norm-structure-member-structure-member-3-10-key-material-j",
                "test-norm-structure-member-structure-member-3-10-key-material-p",
                "test-norm-structure-member-structure-member-3-10-key-material-q",
                "test-norm-structure-member-structure-member-3-10-key-material-y",
                "test-norm-structure-member-structure-member-3-11-key-material-d",
                "test-norm-structure-member-structure-member-3-5-key-material-g",
                "test-norm-structure-member-structure-member-3-5-key-material-p",
                "test-norm-structure-member-structure-member-3-5-key-material-q",
                "test-norm-structure-member-structure-member-3-5-key-material-x",
                "test-norm-structure-member-structure-member-3-6-key-material-g",
                "test-norm-structure-member-structure-member-3-6-key-material-p",
                "test-norm-structure-member-structure-member-3-6-key-material-q",
                "test-norm-structure-member-structure-member-3-6-key-material-y",
                "test-norm-structure-member-structure-member-3-7-key-material-crt-coefficient",
                "test-norm-structure-member-structure-member-3-7-key-material-modulus",
                "test-norm-structure-member-structure-member-3-7-key-material-p",
                "test-norm-structure-member-structure-member-3-7-key-material-prime-exponent-p",
                "test-norm-structure-member-structure-member-3-7-key-material-prime-exponent-q",
                "test-norm-structure-member-structure-member-3-7-key-material-private-exponent",
                "test-norm-structure-member-structure-member-3-7-key-material-public-exponent",
                "test-norm-structure-member-structure-member-3-7-key-material-q",
                "test-norm-structure-member-structure-member-3-8-key-material-modulus",
                "test-norm-structure-member-structure-member-3-8-key-material-public-exponent",
                "test-norm-structure-member-structure-member-3-9-key-material-g",
                "test-norm-structure-member-structure-member-3-9-key-material-j",
                "test-norm-structure-member-structure-member-3-9-key-material-p",
                "test-norm-structure-member-structure-member-3-9-key-material-q",
                "test-norm-structure-member-structure-member-3-9-key-material-x");
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        for (String version : List.of("2.0", "2.1")) {
            LocalCatalogRouteChecks.executeBigIntegerTargets(catalog, version, caseIds, invocations::add);
        }

        assertEquals(caseIds, invocations.stream().map(LocalCatalogRouteChecks.Invocation::caseId)
                .collect(java.util.stream.Collectors.toSet()));
        assertEquals(240, invocations.size(), "Every Big Integer target/version pair must use four routes.");
        assertEquals(Set.of(
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.biginteger",
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
    void executesRsaPublicKeyMaterialThroughTypedModelRoutes() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        Set<String> caseIds = Set.of(
                "test-norm-structure-member-structure-member-3-8-key-material-modulus",
                "test-norm-structure-member-structure-member-3-8-key-material-public-exponent");
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        for (String version : List.of("2.0", "2.1")) {
            LocalCatalogRouteChecks.executeTransparentKeyMaterialTargets(
                    catalog, version, caseIds, invocations::add);
        }

        assertEquals(caseIds.size() * 2 * 2, invocations.size());
        assertEquals(Set.of(
                "java.typed-protocol.model.org.kmipkit.ttlv.keymaterial.fromfields",
                "java.typed-protocol.model.org.kmipkit.ttlv.keymaterial.fromitem"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::routeId)
                        .collect(java.util.stream.Collectors.toSet()));
        assertEquals(caseIds, invocations.stream().map(LocalCatalogRouteChecks.Invocation::caseId)
                .collect(java.util.stream.Collectors.toSet()));
        assertEquals(Set.of("construct"), invocations.stream()
                .map(LocalCatalogRouteChecks.Invocation::action)
                .collect(java.util.stream.Collectors.toSet()));
    }

    @Test
    void executesEveryTransparentKeyMaterialFormatThroughTypedModelRoutes() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        Set<String> caseIds = LocalCatalogRouteChecks.BIG_INTEGER_TARGET_CASE_IDS.stream()
                .filter(caseId -> caseId.contains("-key-material-"))
                .collect(java.util.stream.Collectors.toSet());
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        for (String version : List.of("2.0", "2.1")) {
            LocalCatalogRouteChecks.executeTransparentKeyMaterialTargets(
                    catalog, version, caseIds, invocations::add);
        }

        assertEquals(caseIds.size() * 2 * 2, invocations.size());
        assertEquals(Set.of(
                "java.typed-protocol.model.org.kmipkit.ttlv.keymaterial.fromfields",
                "java.typed-protocol.model.org.kmipkit.ttlv.keymaterial.fromitem"),
                invocations.stream().map(LocalCatalogRouteChecks.Invocation::routeId)
                        .collect(java.util.stream.Collectors.toSet()));
        assertEquals(caseIds, invocations.stream().map(LocalCatalogRouteChecks.Invocation::caseId)
                .collect(java.util.stream.Collectors.toSet()));
        assertEquals(Set.of("construct"), invocations.stream()
                .map(LocalCatalogRouteChecks.Invocation::action)
                .collect(java.util.stream.Collectors.toSet()));
    }

    @Test
    void executesRemainingEnumerationFieldsThroughGenericTtlvRoutes() throws Exception {
        JsonNode catalog = new ObjectMapper().readTree(CATALOG.toFile());
        Set<String> caseIds = Set.of(
                "test-norm-operation-structure-operation-structure-7-15-interop-function",
                "test-norm-structure-member-structure-member-5-5-attribute-reference",
                "test-norm-structure-member-structure-member-7-13-extension-type");
        List<LocalCatalogRouteChecks.Invocation> invocations = new ArrayList<>();

        for (String version : List.of("2.0", "2.1")) {
            LocalCatalogRouteChecks.executeRemainingEnumerationFields(catalog, version, caseIds, invocations::add);
        }

        assertEquals(caseIds, invocations.stream().map(LocalCatalogRouteChecks.Invocation::caseId)
                .collect(java.util.stream.Collectors.toSet()));
        assertEquals(24, invocations.size(), "Every Enumeration field/version pair must use four routes.");
        assertEquals(Set.of(
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.enumeration",
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
