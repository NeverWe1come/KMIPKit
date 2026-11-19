package org.kmipkit.validator;

import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvCodec;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.file.Path;
import java.util.Map;
import java.util.LinkedHashSet;
import java.util.List;
import java.lang.reflect.Method;

/** Exercises public generic TTLV constructors and views without sending a message. */
public final class TtlvModelChecks {
    private static final CodecLimits LIMITS = CodecLimits.defaults();
    private static final ObjectMapper JSON = new ObjectMapper();

    private TtlvModelChecks() { }

    /** Constructs and reads one raw-preserving enumeration, including an unassigned value. */
    public static long constructAndReadEnumeration(long value) {
        try (RawTag rawTag = RawTag.fromRaw(0x420001);
                Tag tag = rawTag.checked();
                TtlvValue ttlvValue = TtlvValue.Enumeration(value);
                TtlvItem item = TtlvItem.create(tag, ttlvValue, LIMITS);
                var itemView = item.view();
                var valueView = itemView.value()) {
            return valueView.enumeration();
        }
    }

    /** Constructs and reads an integer used as a KMIP bitmask, preserving unknown bits. */
    public static int constructAndReadIntegerMask(int value) {
        try (RawTag rawTag = RawTag.fromRaw(0x420002);
                Tag tag = rawTag.checked();
                TtlvValue ttlvValue = TtlvValue.Integer(value);
                TtlvItem item = TtlvItem.create(tag, ttlvValue, LIMITS);
                var itemView = item.view();
                var valueView = itemView.value()) {
            return valueView.integer();
        }
    }

    /** Decodes one deterministic TTLV Enumeration while preserving its unsigned raw value. */
    public static long decodeEnumeration(long value) {
        try (TtlvItem item = TtlvCodec.decode(wireScalar(0x420001, 0x05, (int) value));
                var itemView = item.view(); var valueView = itemView.value()) {
            return valueView.enumeration();
        }
    }

    /** Decodes one deterministic TTLV Integer bitmask while preserving all raw bits. */
    public static int decodeIntegerMask(int value) {
        try (TtlvItem item = TtlvCodec.decode(wireScalar(0x420002, 0x02, value));
                var itemView = item.view(); var valueView = itemView.value()) {
            return valueView.integer();
        }
    }

    /** Builds a generic structure and verifies that item insertion order is preserved. */
    public static int[] constructAndReadTagOrder() {
        TtlvStructure structure = TtlvStructure.create();
        structure = appendEnumeration(structure, 0x420001, 0xf0000001L);
        structure = appendEnumeration(structure, 0x420002, 0x80000002L);
        TtlvStructure ordered = structure;
        try (ordered; var view = ordered.view();
                var first = view.itemAt(0); var firstTag = first.tag();
                var second = view.itemAt(1); var secondTag = second.tag()) {
            return new int[] {firstTag.raw(), secondTag.raw()};
        }
    }

    /** Executes the generic value-preservation check associated with one local catalog case. */
    public static ModelCheck verifyLocalCase(CatalogLoader.CatalogCase testCase) {
        var requirements = testCase.requires();
        if (requirements.hasNonNull("enum_type")) {
            long constructed = constructAndReadEnumeration(0xf0000001L);
            long decoded = decodeEnumeration(0xf0000001L);
            return new ModelCheck(constructed == decoded, "generic enumeration construction and decoding");
        }
        if (requirements.hasNonNull("bitmask_type")) {
            int constructed = constructAndReadIntegerMask(0x80000001);
            int decoded = decodeIntegerMask(0x80000001);
            return new ModelCheck(constructed == decoded, "generic integer mask construction and decoding");
        }
        return verifyLocalCase(testCase, Map.of());
    }

