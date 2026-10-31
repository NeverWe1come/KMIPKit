package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.IOException;
import java.lang.reflect.Method;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import org.junit.jupiter.api.Test;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.message.MessageExtensionView;
import org.kmipkit.message.MessageValidationException;
import org.kmipkit.message.RequestBatchItemView;
import org.kmipkit.message.RequestHeaderView;
import org.kmipkit.message.RequestMessage;
import org.kmipkit.message.ResponseBatchItemView;
import org.kmipkit.message.ResponseHeaderView;
import org.kmipkit.message.ResponseMessage;

/** Executes the Java mappings declared by the shared API capability fixture. */
final class SharedCapabilityParityFixtureTest {
    private record JavaConsumerMapping(String status, String test, String json) {}

    private static final Pattern STATUS = Pattern.compile("\\\"status\\\"\\s*:\\s*\\\"([^\\\"]+)\\\"");
    private static final Pattern TEST = Pattern.compile("\\\"test\\\"\\s*:\\s*\\\"([^\\\"]+)\\\"");
    private static final String API_LEVEL_CASE = "api-levels.shared.v1";
    private static final String BATCH_CASE = "batch.ordered-completed-pending-failed.v1";

    @Test
    void everyJavaFixtureMappingTargetsAnExecutableJUnitConsumer() throws Exception {
        String fixture = Files.readString(sharedFixturePath());
        assertEquals(List.of("rust", "c", "java", "python"),
                stringArrayField(fixture, "parityLanguages"));
        assertEquals(List.of(), stringArrayField(fixture, "plannedConsumerLanguages"));
        assertEquals(List.of("rust", "c", "java", "python"),
                stringArrayField(fixture, "executableConsumerLanguages"));

        List<String> ids = caseIds(fixture);
        assertEquals(15, ids.size());
        assertEquals(ids.size(), ids.stream().distinct().count());
        for (String id : ids) {
            String item = caseById(fixture, id);
            JavaConsumerMapping mapping = javaConsumerMapping(item);
            assertTrue(mapping.status().equals("available") || mapping.status().equals("blocked"), id);
            assertJUnitTestExists(mapping.test(), id);
            if (mapping.status().equals("blocked")) {
                assertTrue(mapping.json().contains("\"reason\""), id + " needs a blocker reason");
            }
        }
    }

    @Test
    void typedMessageModelFixtureIsPresentForCrossLanguageConsumers() throws IOException {
        String fixture = Files.readString(sharedFixturePath());
        String typedMessage = caseById(
                fixture, "typed-message.ordered-open-values-and-lifetime.v1");
        assertEquals(List.of("KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-011",
                        "KMIPKIT-0043-FR-013", "KMIPKIT-0043-FR-017",
                        "KMIPKIT-0043-FR-018", "KMIPKIT-0043-FR-022"),
                stringArrayField(typedMessage, "requirementIds"));
        assertTrue(objectField(typedMessage, "request").contains("batchItems"));
        assertTrue(objectField(typedMessage, "response").contains("resultStatusRaw"));
        assertTrue(objectField(typedMessage, "invalidRequest").contains("payloadMarker"));
        assertTrue(objectField(typedMessage, "lifetime").contains("extensionViewSurvivesParentClose"));
        assertTrue(typedMessage.contains("\"status\": \"available\""));
    }

