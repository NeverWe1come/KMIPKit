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
import org.kmipkit.ttlv.TtlvValue;
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

    private static TtlvItem item(StandardAttribute attribute, TtlvValue value, CodecLimits limits) {
        try (RawTag raw = RawTag.fromRaw(attribute.tag()); Tag tag = raw.checked()) {
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
