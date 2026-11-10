package org.kmipkit.fluent;

import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertEquals;

import org.kmipkit.InvalidInputException;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.protocol.KmipValues.CryptographicUsageMask;
import org.junit.jupiter.api.Test;

/** Checks the Java package shape while preserving the established extensions API. */
final class FluentFacadeShapeTest {
    @Test
    void exposesTaskEntryPointAndRetainsLegacyClientType() throws Exception {
        assertNotNull(Class.forName("org.kmipkit.client.KmipClient"));
        assertNotNull(Class.forName("org.kmipkit.client.SymmetricKeyRequest"));
        assertNotNull(Class.forName("org.kmipkit.fluent.SymmetricKeyBuilder"));
        assertNotNull(Class.forName("org.kmipkit.extensions.Client"));
    }

    @Test
    void buildersExposeBuildBatchAndExecutionMethods() throws Exception {
        assertNotNull(SymmetricKeyBuilder.class.getMethod("build"));
        assertNotNull(SymmetricKeyBuilder.class.getMethod("toBatchItem"));
        assertNotNull(SymmetricKeyBuilder.class.getMethod("withOptions",
                org.kmipkit.extensions.RequestOptions.class));
        assertNotNull(SymmetricKeyBuilder.class.getMethod("asynchronousIndicator", long.class));
        assertNotNull(OperationBuilder.class.getMethod("asynchronousIndicator", long.class));
        assertNotNull(OperationBuilder.class.getMethod("execute",
                org.kmipkit.ttlv.CodecLimits.class));
    }

    @Test
    void kmipClientExposesNamedOperationConveniences() throws Exception {
        Class<?> client = Class.forName("org.kmipkit.client.KmipClient");
        Class<?> request = Class.forName("org.kmipkit.client.SymmetricKeyRequest");
        assertNotNull(client.getMethod("createSymmetricKey", request,
                org.kmipkit.ttlv.CodecLimits.class));
        assertNotNull(client.getMethod("createSymmetricKey", request,
                org.kmipkit.ttlv.CodecLimits.class, org.kmipkit.extensions.RequestOptions.class));
        assertNotNull(client.getMethod("getById",
                org.kmipkit.extensions.UniqueIdentifier.class, org.kmipkit.ttlv.CodecLimits.class));
        assertNotNull(client.getMethod("getById",
                org.kmipkit.extensions.UniqueIdentifier.class, org.kmipkit.ttlv.CodecLimits.class,
                org.kmipkit.extensions.RequestOptions.class));
        assertNotNull(client.getMethod("destroyById",
                org.kmipkit.extensions.UniqueIdentifier.class, org.kmipkit.ttlv.CodecLimits.class));
        assertNotNull(client.getMethod("destroyById",
                org.kmipkit.extensions.UniqueIdentifier.class, org.kmipkit.ttlv.CodecLimits.class,
                org.kmipkit.extensions.RequestOptions.class));
    }

    @Test
    void createBuilderUsesRustValidationAndProducesTypedCreateItem() {
        try (SymmetricKeyBuilder builder = new SymmetricKeyBuilder(null)) {
            assertThrows(InvalidInputException.class, builder::build);
            assertThrows(InvalidInputException.class,
                    () -> builder.aes256().build());
            assertThrows(InvalidInputException.class, () -> builder.name(null));
            assertThrows(InvalidInputException.class, () -> builder.withOptions(null));
            assertThrows(InvalidInputException.class, () -> new OperationBuilder(null, null, false));
        }
        try (SymmetricKeyBuilder builder = new SymmetricKeyBuilder(null)) {
            builder.aes256()
                    .usageMask((int) (CryptographicUsageMask.ENCRYPT | CryptographicUsageMask.DECRYPT))
                    .name("payments-key");
            try (ClientBatchItem item = builder.toBatchItem()) {
                assertNotNull(item);
                assertEquals(1, item.operation());
            }
            builder.close();
            assertThrows(InvalidInputException.class, builder::build);
        }
    }
}