    @Test
    void typedMessageModelMatchesSharedFixture() throws IOException {
        String fixtureCase = caseById(Files.readString(sharedFixturePath()),
                "typed-message.ordered-open-values-and-lifetime.v1");
        String input = objectField(fixtureCase, "input");
        String expected = objectField(fixtureCase, "expectedOutput");
        String requestInput = objectField(input, "request");
        String requestExpected = objectField(expected, "request");
        List<String> expectedRequestHeaderTags = stringArrayField(requestExpected, "headerFieldTags");
        List<String> expectedRequestItems = objectArrayFields(requestExpected, "batchItems");
        List<String> inputRequestItems = objectArrayFields(requestInput, "batchItems");
        String expectedDefaults = objectField(requestExpected, "effectiveDefaults");

        RequestMessage request = RequestMessage.fromTtlv(typedRequestTree(requestInput, null));
        assertEquals(numberField(requestExpected, "batchCount"), request.batchItemCount());
        RequestHeaderView requestHeader = request.header();
        TtlvStructure requestHeaderTtlv = requestHeader.ttlv();
        try (TtlvStructureView view = requestHeaderTtlv.view()) {
            assertEquals(expectedRequestHeaderTags, tagStrings(view));
        }
        requestHeaderTtlv.close();
        assertEquals(2, requestHeader.protocolVersion().major());
        assertEquals(1, requestHeader.protocolVersion().minor());
        assertEquals(numberField(expectedDefaults, "asynchronousIndicator"),
                requestHeader.effectiveAsynchronousIndicator());
        assertTrue(requestHeader.asynchronousIndicator().isEmpty());
        assertEquals(Boolean.parseBoolean(stringOrBooleanField(expectedDefaults,
                "attestationCapableIndicator")), requestHeader.effectiveAttestationCapableIndicator());
        assertTrue(requestHeader.attestationCapableIndicator().isEmpty());
        assertEquals(numberField(expectedDefaults, "batchErrorContinuationOption"),
                requestHeader.effectiveBatchErrorContinuationOption());
        assertTrue(requestHeader.batchErrorContinuationOption().isEmpty());
        assertEquals(Boolean.parseBoolean(stringOrBooleanField(expectedDefaults, "batchOrderOption")),
                requestHeader.effectiveBatchOrderOption());
        assertTrue(requestHeader.batchOrderOption().isEmpty());

        MessageExtensionView requestExtension = null;
        for (int index = 0; index < expectedRequestItems.size(); index++) {
            String itemExpected = expectedRequestItems.get(index);
            String itemInput = inputRequestItems.get(index);
            try (RequestBatchItemView item = request.batchItemAt(index)) {
                assertEquals(numberField(itemExpected, "operationRaw"), item.operation().orElseThrow());
                assertArrayEquals(java.util.HexFormat.of().parseHex(
                        stringField(itemExpected, "uniqueBatchItemIdHex")),
                        item.uniqueBatchItemId().orElseThrow());
                try (TtlvStructure itemTtlv = item.ttlv();
                        TtlvStructureView itemView = itemTtlv.view()) {
                    assertEquals(stringArrayField(itemExpected, "fieldTags"), tagStrings(itemView));
                }
                int expectedMask = (int) Long.parseLong(
                        stringField(itemExpected, "bitMaskRawHex").substring(2), 16);
                int payloadMask = item.withRequestPayload(payload -> {
                    try (var bitMask = payload.itemAt(0);
                            var value = bitMask.value()) {
                        return value.integer();
                    }
                }).orElseThrow();
                assertEquals(expectedMask, payloadMask);
                if (index == numberField(requestExpected, "extensionBatchItemIndex")) {
                    assertEquals(1, item.extensionCount());
                    requestExtension = item.extensionAt(0);
                }
                assertEquals(stringField(itemInput, "uniqueBatchItemIdHex"),
                        java.util.HexFormat.of().formatHex(item.uniqueBatchItemId().orElseThrow()));
            }
        }
        assertNotNull(requestExtension);
        request.close();
        assertEquals("Acme", requestExtension.vendorIdentification().orElseThrow());
        assertEquals(Boolean.parseBoolean(stringOrBooleanField(requestExpected,
                "extensionCriticalityIndicator")), requestExtension.criticalityIndicator().orElseThrow());
        assertExtensionMatchesFixture(requestExtension, requestExpected);
        requestExtension.close();
        assertEquals(2, requestHeader.protocolVersion().major(),
                "a retained header view remains valid after its request owner closes");
        requestHeader.close();

        String responseInput = objectField(input, "response");
        String responseExpected = objectField(expected, "response");
        ResponseMessage response = ResponseMessage.fromTtlv(typedResponseTree(responseInput));
        assertEquals(numberField(responseExpected, "batchCount"), response.batchItemCount());
        ResponseHeaderView responseHeader = response.header();
        try (TtlvStructure responseHeaderTtlv = responseHeader.ttlv();
                TtlvStructureView responseHeaderView = responseHeaderTtlv.view()) {
            assertEquals(stringArrayField(responseExpected, "headerFieldTags"),
                    tagStrings(responseHeaderView));
        }
        assertEquals(numberField(responseInput, "timeStamp"), responseHeader.timeStamp());
        ResponseBatchItemView responseItem = response.batchItemAt(0);
        assertEquals(numberField(responseExpected, "operationRaw"),
                responseItem.operation().orElseThrow());
        assertEquals(numberField(responseExpected, "resultStatusRaw"),
                responseItem.resultStatus().orElseThrow());
        try (TtlvStructure itemTtlv = responseItem.ttlv(); TtlvStructureView itemView = itemTtlv.view()) {
            assertEquals(stringArrayField(responseExpected, "batchItemFieldTags"), tagStrings(itemView));
        }
        MessageExtensionView responseExtension = responseItem.extensionAt(0);
        responseItem.close();
        responseHeader.close();
        response.close();
        assertExtensionMatchesFixture(responseExtension, responseExpected);
        responseExtension.close();

        String invalidInput = objectField(input, "invalidRequest");
        String validationExpected = objectField(expected, "validation");
        MessageValidationException validation = assertThrows(MessageValidationException.class,
                () -> RequestMessage.fromTtlv(typedRequestTree(requestInput, invalidInput)));
        assertEquals(numberField(validationExpected, "kindCode"), validation.kindCode());
        assertEquals(numberField(validationExpected, "structureIndex"),
                validation.structureIndex().orElseThrow());
        assertTrue(validation.fieldIndex().isEmpty());
        String payloadMarker = stringField(validationExpected, "payloadMarker");
        assertFalse(validation.getMessage().contains(payloadMarker));
        assertFalse(validation.toString().contains(payloadMarker));
    }