    /** Exercises one catalog target through the installed Java package's generic TTLV API. */
    public static ModelCheck verifyLocalCase(CatalogLoader.CatalogCase testCase,
            Map<String, JsonNode> normativeTargets) {
        JsonNode requirements = testCase.requires();
        if (requirements.has("exhaustive_bitmask_combinations")) {
            return verifyAllBitmaskCombinations(requirements);
        }
        String targetKind = requirements.path("target_kind").asText();
        String targetId = requirements.path("normative_target_id").asText();
        JsonNode target = normativeTargets.get(targetId);
        if ("data_type".equals(targetKind)) {
            return verifyBoundaryVectors(requirements.path("wire_value").asText());
        }
        if ("tag_range".equals(targetKind)) {
            return verifyTagRange(requirements.path("wire_value").asText());
        }
        if ("tag".equals(targetKind)) {
            return verifyTag(requirements.path("wire_value").asText(),
                    requirements.path("allocation").asText());
        }

        String sourceEncoding = target == null ? "" : target.path("source_encoding").asText();
        if (sourceEncoding.isBlank() && target != null) {
            sourceEncoding = target.path("value_type").asText("");
        }
        String wireValue = requirements.path("wire_value").asText();
        if ("enumeration_value".equals(targetKind) || "referenced_enum_value".equals(targetKind)
                || requirements.hasNonNull("enum_type")) {
            long value = parseWireInteger(wireValue, 0xf0000001L);
            ModelCheck generic = verifyScalar("Enumeration", value, 0x420001);
            if (!generic.passed()) return generic;
            if ("enumeration_value".equals(targetKind)
                    && "assigned".equals(requirements.path("allocation").asText()) && target != null) {
                return verifyAssignedEnumeration(target, normativeTargets, value);
            }
            return generic;
        }
        if ("bitmask_value".equals(targetKind) || requirements.hasNonNull("bitmask_type")) {
            int value = (int) parseWireInteger(wireValue, 0x80000001L);
            ModelCheck generic = verifyScalar("Integer", value, 0x420002);
            if (!generic.passed()) return generic;
            if ("bitmask_value".equals(targetKind) && target != null
                    && "assigned".equals(requirements.path("allocation").asText())) {
                return verifyAssignedBitmaskFlag(target, normativeTargets, value);
            }
            return generic;
        }
        List<String> encodings = resolveEncodings(sourceEncoding);
        for (String encoding : encodings) {
            if ("Any".equalsIgnoreCase(encoding)) {
                encodings = allDataTypes();
                break;
            }
        }
        for (String encoding : encodings) {
            ModelCheck alternativeCheck = verifyScalar(encoding, 1, 0x420003);
            if (!alternativeCheck.passed()) return alternativeCheck;
        }
        return new ModelCheck(true, encodings.size() > 1
                ? "all generic TTLV alternatives constructed and read back"
                : "generic TTLV value constructed and read back for " + encodings.get(0));
    }

    private static ModelCheck verifyBoundaryVectors(String typeCode) {
        try {
            JsonNode vectors = JSON.readTree(Path.of("../shared/vectors/ttlv-boundaries.json").toFile())
                    .path("vectors");
            for (JsonNode vector : vectors) {
                    if (!typeCode.endsWith(vector.path("type_code").asText())) {
                    continue;
                }
                for (JsonNode sample : vector.path("samples")) {
                    byte[] bytes = java.util.HexFormat.of().parseHex(sample.path("hex").asText());
                    boolean valid = sample.path("valid").asBoolean();
                    boolean decoded = decodeSample(bytes);
                    if (valid != decoded) {
                        return new ModelCheck(false, "TTLV boundary vector did not match its expected validity");
                    }
                }
                return new ModelCheck(true, "all pinned TTLV data-type boundary vectors decoded as expected");
            }
            return new ModelCheck(false, "no pinned TTLV boundary vectors match this data type");
        } catch (Exception exception) {
            return new ModelCheck(false, "pinned TTLV boundary vectors could not be evaluated");
        }
    }

    private static boolean decodeSample(byte[] bytes) {
        try (TtlvItem item = TtlvCodec.decode(bytes); var view = item.view(); var value = view.value()) {
            return value.itemType() != null;
        } catch (RuntimeException rejected) {
            return false;
        }
    }

