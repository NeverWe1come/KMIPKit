package org.kmipkit.fluent;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;

import org.junit.jupiter.api.Test;
import org.kmipkit.extensions.AttributeReference;
import org.kmipkit.ttlv.TtlvItem;

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
}
