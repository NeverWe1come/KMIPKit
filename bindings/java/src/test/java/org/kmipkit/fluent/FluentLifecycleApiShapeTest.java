package org.kmipkit.fluent;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;

import java.lang.reflect.Method;
import org.junit.jupiter.api.Test;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.RevocationReason;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.CodecLimits;

/** Verifies the public Java lifecycle facade signatures. */
final class FluentLifecycleApiShapeTest {
    @Test
    void requestNamespaceProvidesAllLifecycleBuilders() throws Exception {
        Method activate = FluentRequest.class.getMethod("activate", UniqueIdentifier.class);
        Method archive = FluentRequest.class.getMethod("archive", UniqueIdentifier.class);
        Method recover = FluentRequest.class.getMethod("recover", UniqueIdentifier.class);
        Method locate = FluentRequest.class.getMethod("locate", AttributeSet.class);
        Method revoke = FluentRequest.class.getMethod(
                "revoke", UniqueIdentifier.class, RevocationReason.class);

        assertEquals(OperationBuilder.class, activate.getReturnType());
        assertEquals(OperationBuilder.class, archive.getReturnType());
        assertEquals(OperationBuilder.class, recover.getReturnType());
        assertEquals("org.kmipkit.fluent.LocateBuilder", locate.getReturnType().getName());
        assertEquals("org.kmipkit.fluent.RevokeBuilder", revoke.getReturnType().getName());
    }

    @Test
    void namedHelpersExposeExchangeOptionsOverloads() throws Exception {
        Class<?> client = KmipClient.class;
        Class<?>[][] required = {
            {UniqueIdentifier.class, CodecLimits.class},
            {UniqueIdentifier.class, CodecLimits.class, RequestOptions.class},
            {AttributeSet.class, CodecLimits.class},
            {AttributeSet.class, CodecLimits.class, RequestOptions.class},
            {UniqueIdentifier.class, RevocationReason.class, Long.class, CodecLimits.class},
            {UniqueIdentifier.class, RevocationReason.class, Long.class,
                    CodecLimits.class, RequestOptions.class}
        };
        String[] names = {
            "activateById", "activateById", "locate", "locate", "revokeById", "revokeById"
        };
        for (int index = 0; index < names.length; index++) {
            assertNotNull(client.getMethod(names[index], required[index]));
        }
        assertNotNull(client.getMethod("archiveById", UniqueIdentifier.class, CodecLimits.class));
        assertNotNull(client.getMethod("archiveById", UniqueIdentifier.class,
                CodecLimits.class, RequestOptions.class));
        assertNotNull(client.getMethod("recoverById", UniqueIdentifier.class, CodecLimits.class));
        assertNotNull(client.getMethod("recoverById", UniqueIdentifier.class,
                CodecLimits.class, RequestOptions.class));
    }
}