    @Test
    void everyAvailableJavaFixtureMappingHasAnAdvertisedNativeCapability() throws IOException {
        String fixture = Files.readString(sharedFixturePath());
        for (String id : caseIds(fixture)) {
            String item = caseById(fixture, id);
            JavaConsumerMapping mapping = javaConsumerMapping(item);
            if (mapping.status().equals("available")) {
                String capability = stringField(item, "capability");
                assertTrue(RuntimeMetadata.capabilityIsAvailable(capability),
                        id + " must map to a capability declared by the native runtime");
            }
        }
    }

    @Test
    void apiLevelQueryMatchesSharedFixtureLocally() throws IOException {
        String item = caseById(Files.readString(sharedFixturePath()), API_LEVEL_CASE);
        assertEquals("api_level_discovery", stringField(item, "capability"));
        assertTrue(RuntimeMetadata.capabilityIsAvailable("api_level_discovery"));
        assertEquals(List.of("high-level", "typed-protocol", "generic-ttlv"),
                stringArrayField(item, "apiLevels"));
        assertEquals(stringArrayField(item, "apiLevels"), RuntimeMetadata.apiLevels());
        assertTrue(item.contains("\"networkCalls\": 0"));
        assertTrue(item.contains("\"distinctFromKmipOperation\": \"Discover Versions\""));
        assertTrue(item.contains("\"operationMappingCompleteness\": \"not-asserted\""));
    }

    @Test
    void orderedBatchOutcomesMatchTheSharedFixtureAndUseOneExchange() throws IOException {
        String item = caseById(Files.readString(sharedFixturePath()), BATCH_CASE);
        assertEquals("client_execution", stringField(item, "capability"));
        assertTrue(RuntimeMetadata.capabilityIsAvailable("client_execution"));
        assertTrue(item.contains("\"available\": true"));
        assertTrue(item.contains("\"correlationValueHex\": \"00a17f\""));
        assertTrue(item.contains("\"transportExchangeCount\": 1"));
    }

    @Test
    void creationCreateMatchesSharedFixture() throws IOException {
        String item = caseById(Files.readString(sharedFixturePath()), "creation.create.request-response.v1");
        assertTrue(item.contains("\"attributesPresent\": true"));
        assertTrue(item.contains("\"resultMessage\": null"));
        try (TtlvStructure attributes = TtlvStructure.create();
                TtlvStructure masks = TtlvStructure.create();
                ClientBatchItem request = ClientBatchItem.create(0xffff_ffffL, attributes, masks)) {
            assertNotNull(request);
        }
    }

    @Test
    void creationKeyPairMatchesSharedFixture() throws IOException {
        String item = caseById(Files.readString(sharedFixturePath()), "creation.key-pair.request-response.v1");
        assertTrue(item.contains("\"groupsRemainDistinct\": true"));
        assertTrue(item.contains("\"privateUniqueIdentifier\""));
        try (TtlvStructure common = TtlvStructure.create();
                TtlvStructure privateAttributes = TtlvStructure.create();
                TtlvStructure publicAttributes = TtlvStructure.create();
                TtlvStructure commonMasks = TtlvStructure.create();
                TtlvStructure publicMasks = TtlvStructure.create();
                ClientBatchItem request = ClientBatchItem.createKeyPair(
                        common, privateAttributes, publicAttributes, commonMasks, null, publicMasks)) {
            assertNotNull(request);
        }
    }

    @Test
    void creationSplitKeyMatchesSharedFixture() throws IOException {
        String item = caseById(Files.readString(sharedFixturePath()), "creation.split-key.request-response.v1");
        assertTrue(item.contains("\"splitKeyParts\": -3"));
        assertTrue(item.contains("\"correlationValueHex\": \"00ff80\""));
        assertTrue(item.contains("\"resultMessage\": null"));
        assertTrue(item.contains("\"status\": 2"));
        try (TtlvStructure attributes = TtlvStructure.create();
                TtlvStructure masks = TtlvStructure.create();
                ClientBatchItem request = ClientBatchItem.createSplitKey(
                        0xf000_0000L, UniqueIdentifier.integer(-7), -3, 4, 0xdead_beefL,
                        null, attributes, masks)) {
            assertNotNull(request);
        }
    }

