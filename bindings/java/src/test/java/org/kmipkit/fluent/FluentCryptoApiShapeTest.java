package org.kmipkit.fluent;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;

import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.protocol.CryptographicParameters;
import org.kmipkit.ttlv.TtlvStructure;
import org.junit.jupiter.api.Test;

/** Pins the Java names and operation-specific surface of the fluent crypto facade. */
final class FluentCryptoApiShapeTest {
    @Test
    void requestEntryPointsReturnOperationSpecificBuilders() throws Exception {
        assertBuilder("encrypt", "EncryptBuilder");
        assertBuilder("decrypt", "DecryptBuilder");
        assertBuilder("sign", "SignBuilder");
        assertBuilder("signatureVerify", "SignatureVerifyBuilder");
        assertBuilder("mac", "MacBuilder");
        assertBuilder("macVerify", "MacVerifyBuilder");
        assertBuilder("hash", "HashBuilder");
    }

    @Test
    void everyOperationHasExplicitParametersAndOneExchangeLifecycle() throws Exception {
        for (String type : new String[] {
                "EncryptBuilder", "DecryptBuilder", "SignBuilder", "SignatureVerifyBuilder",
                "MacBuilder", "MacVerifyBuilder", "HashBuilder"}) {
            Class<?> builder = Class.forName("org.kmipkit.fluent." + type);
            assertNotNull(builder.getMethod("cryptographicParameters", CryptographicParameters.class));
            assertNotNull(builder.getMethod("cryptographicParametersStructure", TtlvStructure.class));
            assertNotNull(builder.getMethod("withOptions", RequestOptions.class));
            assertNotNull(builder.getMethod("asynchronousIndicator", long.class));
            assertNotNull(builder.getMethod("build"));
            assertNotNull(builder.getMethod("toBatchItem"));
            assertNotNull(builder.getMethod("execute", org.kmipkit.ttlv.CodecLimits.class));
            assertNotNull(builder.getMethod("close"));
        }
        assertNotNull(EncryptBuilder.class.getMethod("uniqueIdentifier", UniqueIdentifier.class));
        assertNotNull(DecryptBuilder.class.getMethod("authenticatedEncryptionTag", byte[].class));
        assertNotNull(SignBuilder.class.getMethod("digestedData", byte[].class));
        assertNotNull(SignatureVerifyBuilder.class.getMethod("signatureData", byte[].class));
        assertNotNull(MacVerifyBuilder.class.getMethod("macData", byte[].class));
    }

    private static void assertBuilder(String method, String type) throws Exception {
        assertEquals("org.kmipkit.fluent." + type,
                FluentRequest.class.getMethod(method).getReturnType().getName());
    }
}
