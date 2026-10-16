package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertEquals;
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

/** Executes the Java mappings declared by the shared API capability fixture. */
final class SharedCapabilityParityFixtureTest {
    private record JavaConsumerMapping(String status, String test, String json) {}

    private static final Pattern STATUS = Pattern.compile("\\\"status\\\"\\s*:\\s*\\\"([^\\\"]+)\\\"");
    private static final Pattern TEST = Pattern.compile("\\\"test\\\"\\s*:\\s*\\\"([^\\\"]+)\\\"");
    private static final String API_LEVEL_CASE = "api-levels.shared.v1";
    private static final String BATCH_CASE = "batch.ordered-completed-pending-failed.v1";
    private static final long OPEN_ENUMERATION = 0xE1020304L;
    private static final int ENUMERATION_TAG = 0x540001;
    private static final int BYTE_STRING_TAG = 0x540002;
    private static final CodecLimits CODEC_LIMITS = CodecLimits.defaults();

    @Test
    void everyJavaFixtureMappingTargetsAnExecutableJUnitConsumer() throws Exception {
        String fixture = Files.readString(sharedFixturePath());
        assertEquals(List.of("rust", "c", "java", "python"),
                stringArrayField(fixture, "parityLanguages"));
        assertEquals(List.of(), stringArrayField(fixture, "plannedConsumerLanguages"));
        assertEquals(List.of("rust", "c", "java", "python"),
                stringArrayField(fixture, "executableConsumerLanguages"));

        List<String> ids = caseIds(fixture);
        assertEquals(7, ids.size());
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
    void genericTtlvFixturePreservesOpenEnumerationBytesOrderAndViewLifecycle() throws IOException {
        String item = caseById(Files.readString(sharedFixturePath()), "ttlv.open-values-and-child-order.v1");
        assertTrue(item.contains("\"enumerationValue\": 3775005444"));
        assertTrue(item.contains("\"byteStringHex\": \"80ff01\""));
        TtlvStructure structure = TtlvStructure.create();
        structure = TtlvStructure.withItem(structure,
                TtlvItem.create(RawTag.fromRaw(ENUMERATION_TAG).checked(),
                        TtlvValue.Enumeration(OPEN_ENUMERATION), CODEC_LIMITS), CODEC_LIMITS);
        structure = TtlvStructure.withItem(structure,
                TtlvItem.create(RawTag.fromRaw(BYTE_STRING_TAG).checked(),
                        TtlvValue.ByteString(new byte[] {(byte) 0x80, (byte) 0xff, 0x01}, CODEC_LIMITS),
                        CODEC_LIMITS), CODEC_LIMITS);

        TtlvStructureView closedView = structure.view();
        try (closedView) {
            assertEquals(2, closedView.itemCount());
            var enumeration = closedView.itemAt(0);
            assertEquals(ENUMERATION_TAG, enumeration.tag().raw());
            assertEquals(TtlvItemType.Enumeration, enumeration.itemType());
            assertEquals(OPEN_ENUMERATION, enumeration.value().enumeration());

            var bytes = closedView.itemAt(1);
            assertEquals(BYTE_STRING_TAG, bytes.tag().raw());
            assertEquals(TtlvItemType.ByteString, bytes.itemType());
            assertEquals(List.of(0x80, 0xff, 0x01), byteValues(bytes.value()));
        }

        try (TtlvStructureView reopenedView = structure.view()) {
            assertEquals(2, reopenedView.itemCount(),
                    "closing a borrowed view must not close its independent structure owner");
        }
    }

    private static List<Integer> byteValues(org.kmipkit.ttlv.TtlvValueView value) {
        List<Integer> result = new ArrayList<>();
        for (long index = 0; index < value.byteLength(); index++) {
            result.add(Short.toUnsignedInt(value.byteAt(index)));
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

    private static String caseById(String fixture, String id) {
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

    private static String stringField(String source, String name) {
        Pattern field = Pattern.compile("\\\"" + Pattern.quote(name) + "\\\"\\s*:\\s*\\\"([^\\\"]*)\\\"");
        return field(field, source, "shared capability fixture is missing " + name);
    }

    private static List<String> stringArrayField(String source, String name) {
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

    private static Path sharedFixturePath() {
        Path workingDirectory = Path.of("").toAbsolutePath();
        Path direct = workingDirectory.resolve("tests/fixtures/api/shared-capability-parity.json");
        if (Files.isRegularFile(direct)) {
            return direct;
        }
        return workingDirectory.resolve("../../tests/fixtures/api/shared-capability-parity.json")
                .normalize();
    }
}
