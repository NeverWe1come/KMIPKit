package org.kmipkit.fluent;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.nio.charset.StandardCharsets;

import org.junit.jupiter.api.Test;
import org.kmipkit.ClientOperationException;
import org.kmipkit.InvalidInputException;
import org.kmipkit.extensions.AttributeReference;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.DeleteAttributeRequest;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.protocol.KmipValues;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.StandardAttribute;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;

final class DeleteAttributeBuilderTest {
    @Test
    void fluentRequestExposesTypedDeleteAttributeBuilder() throws Exception {
        assertEquals(DeleteAttributeBuilder.class,
                FluentRequest.class.getMethod("deleteAttribute").getReturnType());
        assertNotNull(DeleteAttributeBuilder.class.getMethod("currentAttribute", TtlvItem.class));
        assertNotNull(DeleteAttributeBuilder.class.getMethod(
                "attributeReference", AttributeReference.class));
        assertNotNull(DeleteAttributeBuilder.class.getMethod("buildRequest"));
        assertNotNull(DeleteAttributeBuilder.class.getMethod("toBatchItem"));
    }

    @Test
    void detachedBuilderCopiesCataloguedCurrentAttributeIntoTypedRequest() {
        CodecLimits limits = CodecLimits.defaults();
        try (TtlvValue description = TtlvValue.TextString(
                        "retired description".getBytes(StandardCharsets.UTF_8), limits);
                DeleteAttributeBuilder builder = new DeleteAttributeBuilder(null)) {
            AttributeReference reference = AttributeReference.tag(StandardAttribute.DESCRIPTION.tag());
            assertSame(builder, builder.uniqueIdentifier("delete-attribute-key"));
            assertSame(builder, builder.cataloguedAttribute(StandardAttribute.DESCRIPTION, description));
            assertSame(builder, builder.attributeReference(reference));
            builder.validate();

            try (DeleteAttributeRequest request = builder.buildRequest();
                    TtlvStructure payload = request.toTtlvPayload();
                    TtlvItem current = request.currentAttribute().orElseThrow()) {
                assertEquals("delete-attribute-key", request.uniqueIdentifier().orElseThrow().text());
                assertEquals(reference, request.attributeReference().orElseThrow());
                try (var view = payload.view()) {
                    assertEquals(3, view.itemCount());
                }
                assertNotNull(current);
            }
            try (var item = builder.toBatchItem()) {
                assertEquals(KmipValues.Operation.DELETE_ATTRIBUTE, item.operation());
            }
        }
    }

    @Test
    void detachedBuilderCoversExplicitVendorAndNamedAttributePaths() {
        CodecLimits limits = CodecLimits.defaults();
        try (RawTag rawTag = RawTag.fromRaw(0x0042_00fc);
                Tag tag = rawTag.checked();
                TtlvValue value = TtlvValue.TextString(
                        "vendor value".getBytes(StandardCharsets.UTF_8), limits);
                TtlvItem item = TtlvItem.create(tag, value, limits);
                DeleteAttributeBuilder builder = new DeleteAttributeBuilder(null)) {
            assertSame(builder, builder.uniqueIdentifier("key-42"));
            assertSame(builder, builder.currentAttribute(item));
            assertSame(builder, builder.attributeReference(AttributeReference.name("KMIPKit", "Region")));
            assertSame(builder, builder.withOptions(RequestOptions.defaults()));
            assertSame(builder, builder.asynchronousIndicator(3));
            builder.validate();
            try (DeleteAttributeRequest request = builder.buildRequest();
                    ClientBatchItem batchItem = builder.toBatchItem()) {
                assertEquals(AttributeReference.name("KMIPKit", "Region"),
                        request.attributeReference().orElseThrow());
                assertEquals(KmipValues.Operation.DELETE_ATTRIBUTE, batchItem.operation());
            }
            assertThrows(InvalidInputException.class, () -> builder.currentAttribute(null));
            assertThrows(InvalidInputException.class,
                    () -> builder.cataloguedAttribute((Tag) null, value));
            assertThrows(InvalidInputException.class,
                    () -> builder.cataloguedAttribute(StandardAttribute.DESCRIPTION, null));
            assertThrows(InvalidInputException.class,
                    () -> builder.vendorAttribute(null, "region", item));
            assertThrows(InvalidInputException.class,
                    () -> builder.attributeReference(null));
            assertThrows(InvalidInputException.class, () -> builder.withOptions(null));
            assertThrows(InvalidInputException.class, () -> builder.asynchronousIndicator(-1));
            assertThrows(InvalidInputException.class,
                    () -> builder.asynchronousIndicator(0x1_0000_0000L));
            assertThrows(InvalidInputException.class, () -> builder.execute(limits));
        }
        try (RawTag rawTag = RawTag.fromRaw(0x0042_00fc);
                Tag tag = rawTag.checked();
                TtlvValue value = TtlvValue.TextString(
                        "north".getBytes(StandardCharsets.UTF_8), limits);
                TtlvItem item = TtlvItem.create(tag, value, limits);
                DeleteAttributeBuilder builder = new DeleteAttributeBuilder(null)) {
            assertSame(builder, builder.currentVendorAttribute("Vendor_Name.1", "region", item));
            builder.validate();
            try (DeleteAttributeRequest request = builder.buildRequest();
                    TtlvItem currentAttribute = request.currentAttribute().orElseThrow()) {
                try (TtlvItemView view = currentAttribute.view(); Tag currentTag = view.tag()) {
                    assertEquals(0x0042_0008, currentTag.raw());
                }
            }
            assertThrows(InvalidInputException.class,
                    () -> builder.currentVendorAttribute("vendor", null, item));
        }
    }

    @Test
    void deleteAttributeJniRejectsMalformedNativeArguments() {
        NativeFluentRegistry.ensureLoaded();
        assertThrows(InvalidInputException.class,
                () -> NativeFluentRegistry.deleteAttributeBuilderSetAttributeReference(
                        0, 99, 0, null, null));
        assertThrows(InvalidInputException.class,
                () -> NativeFluentRegistry.deleteAttributeBuilderSetAttributeReference(
                        0, 1, 0, new byte[] {1}, null));
        assertThrows(InvalidInputException.class,
                () -> NativeFluentRegistry.deleteAttributeBuilderSetAsynchronousIndicator(0, -1));
        assertThrows(ClientOperationException.class,
                () -> NativeFluentRegistry.deleteAttributeBuilderBuildRequest(0, null, null));
        assertThrows(ClientOperationException.class,
                () -> NativeFluentRegistry.deleteAttributeBuilderBuildBatchItem(0));
    }
}
