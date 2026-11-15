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
import org.kmipkit.ttlv.StandardAttribute;
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
            assertThrows(InvalidInputException.class, () -> builder.uniqueIdentifier(null));
            assertThrows(InvalidInputException.class,
                    () -> builder.cataloguedAttribute((StandardAttribute) null, description));
        }
    }
}
