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
        assertEquals(13, ids.size());
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

    private static byte[] byteArray(org.kmipkit.ttlv.TtlvValueView value) {
        byte[] result = new byte[Math.toIntExact(value.byteLength())];
        for (int index = 0; index < result.length; index++) {
            result[index] = (byte) value.byteAt(index);
        }
        return result;
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
        int end = source.indexOf(']', start);
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