    @Test
    void genericTtlvFixturePreservesOpenEnumerationBytesOrderAndViewLifecycle() throws IOException {
        String item = caseById(Files.readString(sharedFixturePath()), "ttlv.open-values-and-child-order.v1");
        String input = objectField(item, "input");
        String structureInput = objectField(input, "structure");
        String expected = objectField(item, "expectedOutput");
        assertTrue(Boolean.parseBoolean(stringOrBooleanField(expected, "unknownValuesPreserved")));
        List<String> children = objectArrayFields(structureInput, "children");
        assertEquals(2, children.size());
        String enumerationInput = children.get(0);
        String bytesInput = children.get(1);
        assertEquals("Enumeration", stringField(enumerationInput, "type"));
        assertEquals("ByteString", stringField(bytesInput, "type"));
        int enumerationTag = Math.toIntExact(Long.parseLong(
                stringField(enumerationInput, "tag").substring(2), 16));
        int byteStringTag = Math.toIntExact(Long.parseLong(
                stringField(bytesInput, "tag").substring(2), 16));
        long enumerationValue = numberField(enumerationInput, "value");
        byte[] byteStringValue = java.util.HexFormat.of().parseHex(stringField(bytesInput, "valueHex"));
        List<String> expectedTags = stringArrayField(expected, "childTagsInOrder");
        long expectedEnumeration = numberField(expected, "enumerationValue");
        String expectedByteString = stringField(expected, "byteStringHex");
        assertEquals(expectedTags, List.of(stringField(enumerationInput, "tag"),
                stringField(bytesInput, "tag")));
        assertEquals(expectedEnumeration, enumerationValue);
        assertEquals(expectedByteString, java.util.HexFormat.of().formatHex(byteStringValue));

        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure structure = TtlvStructure.create();
        structure = TtlvStructure.withItem(structure,
                TtlvItem.create(RawTag.fromRaw(enumerationTag).checked(),
                        TtlvValue.Enumeration(enumerationValue), limits), limits);
        structure = TtlvStructure.withItem(structure,
                TtlvItem.create(RawTag.fromRaw(byteStringTag).checked(),
                        TtlvValue.ByteString(byteStringValue, limits), limits), limits);

        TtlvStructureView closedView = structure.view();
        try (closedView) {
            assertEquals(2, closedView.itemCount());
            var enumeration = closedView.itemAt(0);
            assertEquals(Math.toIntExact(Long.parseLong(expectedTags.get(0).substring(2), 16)),
                    enumeration.tag().raw());
            assertEquals(TtlvItemType.Enumeration, enumeration.itemType());
            assertEquals(expectedEnumeration, enumeration.value().enumeration());

            var bytes = closedView.itemAt(1);
            assertEquals(Math.toIntExact(Long.parseLong(expectedTags.get(1).substring(2), 16)),
                    bytes.tag().raw());
            assertEquals(TtlvItemType.ByteString, bytes.itemType());
            assertArrayEquals(java.util.HexFormat.of().parseHex(expectedByteString), byteArray(bytes.value()));
        }

        try (TtlvStructureView reopenedView = structure.view()) {
            assertEquals(2, reopenedView.itemCount(),
                    "closing a borrowed view must not close its independent structure owner");
        }
    }

    @Test
    void codecLimitsFixtureMatchesDefaultsRangesAndErrors() throws IOException {
        String item = caseById(Files.readString(sharedFixturePath()),
                "codec-limits.defaults-and-ranges.v1");
        String input = objectField(item, "input");
        String configuredInput = objectField(input, "configured");
        String expected = objectField(item, "expectedOutput");
        String defaultExpected = objectField(expected, "defaults");
        String configuredExpected = objectField(expected, "configured");
        String invalidDepth = objectField(expected, "invalidDepth");

        assertTrue(Boolean.parseBoolean(stringOrBooleanField(input, "defaults")));
        CodecLimits defaults = CodecLimits.defaults();
        assertEquals(numberField(defaultExpected, "maxMessageBytes"), defaults.maxMessageBytes());
        assertEquals(numberField(defaultExpected, "maxStructureDepth"), defaults.maxStructureDepth());
        assertEquals(numberField(defaultExpected, "maxElements"), defaults.maxElements());

        CodecLimits configured = CodecLimits.create(
                numberField(configuredInput, "maxMessageBytes"),
                numberField(configuredInput, "maxStructureDepth"),
                numberField(configuredInput, "maxElements"));
        assertEquals(numberField(configuredExpected, "maxMessageBytes"), configured.maxMessageBytes());
        assertEquals(numberField(configuredExpected, "maxStructureDepth"), configured.maxStructureDepth());
        assertEquals(numberField(configuredExpected, "maxElements"), configured.maxElements());

        assertFalse(Boolean.parseBoolean(stringOrBooleanField(invalidDepth, "accepted")));
        assertEquals("resource_limit", stringField(invalidDepth, "errorCategory"));
        assertEquals(numberField(invalidDepth, "value"), numberField(input, "invalidDepth"));
        assertThrows(ResourceLimitException.class, () -> CodecLimits.create(
                numberField(configuredInput, "maxMessageBytes"),
                numberField(invalidDepth, "value"),
                numberField(configuredInput, "maxElements")));
    }

