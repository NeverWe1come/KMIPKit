package org.kmipkit.fluent;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.nio.charset.StandardCharsets;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.SetAttributeRequest;
import org.kmipkit.protocol.KmipValues;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.StandardAttribute;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;

final class SetAttributeBuilderTest {
    @Test
    void fluentRequestExposesSetAttributeBuilderAndTypedBatchItem() throws Exception {
        assertEquals(SetAttributeBuilder.class,
                FluentRequest.class.getMethod("setAttribute").getReturnType());
        assertNotNull(SetAttributeBuilder.class.getMethod("cataloguedAttribute", StandardAttribute.class,
                TtlvValue.class));

        CodecLimits limits = CodecLimits.defaults();
        try (TtlvValue description = TtlvValue.TextString(
                        "managed by the fluent API".getBytes(StandardCharsets.UTF_8), limits);
                RawTag rawTag = RawTag.fromRaw(0x4200fc);
                Tag tag = rawTag.checked();
                TtlvValue expertValue = TtlvValue.TextString(
                        "expert value".getBytes(StandardCharsets.UTF_8), limits);
                TtlvItem expertItem = TtlvItem.create(tag, expertValue, limits);
                SetAttributeBuilder builder = new SetAttributeBuilder(null)) {
            assertThrows(InvalidInputException.class, builder::validate);
            assertSame(builder, builder.uniqueIdentifier("key-42"));
            assertSame(builder, builder.cataloguedAttribute(StandardAttribute.DESCRIPTION, description));
            assertSame(builder, builder.withOptions(RequestOptions.defaults()));
            assertSame(builder, builder.asynchronousIndicator(3));
            builder.validate();
            try (ClientBatchItem item = builder.build()) {
                assertEquals(KmipValues.Operation.SET_ATTRIBUTE, item.operation());
            }
            try (SetAttributeRequest request = builder.buildRequest();
                    TtlvStructure payload = request.toTtlvPayload();
                    ClientBatchItem item = request.toBatchItem()) {
                assertNotNull(payload);
                assertEquals(KmipValues.Operation.SET_ATTRIBUTE, item.operation());
            }
            try (ClientBatchItem item = builder.toBatchItem()) {
                assertEquals(KmipValues.Operation.SET_ATTRIBUTE, item.operation());
            }
            assertThrows(InvalidInputException.class, () -> builder.uniqueIdentifier(null));
            assertThrows(InvalidInputException.class,
                    () -> builder.cataloguedAttribute((StandardAttribute) null, description));
            assertThrows(InvalidInputException.class, () -> builder.cataloguedAttribute(tag, null));
            assertThrows(InvalidInputException.class, () -> builder.vendorAttribute(null, "region", expertItem));
            assertThrows(InvalidInputException.class, () -> builder.withOptions(null));
            assertThrows(InvalidInputException.class, () -> builder.asynchronousIndicator(-1));
            assertThrows(InvalidInputException.class, () -> builder.asynchronousIndicator(0x1_0000_0000L));
            assertThrows(InvalidInputException.class, () -> builder.execute(null));
            assertThrows(InvalidInputException.class, () -> builder.execute(limits));
        }
        try (RawTag rawTag = RawTag.fromRaw(0x4200fc);
                Tag tag = rawTag.checked();
                TtlvValue value = TtlvValue.TextString(
                        "tag overload".getBytes(StandardCharsets.UTF_8), limits);
                SetAttributeBuilder builder = new SetAttributeBuilder(null)) {
            builder.uniqueIdentifier("key-42").cataloguedAttribute(tag, value);
            builder.validate();
        }
        try (RawTag rawTag = RawTag.fromRaw(0x4200fc);
                Tag tag = rawTag.checked();
                TtlvValue value = TtlvValue.TextString(
                        "expert value".getBytes(StandardCharsets.UTF_8), limits);
                TtlvItem item = TtlvItem.create(tag, value, limits);
                SetAttributeBuilder builder = new SetAttributeBuilder(null)) {
            builder.uniqueIdentifier("key-42").attributeItem(item);
            builder.validate();
            assertThrows(InvalidInputException.class, () -> builder.attributeItem(null));
        }
        try (RawTag rawTag = RawTag.fromRaw(0x540001);
                Tag tag = rawTag.checked();
                TtlvValue value = TtlvValue.TextString(
                        "vendor value".getBytes(StandardCharsets.UTF_8), limits);
                TtlvItem item = TtlvItem.create(tag, value, limits);
                SetAttributeBuilder builder = new SetAttributeBuilder(null)) {
            builder.uniqueIdentifier("key-42").vendorAttribute("Vendor_Name.1", "region", item);
            builder.validate();
            try (SetAttributeRequest request = builder.buildRequest()) {
                try (TtlvStructure payload = request.toTtlvPayload()) {
                    assertNotNull(payload);
                }
            }
        }
    }
}
