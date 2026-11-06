package org.kmipkit.typed;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TtlvValueView;
import org.kmipkit.ttlv.StandardAttribute;
import org.kmipkit.ttlv.TypedAttribute;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.extensions.AttributeSet;

final class TypedAttributeParityTest {
    @Test
    void exposesAllSixtyTwoDirectAttributesWithUniqueTags() {
        assertEquals(62, StandardAttribute.values().length);
        assertEquals(62, java.util.Arrays.stream(StandardAttribute.values())
                .map(StandardAttribute::tag).distinct().count());
        assertEquals(0x0042_0001, StandardAttribute.ACTIVATION_DATE.tag());
        assertEquals(0x0042_0008, StandardAttribute.VENDOR_ATTRIBUTE.tag());
    }

    @Test
    void roundTripsEveryCataloguedAttributeAndItsSourceTtlvType() {
        CodecLimits limits = CodecLimits.defaults();
        for (StandardAttribute attribute : StandardAttribute.values()) {
            TtlvItem source = item(attribute, valueFor(attribute, limits), limits);
            try (source; TypedAttribute typed = TypedAttribute.fromItem(attribute, source);
                    TtlvItemView view = typed.view(); Tag actualTag = view.tag()) {
                assertEquals(attribute, typed.attribute());
                assertEquals(attribute.tag(), actualTag.raw());
                assertEquals(expectedType(attribute), view.itemType(), attribute.name());
                try (TtlvItemView sourceView = source.view(); Tag sourceTag = sourceView.tag()) {
                    assertEquals(attribute.tag(), sourceTag.raw(), "conversion must preserve input owner");
                }
            }
        }
    }

    @Test
    void constructsAndReadsActualCataloguedValues() {
        CodecLimits limits = CodecLimits.defaults();
        for (StandardAttribute attribute : StandardAttribute.values()) {
            try (TtlvValue value = valueFor(attribute, limits);
                    TypedAttribute typed = TypedAttribute.create(attribute, value);
                    TtlvValue copied = typed.value();
                    TtlvValueView actual = copied.view()) {
                assertEquals(expectedType(attribute), actual.itemType(), attribute.name());
                switch (attribute.valueKind()) {
                    case "Enumeration" -> assertEquals(0xffff_ffffL, actual.enumeration());
                    case "Integer" -> assertEquals(7, actual.integer());
                    case "Boolean" -> assertEquals(true, actual.booleanValue());
                    case "BigInteger", "ByteString" -> assertBytes(actual, new byte[] {1});
                    case "TextString" -> assertBytes(actual, "attribute-value".getBytes(
                            java.nio.charset.StandardCharsets.UTF_8));
                    case "DateTime" -> assertEquals(1_700_000_000L, actual.dateTime());
                    case "DateTimeExtended" -> assertEquals(1_700_000_000L,
                            actual.dateTimeExtended());
                    case "Interval" -> assertEquals(7, actual.interval());
                    case "Structure" -> {
                        try (TtlvStructureView structure = actual.structure()) {
                            assertEquals(attribute == StandardAttribute.VENDOR_ATTRIBUTE ? 3 : 0,
                                    structure.itemCount());
                            if (attribute == StandardAttribute.VENDOR_ATTRIBUTE) {
                                assertStructureChild(structure, 0, 0x0042_009d,
                                        "KMIPKit.TestVendor_1".getBytes());
                                assertStructureChild(structure, 1, 0x0042_000a,
                                        "Opaque.Attribute".getBytes());
                                assertStructureChild(structure, 2, 0x0042_000b,
                                        new byte[] {1});
                            }
                        }
                    }
                    default -> { }
                }
            }
        }
    }