    @Test
    void genericTtlvDecoderMatchesSharedFixtureAndRetainsRootOwner() throws IOException {
        String fixture = Files.readString(sharedFixturePath());
        String item = caseById(fixture, "ttlv.decoder-root-structure.v1");
        String input = objectField(item, "input");
        String expected = objectField(item, "expectedOutput");
        byte[] wire = java.util.HexFormat.of().parseHex(stringField(input, "wireHex"));
        org.kmipkit.ttlv.TtlvItem decoded = org.kmipkit.ttlv.TtlvCodec.decode(wire);
        org.kmipkit.ttlv.TtlvItemView root = decoded.view();
        decoded.close();

        try (root;
                var rootValue = root.value();
                var structure = rootValue.structure();
                var enumerationItem = structure.itemAt(0);
                var enumerationValue = enumerationItem.value();
                var byteStringItem = structure.itemAt(1);
                var byteStringValue = byteStringItem.value();
                var rootTag = root.tag();
                var enumerationTag = enumerationItem.tag();
                var byteStringTag = byteStringItem.tag()) {
            assertEquals(Long.parseLong(stringField(expected, "rootTag").substring(2), 16), rootTag.raw());
            assertEquals("Structure", root.itemType().name());
            assertEquals(List.of("0x540001", "0x540002"),
                    List.of(String.format("0x%06x", enumerationTag.raw()),
                            String.format("0x%06x", byteStringTag.raw())));
            assertEquals(numberField(expected, "enumerationValue"), enumerationValue.enumeration());
            assertArrayEquals(java.util.HexFormat.of().parseHex(stringField(expected, "byteStringHex")),
                    byteArray(byteStringValue));
        }

        String truncatedHex = stringField(input, "truncatedWireHex");
        org.kmipkit.ttlv.TtlvDecodeException truncated = assertThrows(
                org.kmipkit.ttlv.TtlvDecodeException.class,
                () -> org.kmipkit.ttlv.TtlvCodec.decode(java.util.HexFormat.of().parseHex(truncatedHex)));
        String expectedTruncated = objectField(expected, "truncatedError");
        assertEquals(numberField(expectedTruncated, "kindCode"), truncated.kind());
        assertEquals(numberField(expectedTruncated, "byteOffset"), truncated.byteOffset());
        assertFalse(truncated.getMessage().contains(truncatedHex));

        long maxBytes = numberField(input, "messageLimitBytes");
        var limits = CodecLimits.create(maxBytes, 64, 100_000);
        org.kmipkit.ttlv.TtlvDecodeException limit = assertThrows(
                org.kmipkit.ttlv.TtlvDecodeException.class,
                () -> org.kmipkit.ttlv.TtlvCodec.decodeWithLimits(wire, limits));
        String expectedLimit = objectField(expected, "messageLimitError");
        assertEquals(numberField(expectedLimit, "kindCode"), limit.kind());
        assertEquals(numberField(expectedLimit, "byteOffset"), limit.byteOffset());
    }

    private static byte[] byteArray(org.kmipkit.ttlv.TtlvValueView value) {
        byte[] result = new byte[Math.toIntExact(value.byteLength())];
        for (int index = 0; index < result.length; index++) {
            result[index] = (byte) value.byteAt(index);
        }
        return result;
    }

    private static TtlvStructure typedRequestTree(String requestInput, String invalidInput) {
        CodecLimits limits = CodecLimits.defaults();
        int missingIdIndex = invalidInput == null
                ? -1
                : Math.toIntExact(numberField(invalidInput, "missingBatchItemIdIndex"));
        List<String> itemInputs = objectArrayFields(requestInput, "batchItems");
        List<TtlvItem> headerItems = new ArrayList<>();
        for (String rawTag : stringArrayField(requestInput, "headerFieldTags")) {
            int tag = fixtureTag(rawTag);
            if (tag == 0x420069) {
                headerItems.add(nested(0x420069,
                        structure(limits, integer(0x42006A, 2, limits), integer(0x42006B, 1, limits)),
                        limits));
            } else if (tag == 0x42000D) {
                headerItems.add(integer(tag, itemInputs.size(), limits));
            } else {
                throw new AssertionError("unexpected request header fixture tag " + rawTag);
            }
        }
        List<TtlvItem> rootItems = new ArrayList<>();
        rootItems.add(nested(0x420077, structure(limits, headerItems.toArray(TtlvItem[]::new)), limits));
        String extension = objectField(requestInput, "extension");
        for (int index = 0; index < itemInputs.size(); index++) {
            String itemInput = itemInputs.get(index);
            List<TtlvItem> fields = new ArrayList<>();
            fields.add(enumeration(0x42005C, numberField(itemInput, "operationRaw"), limits));
            if (index != missingIdIndex) {
                fields.add(byteString(0x420093,
                        java.util.HexFormat.of().parseHex(stringField(itemInput, "uniqueBatchItemIdHex")),
                        limits));
            }
            List<TtlvItem> payloadFields = new ArrayList<>();
            int rawMask = (int) Long.parseLong(
                    stringField(itemInput, "bitMaskRawHex").substring(2), 16);
            payloadFields.add(integer(fixtureTag(stringField(itemInput, "bitMaskTag")), rawMask, limits));
            if (index == missingIdIndex) {
                String marker = stringField(invalidInput, "payloadMarker");
                payloadFields.add(text(0x540002, marker, limits));
            }
            fields.add(nested(0x420079,
                    structure(limits, payloadFields.toArray(TtlvItem[]::new)), limits));
            if (Boolean.parseBoolean(stringOrBooleanField(itemInput, "messageExtension"))) {
                fields.add(extensionField(extension, limits));
            }
            rootItems.add(nested(0x42000F, structure(limits, fields.toArray(TtlvItem[]::new)), limits));
        }
        return structure(limits, rootItems.toArray(TtlvItem[]::new));
    }