    private static ModelCheck verifyTag(String rawValue, String allocation) {
        try {
            int raw = (int) parseWireInteger(rawValue, -1);
            try (RawTag rawTag = RawTag.fromRaw(raw)) {
                if (!"reserved".equals(allocation)) {
                    try (Tag tag = rawTag.checked()) {
                        return new ModelCheck(tag.raw() == raw,
                                "assigned catalog KMIP tag accepted and preserved by RawTag");
                    }
                }
                boolean checkedRejected = false;
                try (Tag ignored = rawTag.checked()) {
                    // Reserved tag allocations are not exposed as named typed tags.
                } catch (RuntimeException rejected) {
                    checkedRejected = true;
                }
                return new ModelCheck(rawTag.raw() == raw,
                        checkedRejected
                                ? "reserved tag raw value preserved; typed Tag rejects reserved allocation"
                                : "reserved tag raw and checked Tag values preserved");
            }
        } catch (RuntimeException rejected) {
            return new ModelCheck(false, "catalog KMIP tag was rejected by RawTag");
        }
    }

    private static ModelCheck verifyTagRange(String range) {
        String[] bounds = range.split("\\s+[–-]\\s+", 2);
        if (bounds.length != 2) {
            return new ModelCheck(false, "tag range does not contain two hexadecimal bounds");
        }
        try {
            int minimum = (int) parseWireInteger(bounds[0].replace('X', '0').replace('x', '0'), -1);
            int maximum = (int) parseWireInteger(bounds[1].replace('X', 'F').replace('x', 'F'), -1);
            return new ModelCheck(minimum <= maximum && rawTagRoundTrips(minimum) && rawTagRoundTrips(maximum),
                    "tag range bounds are representable by the public raw tag constructor");
        } catch (RuntimeException rejected) {
            return new ModelCheck(false, "tag range bounds could not be constructed");
        }
    }

    private static ModelCheck verifyAssignedEnumeration(JsonNode target,
            Map<String, JsonNode> normativeTargets, long value) {
        try {
            Class<?> type = protocolValueClass(target, normativeTargets, "ENUMERATION-");
            Method name = type.getMethod("name", long.class);
            Object result = name.invoke(null, value);
            return new ModelCheck(result instanceof String && !((String) result).isBlank(),
                    "assigned KMIP enumeration value resolved by its named Java enum API");
        } catch (ReflectiveOperationException | IllegalArgumentException missingValue) {
            return new ModelCheck(false, "assigned KMIP enumeration value is absent from its named Java enum API");
        }
    }

    private static ModelCheck verifyAssignedBitmaskFlag(JsonNode target,
            Map<String, JsonNode> normativeTargets, long value) {
        try {
            Class<?> type = protocolValueClass(target, normativeTargets, "BITMASK-");
            Method knownBits = type.getMethod("knownBits", long.class);
            Method unknownBits = type.getMethod("unknownBits", long.class);
            long known = ((Number) knownBits.invoke(null, value)).longValue();
            long unknown = ((Number) unknownBits.invoke(null, value)).longValue();
            boolean oneBit = value != 0 && (value & (value - 1)) == 0;
            return new ModelCheck(oneBit && known == value && unknown == 0,
                    "assigned KMIP bitmask flag classified by its named Java mask API");
        } catch (ReflectiveOperationException | IllegalArgumentException missingValue) {
            return new ModelCheck(false, "assigned KMIP bitmask flag is absent from its named Java mask API ("
                    + missingValue.getClass().getSimpleName() + ": " + missingValue.getMessage() + ")");
        }
    }

    private static Class<?> protocolValueClass(JsonNode target, Map<String, JsonNode> normativeTargets,
            String expectedParentPrefix) throws ClassNotFoundException {
        JsonNode parents = target.path("parent_element_ids");
        if (!parents.isArray() || parents.isEmpty()) {
            String parentName = target.path("parent_name").asText();
            if (parentName.isBlank()) throw new ClassNotFoundException("No parent enum or mask");
            return Class.forName("org.kmipkit.protocol.KmipValues$" + classSuffix(parentName));
        }
        String parentId = parents.get(0).asText();
        String expectedId = "KMIPKIT-ELEM-" + expectedParentPrefix;
        if (!parentId.startsWith(expectedId)) {
            throw new ClassNotFoundException("No matching parent enum or mask target");
        }
        JsonNode parent = normativeTargets.get(parentId);
        String suffix = parent == null ? parentId.substring(expectedId.length()) : parent.path("name").asText();
        return Class.forName("org.kmipkit.protocol.KmipValues$" + classSuffix(suffix));
    }