    @Test
    void uniqueIdentifierPreservesOpenEnumerationAndIntegerVariants() {
        try (TtlvValue enumValue = TtlvValue.Enumeration(0xffff_ffffL);
                TypedAttribute enumAttribute = TypedAttribute.create(
                        StandardAttribute.UNIQUE_IDENTIFIER, enumValue);
                TtlvValue copiedEnum = enumAttribute.value();
                TtlvValueView enumView = copiedEnum.view()) {
            assertEquals(TtlvItemType.Enumeration, enumView.itemType());
            assertEquals(0xffff_ffffL, enumView.enumeration());
        }
        try (TtlvValue integerValue = TtlvValue.Integer(-23);
                TypedAttribute integerAttribute = TypedAttribute.create(
                        StandardAttribute.UNIQUE_IDENTIFIER, integerValue);
                TtlvValue copiedInteger = integerAttribute.value();
                TtlvValueView integerView = copiedInteger.view()) {
            assertEquals(TtlvItemType.Integer, integerView.itemType());
            assertEquals(-23, integerView.integer());
        }
    }

    private static void assertBytes(TtlvValueView value, byte[] expected) {
        assertEquals(expected.length, value.byteLength());
        for (int index = 0; index < expected.length; index++) {
            assertEquals(Byte.toUnsignedInt(expected[index]),
                    Short.toUnsignedInt(value.byteAt(index)), "byte at " + index);
        }
    }

    private static void assertStructureChild(
            TtlvStructureView structure, long index, int expectedTag, byte[] expectedValue) {
        try (TtlvItemView child = structure.itemAt(index);
                Tag tag = child.tag();
                TtlvValueView value = child.value()) {
            assertEquals(expectedTag, tag.raw());
            assertBytes(value, expectedValue);
        }
    }

    @Test
    void typedAttributeSetSupportsOrderedRepeatedValues() {
        CodecLimits limits = CodecLimits.defaults();
        try (TtlvValue firstValue = TtlvValue.Enumeration(0xffff_ffffL);
                TtlvValue secondValue = TtlvValue.Enumeration(0xffff_fffeL);
                TypedAttribute first = TypedAttribute.create(
                        StandardAttribute.CRYPTOGRAPHIC_ALGORITHM, firstValue);
                TypedAttribute second = TypedAttribute.create(
                        StandardAttribute.CRYPTOGRAPHIC_ALGORITHM, secondValue);
                TtlvStructure empty = TtlvStructure.create();
                AttributeSet attributes = AttributeSet.tryNew(empty)) {
            attributes.tryPushTyped(StandardAttribute.CRYPTOGRAPHIC_ALGORITHM, first);
            attributes.tryPushTyped(StandardAttribute.CRYPTOGRAPHIC_ALGORITHM, second);
            assertEquals(2, attributes.typedCount(StandardAttribute.CRYPTOGRAPHIC_ALGORITHM));
            try (TypedAttribute actual = attributes.typedAt(
                            StandardAttribute.CRYPTOGRAPHIC_ALGORITHM, 1);
                    TtlvValue copied = actual.value();
                    TtlvValueView value = copied.view()) {
                assertEquals(0xffff_fffeL, value.enumeration());
            }
            var repeated = attributes.typed(StandardAttribute.CRYPTOGRAPHIC_ALGORITHM);
            assertEquals(2, repeated.size());
            repeated.forEach(TypedAttribute::close);
        }
    }