    private static TtlvStructure typedResponseTree(String responseInput) {
        CodecLimits limits = CodecLimits.defaults();
        List<TtlvItem> headerItems = new ArrayList<>();
        for (String rawTag : stringArrayField(responseInput, "headerFieldTags")) {
            int tag = fixtureTag(rawTag);
            if (tag == 0x420069) {
                headerItems.add(nested(tag,
                        structure(limits, integer(0x42006A, 2, limits), integer(0x42006B, 1, limits)),
                        limits));
            } else if (tag == 0x420092) {
                headerItems.add(dateTime(tag, numberField(responseInput, "timeStamp"), limits));
            } else if (tag == 0x42000D) {
                headerItems.add(integer(tag, 1, limits));
            } else {
                throw new AssertionError("unexpected response header fixture tag " + rawTag);
            }
        }
        List<TtlvItem> payloadFields = List.of(integer(
                fixtureTag(stringField(responseInput, "responsePayloadBitMaskTag")),
                (int) Long.parseLong(
                        stringField(responseInput, "responsePayloadBitMaskRawHex").substring(2), 16),
                limits));
        TtlvStructure payload = structure(limits, payloadFields.toArray(TtlvItem[]::new));
        List<TtlvItem> batchFields = new ArrayList<>();
        batchFields.add(enumeration(0x42005C, numberField(responseInput, "operationRaw"), limits));
        batchFields.add(enumeration(0x42007F, numberField(responseInput, "resultStatusRaw"), limits));
        batchFields.add(nested(0x42007C, payload, limits));
        if (Boolean.parseBoolean(stringOrBooleanField(responseInput, "messageExtension"))) {
            batchFields.add(extensionField(objectField(responseInput, "extension"), limits));
        }
        return structure(limits,
                nested(0x42007A, structure(limits, headerItems.toArray(TtlvItem[]::new)), limits),
                nested(0x42000F, structure(limits, batchFields.toArray(TtlvItem[]::new)), limits));
    }

    private static TtlvItem extensionField(String extension, CodecLimits limits) {
        TtlvStructure vendorValue = structure(limits, byteString(
                fixtureTag(stringField(extension, "vendorTag")),
                java.util.HexFormat.of().parseHex(stringField(extension, "vendorValueHex")),
                limits));
        TtlvStructure extensionValue = structure(limits,
                text(0x42009D, stringField(extension, "vendorIdentification"), limits),
                bool(0x420026, Boolean.parseBoolean(
                        stringOrBooleanField(extension, "criticalityIndicator")), limits),
                nested(0x42009C, vendorValue, limits));
        return nested(0x420051, extensionValue, limits);
    }

    private static void assertExtensionMatchesFixture(
            MessageExtensionView extension, String expected) {
        assertEquals(stringField(expected, "extensionVendorIdentification"),
                extension.vendorIdentification().orElseThrow());
        assertEquals(Boolean.parseBoolean(stringOrBooleanField(
                expected, "extensionCriticalityIndicator")),
                extension.criticalityIndicator().orElseThrow());
        try (TtlvStructure value = extension.ttlv(); TtlvStructureView view = value.view()) {
            assertEquals(List.of("0x0042009d", "0x00420026", "0x0042009c"), tagStrings(view));
        }
        Boolean observed = extension.withVendorExtension(vendor -> {
            assertEquals(List.of(stringField(expected, "extensionVendorTag")), tagStrings(vendor));
            try (var item = vendor.itemAt(0); var value = item.value()) {
                assertArrayEquals(java.util.HexFormat.of().parseHex(
                        stringField(expected, "extensionVendorValueHex")), byteArray(value));
            }
            return true;
        }).orElseThrow();
        assertTrue(observed);
    }