    private static String classSuffix(String protocolName) {
        StringBuilder suffix = new StringBuilder();
        for (String token : protocolName.split("[^A-Za-z0-9]+")) {
            if (token.isEmpty()) continue;
            String lower = token.toLowerCase(java.util.Locale.ROOT);
            suffix.append(Character.toUpperCase(lower.charAt(0))).append(lower.substring(1));
        }
        return suffix.toString();
    }

    private static ModelCheck verifyAllBitmaskCombinations(JsonNode requirements) {
        JsonNode combinations = requirements.path("exhaustive_bitmask_combinations");
        try {
            Class<?> type = Class.forName("org.kmipkit.protocol.KmipValues$"
                    + classSuffix(requirements.path("bitmask_type").asText()));
            Method knownBits = type.getMethod("knownBits", long.class);
            Method unknownBits = type.getMethod("unknownBits", long.class);
            JsonNode flags = combinations.path("flag_wire_values");
            long count = combinations.path("combination_count").asLong(-1);
            if (!flags.isArray() || count != (1L << flags.size())) {
                return new ModelCheck(false, "bitmask combination inventory is inconsistent");
            }
            int[] masks = new int[flags.size()];
            for (int index = 0; index < flags.size(); index++) {
                masks[index] = (int) parseWireInteger(flags.get(index).asText(), -1);
            }
            for (long subset = 0; subset < count; subset++) {
                int mask = 0;
                for (int flag = 0; flag < masks.length; flag++) {
                    if ((subset & (1L << flag)) != 0) mask |= masks[flag];
                }
                long wireMask = Integer.toUnsignedLong(mask);
                if (constructAndReadIntegerMask(mask) != mask
                        || ((Number) knownBits.invoke(null, wireMask)).longValue() != wireMask
                        || ((Number) unknownBits.invoke(null, wireMask)).longValue() != 0) {
                    return new ModelCheck(false, "a KMIP bitmask combination lost or misclassified bits");
                }
            }
            long allFlags = 0;
            for (int mask : masks) allFlags |= Integer.toUnsignedLong(mask);
            long unknownProbe = allFlags | 0x80000000L;
            return new ModelCheck(((Number) knownBits.invoke(null, unknownProbe)).longValue() == allFlags
                            && ((Number) unknownBits.invoke(null, unknownProbe)).longValue() == 0x80000000L,
                    "all bitmask combinations and a separate unknown-bit value were checked");
        } catch (ReflectiveOperationException | IllegalArgumentException invalidInventory) {
            return new ModelCheck(false, "exhaustive KMIP bitmask combinations could not be evaluated");
        }
    }

    private static boolean rawTagRoundTrips(int raw) {
        try (RawTag tag = RawTag.fromRaw(raw)) {
            return tag.raw() == raw;
        }
    }

