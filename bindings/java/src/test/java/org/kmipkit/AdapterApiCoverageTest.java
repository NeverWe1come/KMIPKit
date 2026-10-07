package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.Optional;

import org.junit.jupiter.api.Test;
import org.kmipkit.extensions.ClientConfiguration;
import org.kmipkit.extensions.ClientExtensionRegistry;
import org.kmipkit.extensions.Compatibility;
import org.kmipkit.extensions.Discriminator;
import org.kmipkit.extensions.ExtensionChildRule;
import org.kmipkit.extensions.ExtensionDefinition;
import org.kmipkit.extensions.ExtensionIdentity;
import org.kmipkit.extensions.ExtensionInformation;
import org.kmipkit.extensions.ExtensionOrderConstraint;
import org.kmipkit.extensions.ExtensionRecognition;
import org.kmipkit.extensions.ExtensionSchema;
import org.kmipkit.extensions.InvalidExtensionIdentityException;
import org.kmipkit.extensions.TtlvPath;
import org.kmipkit.extensions.ValidatedExtensionValue;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TtlvValueView;

/** Focused execution of Java adapter facade paths that are otherwise easy to leave uncovered. */
final class AdapterApiCoverageTest {
    private static final String VENDOR = "coverage.vendor";
    private static final String NAME = "coverage";
    private static final String VERSION = "1";
    private static final int ROOT_TAG = 0x420010;
    private static final int REQUIRED_TAG = 0x420011;
    private static final int OPTIONAL_TAG = 0x420012;
    private static final int REPEATED_TAG = 0x420013;
    private static final CodecLimits CODEC_LIMITS = CodecLimits.defaults();

    @Test
    void scalarTtlvFactoriesAndViewsExposeAllSupportedTypes() {
        assertEquals(17, TtlvValue.Integer(17).view().integer());
        assertEquals(TtlvItemType.Integer, TtlvValue.Integer(17).view().itemType());
        assertEquals(-123L, TtlvValue.LongInteger(-123).view().longInteger());
        assertEquals(4_294_967_295L, TtlvValue.Enumeration(4_294_967_295L).view().enumeration());
        assertTrue(TtlvValue.Boolean(true).view().booleanValue());
        assertEquals(1_699_999_999L, TtlvValue.DateTime(1_699_999_999).view().dateTime());
        assertEquals(120L, TtlvValue.Interval(120).view().interval());
        assertEquals(2_000_000_000L, TtlvValue.DateTimeExtended(2_000_000_000L)
                .view().dateTimeExtended());
        assertByteValue(TtlvValue.BigInteger(new byte[] {1, 2, 3}, CODEC_LIMITS).view(),
                TtlvItemType.BigInteger, new byte[] {1, 2, 3});
        assertByteValue(TtlvValue.TextString("adapter".getBytes(StandardCharsets.UTF_8), CODEC_LIMITS).view(),
                TtlvItemType.TextString, "adapter".getBytes(StandardCharsets.UTF_8));
        assertByteValue(TtlvValue.ByteString(new byte[] {0, -1, 42}, CODEC_LIMITS).view(),
                TtlvItemType.ByteString, new byte[] {0, -1, 42});

        TtlvValueView structureValue = TtlvValue.structure(TtlvStructure.create(), CODEC_LIMITS).view();
        assertEquals(TtlvItemType.Structure, structureValue.itemType());
        assertEquals(0, structureValue.structure().itemCount());
    }

    @Test
    void tagsCodecLimitsAndItemTypeCodesValidateTheirBoundaries() {
        Tag tag = RawTag.fromRaw(REQUIRED_TAG).checked();
        assertEquals(REQUIRED_TAG, tag.raw());
        assertEquals(REQUIRED_TAG, RawTag.fromRaw(REQUIRED_TAG).raw());
        assertThrows(InvalidInputException.class, () -> RawTag.fromRaw(0x01000000));
        assertThrows(org.kmipkit.extensions.InvalidExtensionSchemaException.class,
                () -> RawTag.fromRaw(0x420000).checked());
        assertThrows(InvalidInputException.class, () -> TtlvItemType.fromCode(0));
        for (TtlvItemType type : TtlvItemType.values()) {
            assertEquals(type, TtlvItemType.fromCode(type.code()));
        }

        CodecLimits limits = CodecLimits.create(1, 2, 3);
        assertEquals(1, limits.maxMessageBytes());
        assertEquals(2, limits.maxStructureDepth());
        assertEquals(3, limits.maxElements());
        assertThrows(InvalidInputException.class, () -> CodecLimits.create(-1, 0, 0));
        assertThrows(InvalidInputException.class, () -> CodecLimits.create(0, 0, -1));
        assertThrows(ResourceLimitException.class, () -> CodecLimits.create(0, 65, 0));
    }