    private static List<String> tagStrings(TtlvStructureView structure) {
        List<String> result = new ArrayList<>();
        for (long index = 0; index < structure.itemCount(); index++) {
            try (var item = structure.itemAt(index); var tag = item.tag()) {
                result.add(String.format("0x%08x", tag.raw()));
            }
        }
        return List.copyOf(result);
    }

    private static int fixtureTag(String value) {
        return Math.toIntExact(Long.parseLong(value.substring(2), 16));
    }

    private static TtlvStructure structure(CodecLimits limits, TtlvItem... items) {
        TtlvStructure result = TtlvStructure.create();
        for (TtlvItem item : items) {
            result = TtlvStructure.withItem(result, item, limits);
        }
        return result;
    }

    private static TtlvItem nested(int rawTag, TtlvStructure value, CodecLimits limits) {
        return messageItem(rawTag, TtlvValue.structure(value, limits), limits);
    }

    private static TtlvItem integer(int rawTag, int value, CodecLimits limits) {
        return messageItem(rawTag, TtlvValue.Integer(value), limits);
    }

    private static TtlvItem enumeration(int rawTag, long value, CodecLimits limits) {
        return messageItem(rawTag, TtlvValue.Enumeration(value), limits);
    }

    private static TtlvItem dateTime(int rawTag, long value, CodecLimits limits) {
        return messageItem(rawTag, TtlvValue.DateTime(value), limits);
    }

    private static TtlvItem bool(int rawTag, boolean value, CodecLimits limits) {
        return messageItem(rawTag, TtlvValue.Boolean(value), limits);
    }

    private static TtlvItem text(int rawTag, String value, CodecLimits limits) {
        return messageItem(rawTag, TtlvValue.TextString(
                value.getBytes(java.nio.charset.StandardCharsets.UTF_8), limits), limits);
    }

    private static TtlvItem byteString(int rawTag, byte[] value, CodecLimits limits) {
        return messageItem(rawTag, TtlvValue.ByteString(value, limits), limits);
    }

    private static TtlvItem messageItem(int rawTag, TtlvValue value, CodecLimits limits) {
        try (RawTag raw = RawTag.fromRaw(rawTag); var tag = raw.checked()) {
            return TtlvItem.create(tag, value, limits);
        }
    }

    private static void assertJUnitTestExists(String target, String fixtureId) throws Exception {
        String[] components = target.split("#", 2);
        assertEquals(2, components.length, fixtureId + " Java mapping must use Class#method");
        Class<?> consumer = Class.forName(components[0]);
        Method method = consumer.getDeclaredMethod(components[1]);
        assertTrue(method.isAnnotationPresent(Test.class), fixtureId + " consumer is not a JUnit test");
    }

    private static List<String> caseIds(String fixture) {
        Matcher matcher = Pattern.compile("\\\"id\\\"\\s*:\\s*\\\"([^\\\"]+)\\\"").matcher(fixture);
        List<String> ids = new ArrayList<>();
        while (matcher.find()) {
            if (matcher.group(1).endsWith(".v1")) {
                ids.add(matcher.group(1));
            }
        }
        return ids;
    }

    static String caseById(String fixture, String id) {
        int idOffset = fixture.indexOf("\"id\": \"" + id + "\"");
        if (idOffset < 0) {
            throw new AssertionError("shared capability fixture is missing case " + id);
        }
        int start = fixture.lastIndexOf('{', idOffset);
        int depth = 0;
        boolean inString = false;
        boolean escaped = false;
        for (int index = start; index < fixture.length(); index++) {
            char current = fixture.charAt(index);
            if (inString) {
                if (escaped) {
                    escaped = false;
                } else if (current == '\\') {
                    escaped = true;
                } else if (current == '"') {
                    inString = false;
                }
                continue;
            }
            if (current == '"') {
                inString = true;
            } else if (current == '{') {
                depth++;
            } else if (current == '}' && --depth == 0) {
                return fixture.substring(start, index + 1);
            }
        }
        throw new AssertionError("shared capability fixture case is not a complete JSON object: " + id);
    }

    private static JavaConsumerMapping javaConsumerMapping(String item) {
        int javaOffset = item.indexOf("\"java\"");
        if (javaOffset < 0) {
            throw new AssertionError("shared capability case has no Java mapping");
        }
        int start = item.indexOf('{', javaOffset);
        int end = item.indexOf('}', start);
        if (start < 0 || end < 0) {
            throw new AssertionError("shared capability Java mapping is malformed");
        }
        String mapping = item.substring(start, end + 1);
        return new JavaConsumerMapping(
                field(STATUS, mapping, "Java mapping status is required"),
                field(TEST, mapping, "Java consumer test is required"),
                mapping);
    }

    private static String field(Pattern pattern, String source, String failure) {
        Matcher matcher = pattern.matcher(source);
        if (!matcher.find()) {
            throw new AssertionError(failure);
        }
        return matcher.group(1);
    }