    @Test
    void typedAttributeCopiesItemAndLeavesSourceOwnerOpen() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvItem source = item(StandardAttribute.ACTIVATION_DATE,
                TtlvValue.DateTime(1_700_000_000L), limits);
        try (source;
                TypedAttribute typed = TypedAttribute.fromItem(StandardAttribute.ACTIVATION_DATE, source);
                TtlvItem copy = typed.asItem();
                TtlvItemView view = copy.view();
                Tag tag = view.tag()) {
            assertEquals(StandardAttribute.ACTIVATION_DATE.tag(), tag.raw());
            try (TtlvItemView sourceView = source.view(); Tag sourceTag = sourceView.tag()) {
                assertEquals(StandardAttribute.ACTIVATION_DATE.tag(), sourceTag.raw());
            }
        }
    }

    @Test
    void rejectsWrongTagAndTypeAndKeepsGenericRepeatedAttributes() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvItem wrongTag = item(StandardAttribute.ACTIVATION_DATE,
                TtlvValue.DateTime(17), limits);
        TtlvItem wrongType = item(StandardAttribute.NAME,
                TtlvValue.DateTime(17), limits);
        try (wrongTag; wrongType) {
            assertThrows(InvalidInputException.class,
                    () -> TypedAttribute.fromItem(StandardAttribute.NAME, wrongTag));
            assertThrows(InvalidInputException.class,
                    () -> TypedAttribute.fromItem(StandardAttribute.NAME, wrongType));
        }

        CodecLimits repeatLimits = CodecLimits.defaults();
        TtlvStructure empty = TtlvStructure.create();
        TtlvItem first = item(StandardAttribute.APPLICATION_SPECIFIC_INFORMATION,
                TtlvValue.structure(TtlvStructure.create(), repeatLimits), repeatLimits);
        TtlvStructure one = TtlvStructure.withItem(empty, first, repeatLimits);
        TtlvItem second = item(StandardAttribute.APPLICATION_SPECIFIC_INFORMATION,
                TtlvValue.structure(TtlvStructure.create(), repeatLimits), repeatLimits);
        TtlvStructure repeated = TtlvStructure.withItem(one, second, repeatLimits);
        try (repeated; AttributeSet attributes = AttributeSet.tryNew(repeated)) {
                assertEquals(2, attributes.len());
        }
    }

    @Test
    void unknownGenericItemSurvivesAttributeSetRoundtrip() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvItem unknown = item(0x0054_0001,
                TtlvValue.TextString("opaque-value".getBytes(), limits), limits);
        TtlvStructure empty = TtlvStructure.create();
        TtlvStructure source = TtlvStructure.withItem(empty, unknown, limits);
        try (source; AttributeSet attributes = AttributeSet.tryNew(source)) {
            try (TtlvValue knownValue = TtlvValue.TextString("known-name".getBytes(), limits);
                    TypedAttribute typed = TypedAttribute.create(
                            StandardAttribute.UNIQUE_IDENTIFIER, knownValue)) {
                attributes.tryPushTyped(StandardAttribute.UNIQUE_IDENTIFIER, typed);
            }
            assertEquals(1, attributes.typedCount(StandardAttribute.UNIQUE_IDENTIFIER));
            try (TypedAttribute typed = attributes.typedAt(
                    StandardAttribute.UNIQUE_IDENTIFIER, 0);
                    TtlvValue typedValue = typed.value();
                    TtlvValueView value = typedValue.view()) {
                assertBytes(value, "known-name".getBytes());
            }
            assertEquals(2, attributes.len());
            try (TtlvStructure copied = attributes.asItems();
                    TtlvStructureView view = copied.view()) {
                assertEquals(2, view.itemCount());
                try (TtlvItemView child = view.itemAt(0); Tag tag = child.tag();
                        TtlvValueView value = child.value()) {
                    assertEquals(0x0054_0001, tag.raw());
                    assertBytes(value, "opaque-value".getBytes());
                }
                try (TtlvItemView child = view.itemAt(1); Tag tag = child.tag()) {
                    assertEquals(StandardAttribute.UNIQUE_IDENTIFIER.tag(), tag.raw());
                }
            }
        }
    }

    @Test
    void typedAttributeDiagnosticsRedactSensitiveValues() {
        byte[] marker = "typed-attribute-private-marker".getBytes();
        CodecLimits limits = CodecLimits.defaults();
        try (TtlvValue invalid = TtlvValue.TextString(marker, limits)) {
            InvalidInputException error = assertThrows(InvalidInputException.class,
                    () -> TypedAttribute.create(StandardAttribute.ACTIVATION_DATE, invalid));
            org.junit.jupiter.api.Assertions.assertFalse(error.getMessage().contains(
                    "typed-attribute-private-marker"));
        }
        try (TtlvValue value = TtlvValue.TextString(marker, limits);
                TypedAttribute typed = TypedAttribute.create(
                        StandardAttribute.UNIQUE_IDENTIFIER, value);
                TtlvValue copied = typed.value()) {
            org.junit.jupiter.api.Assertions.assertFalse(typed.toString().contains(
                    "typed-attribute-private-marker"));
            org.junit.jupiter.api.Assertions.assertFalse(copied.toString().contains(
                    "typed-attribute-private-marker"));
        }
    }

    private static TtlvItem item(StandardAttribute attribute, TtlvValue value, CodecLimits limits) {
        return item(attribute.tag(), value, limits);
    }

    private static TtlvItem item(int rawValue, TtlvValue value, CodecLimits limits) {
        try (RawTag raw = RawTag.fromRaw(rawValue); Tag tag = raw.checked()) {
            return TtlvItem.create(tag, value, limits);
        }
    }

    private static TtlvItemType expectedType(StandardAttribute attribute) {
        return switch (attribute.valueKind()) {
            case "BigInteger" -> TtlvItemType.BigInteger;
            case "Boolean" -> TtlvItemType.Boolean;
            case "ByteString" -> TtlvItemType.ByteString;
            case "DateTime" -> TtlvItemType.DateTime;
            case "DateTimeExtended" -> TtlvItemType.DateTimeExtended;
            case "Enumeration" -> TtlvItemType.Enumeration;
            case "Integer" -> TtlvItemType.Integer;
            case "Interval" -> TtlvItemType.Interval;
            case "Structure" -> TtlvItemType.Structure;
            case "TextString", "UniqueIdentifier" -> TtlvItemType.TextString;
            default -> throw new AssertionError("unhandled catalogued attribute type");
        };
    }

    private static TtlvValue valueFor(StandardAttribute attribute, CodecLimits limits) {
        return switch (attribute.valueKind()) {
            case "BigInteger" -> TtlvValue.BigInteger(new byte[] {1}, limits);
            case "Boolean" -> TtlvValue.Boolean(true);
            case "ByteString" -> TtlvValue.ByteString(new byte[] {1}, limits);
            case "DateTime" -> TtlvValue.DateTime(1_700_000_000L);
            case "DateTimeExtended" -> TtlvValue.DateTimeExtended(1_700_000_000L);
            case "Enumeration" -> TtlvValue.Enumeration(0xffff_ffffL);
            case "Integer" -> TtlvValue.Integer(7);
            case "Interval" -> TtlvValue.Interval(7);
            case "Structure" -> TtlvValue.structure(
                    attribute == StandardAttribute.VENDOR_ATTRIBUTE
                            ? vendorAttribute(limits) : TtlvStructure.create(), limits);
            case "TextString", "UniqueIdentifier" -> TtlvValue.TextString(
                    "attribute-value".getBytes(java.nio.charset.StandardCharsets.UTF_8), limits);
            default -> throw new AssertionError("unhandled catalogued attribute type");
        };
    }

    private static TtlvStructure vendorAttribute(CodecLimits limits) {
        TtlvStructure structure = TtlvStructure.create();
        structure = append(structure, 0x0042_009d,
                TtlvValue.TextString("KMIPKit.TestVendor_1".getBytes(), limits), limits);
        structure = append(structure, 0x0042_000a,
                TtlvValue.TextString("Opaque.Attribute".getBytes(), limits), limits);
        structure = append(structure, 0x0042_000b,
                TtlvValue.ByteString(new byte[] {1}, limits), limits);
        return structure;
    }

    private static TtlvStructure append(
            TtlvStructure structure, int rawTag, TtlvValue value, CodecLimits limits) {
        try (RawTag raw = RawTag.fromRaw(rawTag); Tag tag = raw.checked();
                TtlvItem child = TtlvItem.create(tag, value, limits)) {
            return TtlvStructure.withItem(structure, child, limits);
        }
    }
}