    private static ModelCheck verifyScalar(String encoding, long numeric, int rawTag) {
        TtlvStructure empty = null;
        TtlvValue value = null;
        try {
            String normalized = encoding.toLowerCase(java.util.Locale.ROOT)
                    .replaceAll("[^a-z0-9]", "");
            value = switch (normalized) {
                case "integer" -> TtlvValue.Integer((int) numeric);
                case "longinteger" -> TtlvValue.LongInteger(numeric);
                case "enumeration" -> TtlvValue.Enumeration(numeric);
                case "boolean" -> TtlvValue.Boolean((numeric & 1) != 0);
                case "datetime" -> TtlvValue.DateTime(numeric);
                case "interval" -> TtlvValue.Interval(numeric);
                case "datetimeextended" -> TtlvValue.DateTimeExtended(numeric);
                case "biginteger" -> TtlvValue.BigInteger(ByteBuffer.allocate(Long.BYTES)
                        .order(ByteOrder.BIG_ENDIAN).putLong(numeric).array(), LIMITS);
                case "textstring" -> TtlvValue.TextString("kmip".getBytes(java.nio.charset.StandardCharsets.UTF_8),
                        LIMITS);
                case "bytestring" -> TtlvValue.ByteString(new byte[] {0x01, 0x02}, LIMITS);
                case "structure" -> {
                    empty = TtlvStructure.create();
                    yield TtlvValue.structure(empty, LIMITS);
                }
                default -> throw new IllegalArgumentException("Unsupported TTLV data type");
            };
            try (RawTag raw = RawTag.fromRaw(rawTag); Tag tag = raw.checked();
                    TtlvItem item = TtlvItem.create(tag, value, LIMITS);
                    var itemView = item.view(); var readback = itemView.value()) {
                value = null;
                return new ModelCheck(readback.itemType() != null,
                        "public generic TTLV constructor and value view exercised for " + encoding);
            }
        } catch (RuntimeException rejected) {
            return new ModelCheck(false, "public generic TTLV model rejected " + encoding);
        } finally {
            if (value != null) value.close();
            if (empty != null) empty.close();
        }
    }

    private static List<String> resolveEncodings(String sourceEncoding) {
        if (sourceEncoding == null || sourceEncoding.isBlank()) return List.of("Structure");
        if (sourceEncoding.toLowerCase(java.util.Locale.ROOT).startsWith("varies")) {
            return List.of("Structure");
        }
        String normalized = sourceEncoding.replaceAll("(?i),?\\s*may be repeated", "")
                .replaceAll("(?i),?\\s*see below", "");
        int qualifier = normalized.indexOf('(');
        if (qualifier >= 0) normalized = normalized.substring(0, qualifier);
        var encodings = new LinkedHashSet<String>();
        for (String branch : normalized.split(";")) {
            int colon = branch.indexOf(':');
            if (colon >= 0) branch = branch.substring(0, colon);
            for (String alternative : branch.split("(?i)\\s+or\\s+|[,/|]")) {
                String encoding = alternative.trim();
                if (encoding.isBlank()) continue;
                String lower = encoding.toLowerCase(java.util.Locale.ROOT);
                if (lower.contains("structure") || lower.equals("attribute")
                        || lower.startsWith("varies") || lower.contains("credential")) {
                    encodings.add("Structure");
                } else if (lower.equals("textstring")) {
                    encodings.add("Text String");
                } else if (lower.equals("bytestring")) {
                    encodings.add("Byte String");
                } else {
                    encodings.add(encoding);
                }
            }
        }
        return encodings.isEmpty() ? List.of("Structure") : List.copyOf(encodings);
    }

    private static List<String> allDataTypes() {
        return List.of("Structure", "Integer", "Long Integer", "Big Integer", "Enumeration",
                "Boolean", "Text String", "Byte String", "Date Time", "Interval", "Date Time Extended");
    }

    private static long parseWireInteger(String wireValue, long fallback) {
        if (wireValue == null || wireValue.isBlank() || wireValue.indexOf('X') >= 0
                || wireValue.contains(" - ") || wireValue.indexOf('-') >= 0) return fallback;
        String normalized = wireValue.startsWith("0x") || wireValue.startsWith("0X")
                ? wireValue.substring(2) : wireValue;
        return Long.parseUnsignedLong(normalized, 16);
    }

    private static TtlvStructure appendEnumeration(TtlvStructure structure, int tagValue, long value) {
        try (RawTag rawTag = RawTag.fromRaw(tagValue);
                Tag tag = rawTag.checked();
                TtlvValue ttlvValue = TtlvValue.Enumeration(value)) {
            return TtlvStructure.withItem(structure, TtlvItem.create(tag, ttlvValue, LIMITS), LIMITS);
        }
    }

    private static byte[] wireScalar(int tag, int type, int value) {
        return java.nio.ByteBuffer.allocate(16)
                .put((byte) (tag >>> 16)).put((byte) (tag >>> 8)).put((byte) tag)
                .put((byte) type).putInt(4).putInt(value).putInt(0).array();
    }

    public record ModelCheck(boolean passed, String diagnostic) { }
}