    @Test
    void extensionIdentityChecksTextAndImplementsValueEquality() {
        ExtensionIdentity identity = identity(NAME);
        assertEquals(VENDOR, identity.vendorIdentifier());
        assertEquals(NAME, identity.name());
        assertEquals(VERSION, identity.version());
        assertEquals(identity, identity(NAME));
        assertEquals(identity.hashCode(), identity(NAME).hashCode());
        assertNotEquals(identity, identity("other"));
        assertNotEquals(identity, "not an identity");
        assertTrue(identity.toString().contains(NAME));

        assertThrows(InvalidExtensionIdentityException.class, () -> ExtensionIdentity.create(null, NAME, VERSION));
        assertThrows(InvalidExtensionIdentityException.class, () -> ExtensionIdentity.create("bad vendor", NAME, VERSION));
        assertThrows(InvalidExtensionIdentityException.class, () -> ExtensionIdentity.create(VENDOR, "", VERSION));
        assertThrows(InvalidExtensionIdentityException.class, () -> ExtensionIdentity.create(VENDOR, NAME, null));
        assertThrows(ResourceLimitException.class,
                () -> ExtensionIdentity.create(VENDOR, "x".repeat(4_097), VERSION));
    }

    @Test
    void schemaModifiersChildRulesAndOrderConstraintsBuildAValidatedTree() {
        ExtensionSchema text = ExtensionSchema.with_maximum_length(
                ExtensionSchema.with_minimum_length(
                        ExtensionSchema.scalar(TtlvItemType.TextString), 1), 32);
        ExtensionSchema signed = ExtensionSchema.with_signed_range(
                ExtensionSchema.scalar(TtlvItemType.Integer), -5, 5);
        ExtensionSchema unsigned = ExtensionSchema.with_unsigned_range(
                ExtensionSchema.scalar(TtlvItemType.Interval), 0, 100);
        ExtensionSchema enumeration = ExtensionSchema.with_allowed_enumeration(
                ExtensionSchema.scalar(TtlvItemType.Enumeration), 7);
        ExtensionSchema allowedMask = ExtensionSchema.with_allowed_bit_mask(
                ExtensionSchema.scalar(TtlvItemType.Integer), 0x7f);
        ExtensionSchema requiredMask = ExtensionSchema.with_required_bit_mask(
                ExtensionSchema.scalar(TtlvItemType.Integer), 0x20);
        assertTrue(text != signed && unsigned != enumeration && allowedMask != requiredMask);

        Tag requiredTag = tag(REQUIRED_TAG);
        Tag optionalTag = tag(OPTIONAL_TAG);
        Tag repeatedTag = tag(REPEATED_TAG);
        ExtensionChildRule required = ExtensionChildRule.required(requiredTag, text);
        ExtensionChildRule optional = ExtensionChildRule.optional(optionalTag, signed);
        ExtensionChildRule repeated = ExtensionChildRule.repeated(repeatedTag, unsigned);
        ExtensionOrderConstraint order = ExtensionOrderConstraint.create(requiredTag, optionalTag);
        ExtensionSchema tree = ExtensionSchema.structure(
                List.of(required, optional, repeated), List.of(order), false);
        ExtensionSchema rootSchema = ExtensionSchema.structure(List.of(
                ExtensionChildRule.required(tag(ROOT_TAG), tree)), List.of(), false);

        TtlvPath path = TtlvPath.create(tag(ROOT_TAG));
        path = TtlvPath.withChildTag(path, requiredTag);
        TtlvValue discriminatorValue = TtlvValue.TextString(bytes("marker"), CODEC_LIMITS);
        Discriminator discriminator = Discriminator.create(path, discriminatorValue);
        ExtensionDefinition definition = definition(rootSchema, discriminator);
        ClientExtensionRegistry registry = ClientExtensionRegistry.create(List.of(definition),
                org.kmipkit.extensions.ExtensionRegistryLimits.defaults());
        ExtensionRecognition recognition = registry.inspect(VENDOR, nestedPayloadForRegistry(), CODEC_LIMITS);
        assertTrue(ExtensionRecognition.isRecognized(recognition));
        assertTrue(ExtensionRecognition.validatedValue(recognition).isPresent());
    }

    @Test
    void nestedValidatedValuesAndOptionalDefinitionInformationRemainAccessible() {
        ExtensionSchema nestedText = ExtensionSchema.scalar(TtlvItemType.TextString);
        ExtensionSchema nestedSchema = ExtensionSchema.structure(
                List.of(ExtensionChildRule.required(tag(REQUIRED_TAG), nestedText)), List.of(), false);
        ExtensionSchema rootSchema = ExtensionSchema.structure(
                List.of(ExtensionChildRule.required(tag(ROOT_TAG), nestedSchema)), List.of(), false);
        TtlvPath path = TtlvPath.withChildTag(TtlvPath.create(tag(ROOT_TAG)), tag(REQUIRED_TAG));
        Discriminator discriminator = Discriminator.create(path,
                TtlvValue.TextString(bytes("nested-marker"), CODEC_LIMITS));
        ExtensionDefinition definition = definition(rootSchema, discriminator);

        assertTrue(ExtensionDefinition.information(definition).isEmpty());
        ExtensionInformation information = ExtensionInformation.create(NAME);
        ExtensionDefinition enriched = ExtensionDefinition.withInformation(definition, information);
        assertEquals(Optional.of(information), ExtensionDefinition.information(enriched));
        assertEquals(identity(NAME), ExtensionDefinition.identity(enriched));

        ValidatedExtensionValue validated = ExtensionDefinition.validate(enriched, nestedPayload(), CODEC_LIMITS);
        assertEquals(identity(NAME), ValidatedExtensionValue.identity(validated));
        TtlvStructureView generic = ValidatedExtensionValue.genericValue(validated);
        assertEquals(1, generic.itemCount());
        TtlvPath valuePath = TtlvPath.withChildTag(TtlvPath.create(tag(ROOT_TAG)), tag(REQUIRED_TAG));
        TtlvValueView child = ValidatedExtensionValue.valueAt(validated, valuePath);
        assertEquals(TtlvItemType.TextString, child.itemType());
        assertEquals("nested-marker", textValue(child));
        assertFalse(validated.toString().contains("nested-marker"));

        ClientConfiguration configuration = ClientConfiguration.create(
                ClientExtensionRegistry.create(List.of(enriched),
                        org.kmipkit.extensions.ExtensionRegistryLimits.defaults()));
        assertEquals(1, configuration.extensionRegistry().definitionCount());
    }

