package org.kmipkit.fluent;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertSame;

import java.nio.charset.StandardCharsets;

import org.junit.jupiter.api.Test;
import org.kmipkit.extensions.AttributeReference;
import org.kmipkit.extensions.DeleteAttributeRequest;
import org.kmipkit.protocol.KmipValues;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.StandardAttribute;
import org.kmipkit.ttlv.TtlvItem;
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
}
