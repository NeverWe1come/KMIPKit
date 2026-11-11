package org.kmipkit.fluent;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.kmipkit.InvalidInputException;
import org.kmipkit.protocol.KmipValues.CryptographicAlgorithm;
import org.kmipkit.protocol.KmipValues.HashingAlgorithm;
import org.kmipkit.extensions.ClientBatchItem;
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

    @Test
    void allBuildersCreateTheirTypedOperationAndRejectUseAfterClose() {
        UniqueIdentifier id = UniqueIdentifier.textString("crypto-key");
        try (CryptographicParameters parameters = CryptographicParameters.builder()
                        .cryptographicAlgorithm(CryptographicAlgorithm.AES).build();
                CryptographicParameters hashParameters = CryptographicParameters.builder()
                        .hashingAlgorithm(HashingAlgorithm.SHA_256).build();
                EncryptBuilder encrypt = new EncryptBuilder(null);
                DecryptBuilder decrypt = new DecryptBuilder(null);
                SignBuilder sign = new SignBuilder(null);
                SignatureVerifyBuilder signatureVerify = new SignatureVerifyBuilder(null);
                MacBuilder mac = new MacBuilder(null);
                MacVerifyBuilder macVerify = new MacVerifyBuilder(null);
                HashBuilder hash = new HashBuilder(null);
                ClientBatchItem encryptItem = encrypt.uniqueIdentifier(id)
                        .cryptographicParameters(parameters).data(new byte[] {1, 2}).build();
                ClientBatchItem decryptItem = decrypt.uniqueIdentifier(id)
                        .cryptographicParameters(parameters).data(new byte[] {3, 4})
                        .authenticatedEncryptionTag(new byte[] {5}).build();
                ClientBatchItem signItem = sign.uniqueIdentifier(id)
                        .cryptographicParameters(parameters).data(new byte[] {6})
                        .digestedData(new byte[] {7}).build();
                ClientBatchItem signatureVerifyItem = signatureVerify.uniqueIdentifier(id)
                        .cryptographicParameters(parameters).data(new byte[] {8})
                        .signatureData(new byte[] {9}).build();
                ClientBatchItem macItem = mac.uniqueIdentifier(id)
                        .cryptographicParameters(parameters).data(new byte[] {10}).build();
                ClientBatchItem macVerifyItem = macVerify.uniqueIdentifier(id)
                        .cryptographicParameters(parameters).data(new byte[] {11})
                        .macData(new byte[] {12}).build();
                ClientBatchItem hashItem = hash.cryptographicParameters(hashParameters)
                        .data(new byte[] {13}).build()) {
            assertEquals(31, encryptItem.operation());
            assertEquals(32, decryptItem.operation());
            assertEquals(33, signItem.operation());
            assertEquals(34, signatureVerifyItem.operation());
            assertEquals(35, macItem.operation());
            assertEquals(36, macVerifyItem.operation());
            assertEquals(39, hashItem.operation());

            encrypt.close();
            assertThrows(InvalidInputException.class, encrypt::build);
            assertThrows(InvalidInputException.class, () -> decrypt.data(null));
            assertThrows(InvalidInputException.class, () -> hash.withOptions(null));
            assertThrows(InvalidInputException.class, () -> sign.asynchronousIndicator(-1));
            assertThrows(InvalidInputException.class,
                    () -> signatureVerify.asynchronousIndicator(0x1_0000_0000L));
        }
    }

    private static void assertBuilder(String method, String type) throws Exception {
        assertEquals("org.kmipkit.fluent." + type,
                FluentRequest.class.getMethod(method).getReturnType().getName());
    }
}