    @Test
    void genericStructuresRetainItemTagTypeAndValueViews() {
        TtlvStructure structure = TtlvStructure.create();
        TtlvItem item = TtlvItem.create(tag(REQUIRED_TAG),
                TtlvValue.Integer(9), CODEC_LIMITS);
        structure = TtlvStructure.withItem(structure, item, CODEC_LIMITS);
        TtlvStructureView view = structure.view();
        assertEquals(1, view.itemCount());
        assertEquals(REQUIRED_TAG, view.itemAt(0).tag().raw());
        assertEquals(TtlvItemType.Integer, view.itemAt(0).itemType());
        assertEquals(9, view.itemAt(0).value().integer());
        assertThrows(InvalidInputException.class, () -> view.itemAt(1));
    }

    private static void assertByteValue(TtlvValueView value, TtlvItemType type, byte[] expected) {
        assertEquals(type, value.itemType());
        assertEquals(expected.length, value.byteLength());
        byte[] actual = new byte[expected.length];
        for (int index = 0; index < actual.length; index++) {
            actual[index] = (byte) value.byteAt(index);
        }
        assertEquals(java.util.Arrays.toString(expected), java.util.Arrays.toString(actual));
    }

    private static String textValue(TtlvValueView value) {
        byte[] bytes = new byte[Math.toIntExact(value.byteLength())];
        for (int index = 0; index < bytes.length; index++) {
            bytes[index] = (byte) value.byteAt(index);
        }
        return new String(bytes, StandardCharsets.UTF_8);
    }

    private static TtlvStructure nestedPayload() {
        TtlvStructure inner = TtlvStructure.withItem(TtlvStructure.create(),
                TtlvItem.create(tag(REQUIRED_TAG),
                        TtlvValue.TextString(bytes("nested-marker"), CODEC_LIMITS), CODEC_LIMITS), CODEC_LIMITS);
        return TtlvStructure.withItem(TtlvStructure.create(),
                TtlvItem.create(tag(ROOT_TAG), TtlvValue.structure(inner, CODEC_LIMITS), CODEC_LIMITS),
                CODEC_LIMITS);
    }

    private static TtlvStructure nestedPayloadForRegistry() {
        TtlvStructure inner = TtlvStructure.create();
        inner = TtlvStructure.withItem(inner, TtlvItem.create(tag(REQUIRED_TAG),
                TtlvValue.TextString(bytes("marker"), CODEC_LIMITS), CODEC_LIMITS), CODEC_LIMITS);
        inner = TtlvStructure.withItem(inner, TtlvItem.create(tag(OPTIONAL_TAG),
                TtlvValue.Integer(2), CODEC_LIMITS), CODEC_LIMITS);
        inner = TtlvStructure.withItem(inner, TtlvItem.create(tag(REPEATED_TAG),
                TtlvValue.Interval(9), CODEC_LIMITS), CODEC_LIMITS);
        inner = TtlvStructure.withItem(inner, TtlvItem.create(tag(REPEATED_TAG),
                TtlvValue.Interval(10), CODEC_LIMITS), CODEC_LIMITS);
        return TtlvStructure.withItem(TtlvStructure.create(), TtlvItem.create(tag(ROOT_TAG),
                TtlvValue.structure(inner, CODEC_LIMITS), CODEC_LIMITS), CODEC_LIMITS);
    }

    private static ExtensionDefinition definition(ExtensionSchema schema, Discriminator discriminator) {
        return ExtensionDefinition.create(identity(NAME),
                Compatibility.create(2, 1, 2, 1, "0.0.0", "99.0.0"), discriminator, schema);
    }

    private static ExtensionIdentity identity(String name) {
        return ExtensionIdentity.create(VENDOR, name, VERSION);
    }

    private static Tag tag(int raw) {
        return RawTag.fromRaw(raw).checked();
    }

    private static byte[] bytes(String value) {
        return value.getBytes(StandardCharsets.UTF_8);
    }
}
