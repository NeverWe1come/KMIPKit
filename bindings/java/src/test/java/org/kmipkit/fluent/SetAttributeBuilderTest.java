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
import org.kmipkit.protocol.KmipValues;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.StandardAttribute;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvValue;

final class SetAttributeBuilderTest {
    @Test
    void fluentRequestExposesSetAttributeBuilderAndTypedBatchItem() throws Exception {
        assertEquals(SetAttributeBuilder.class,
                FluentRequest.class.getMethod("setAttribute").getReturnType());
        assertNotNull(SetAttributeBuilder.class.getMethod("cataloguedAttribute", Tag.class,
                TtlvValue.class));

        CodecLimits limits = CodecLimits.defaults();
        try (RawTag raw = RawTag.fromRaw(StandardAttribute.DESCRIPTION.tag());
                Tag tag = raw.checked();
                TtlvValue description = TtlvValue.TextString(
                        "managed by the fluent API".getBytes(StandardCharsets.UTF_8), limits);
                SetAttributeBuilder builder = new SetAttributeBuilder(null)) {
            assertThrows(InvalidInputException.class, builder::validate);
            assertSame(builder, builder.uniqueIdentifier("key-42"));
            assertSame(builder, builder.cataloguedAttribute(tag, description));
            assertSame(builder, builder.withOptions(RequestOptions.defaults()));
            assertSame(builder, builder.asynchronousIndicator(3));
            builder.validate();
            try (ClientBatchItem item = builder.build()) {
                assertEquals(KmipValues.Operation.SET_ATTRIBUTE, item.operation());
            }
            assertThrows(InvalidInputException.class, () -> builder.uniqueIdentifier(null));
            assertThrows(InvalidInputException.class,
                    () -> builder.cataloguedAttribute(null, description));
        }
    }
}