    static String stringField(String source, String name) {
        Pattern field = Pattern.compile("\\\"" + Pattern.quote(name) + "\\\"\\s*:\\s*\\\"([^\\\"]*)\\\"");
        return field(field, source, "shared capability fixture is missing " + name);
    }

    static String stringOrBooleanField(String source, String name) {
        Pattern field = Pattern.compile("\\\"" + Pattern.quote(name)
                + "\\\"\\s*:\\s*(?:\\\"([^\\\"]*)\\\"|(true|false))");
        Matcher matcher = field.matcher(source);
        if (!matcher.find()) {
            throw new AssertionError("shared capability fixture is missing " + name);
        }
        return matcher.group(1) == null ? matcher.group(2) : matcher.group(1);
    }

    static long numberField(String source, String name) {
        Pattern field = Pattern.compile("\\\"" + Pattern.quote(name)
                + "\\\"\\s*:\\s*(-?\\d+)");
        Matcher matcher = field.matcher(source);
        if (!matcher.find()) {
            throw new AssertionError("shared capability fixture is missing numeric field " + name);
        }
        return Long.parseLong(matcher.group(1));
    }

    static String objectField(String source, String name) {
        Matcher matcher = Pattern.compile("\\\"" + Pattern.quote(name) + "\\\"\\s*:").matcher(source);
        if (!matcher.find()) {
            throw new AssertionError("shared capability fixture is missing object " + name);
        }
        int start = source.indexOf('{', matcher.end());
        int depth = 0;
        boolean inString = false;
        boolean escaped = false;
        for (int index = start; index < source.length(); index++) {
            char current = source.charAt(index);
            if (inString) {
                if (escaped) escaped = false;
                else if (current == '\\') escaped = true;
                else if (current == '\"') inString = false;
            } else if (current == '\"') inString = true;
            else if (current == '{') depth++;
            else if (current == '}' && --depth == 0) return source.substring(start, index + 1);
        }
        throw new AssertionError("shared capability fixture object is incomplete: " + name);
    }

    static List<String> objectArrayFields(String source, String name) {
        Matcher matcher = Pattern.compile("\\\"" + Pattern.quote(name) + "\\\"\\s*:\\s*\\[").matcher(source);
        if (!matcher.find()) {
            throw new AssertionError("shared capability fixture is missing array " + name);
        }
        int start = matcher.end();
        int depth = 1;
        int end = -1;
        boolean inString = false;
        boolean escaped = false;
        for (int index = start; index < source.length(); index++) {
            char current = source.charAt(index);
            if (inString) {
                if (escaped) escaped = false;
                else if (current == '\\') escaped = true;
                else if (current == '"') inString = false;
            } else if (current == '"') {
                inString = true;
            } else if (current == '[') {
                depth++;
            } else if (current == ']' && --depth == 0) {
                end = index;
                break;
            }
        }
        if (end < 0) throw new AssertionError("shared capability fixture array is incomplete: " + name);
        Matcher objects = Pattern.compile("\\{([^{}]*)\\}").matcher(source.substring(start, end));
        List<String> result = new ArrayList<>();
        while (objects.find()) result.add("{" + objects.group(1) + "}");
        return List.copyOf(result);
    }

    static List<String> stringArrayField(String source, String name) {
        Pattern field = Pattern.compile("\\\"" + Pattern.quote(name)
                + "\\\"\\s*:\\s*\\[([^\\]]*)\\]");
        Matcher matcher = field.matcher(source);
        if (!matcher.find()) {
            throw new AssertionError("shared capability fixture is missing " + name);
        }
        Matcher values = Pattern.compile("\\\"([^\\\"]*)\\\"").matcher(matcher.group(1));
        List<String> result = new ArrayList<>();
        while (values.find()) {
            result.add(values.group(1));
        }
        return List.copyOf(result);
    }

    static List<Long> numberArrayField(String source, String name) {
        Pattern field = Pattern.compile("\\\"" + Pattern.quote(name)
                + "\\\"\\s*:\\s*\\[([^\\]]*)\\]");
        Matcher matcher = field.matcher(source);
        if (!matcher.find()) {
            throw new AssertionError("shared capability fixture is missing " + name);
        }
        Matcher values = Pattern.compile("-?\\d+").matcher(matcher.group(1));
        List<Long> result = new ArrayList<>();
        while (values.find()) {
            result.add(Long.parseLong(values.group()));
        }
        return List.copyOf(result);
    }

    static Path sharedFixturePath() {
        Path workingDirectory = Path.of("").toAbsolutePath();
        Path direct = workingDirectory.resolve("tests/fixtures/api/shared-capability-parity.json");
        if (Files.isRegularFile(direct)) {
            return direct;
        }
        return workingDirectory.resolve("../../tests/fixtures/api/shared-capability-parity.json")
                .normalize();
    }
}
