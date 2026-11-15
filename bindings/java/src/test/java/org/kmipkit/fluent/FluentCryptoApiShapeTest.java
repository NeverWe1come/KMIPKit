package org.kmipkit.fluent;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.protocol.KmipValues.CryptographicAlgorithm;
import org.kmipkit.protocol.KmipValues.HashingAlgorithm;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.protocol.CryptographicParameters;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;
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
            sign.close();
            signatureVerify.close();
            assertThrows(InvalidInputException.class, encrypt::build);
            assertThrows(InvalidInputException.class, sign::build);
            assertThrows(InvalidInputException.class, signatureVerify::build);
            assertThrows(InvalidInputException.class, () -> decrypt.data(null));
            assertThrows(InvalidInputException.class, () -> hash.withOptions(null));
            assertThrows(InvalidInputException.class, () -> sign.asynchronousIndicator(-1));
            assertThrows(InvalidInputException.class,
                    () -> signatureVerify.asynchronousIndicator(0x1_0000_0000L));
        }
    }

    @Test
    void buildersCoverTypedInputsAndRejectMissingLocalArguments() {
        try (CryptographicParameters parameters = CryptographicParameters.builder()
                        .cryptographicAlgorithm(CryptographicAlgorithm.AES).build();
                TtlvStructure parameterStructure = TtlvStructure.create();
                TtlvValue dataValue = TtlvValue.ByteString(new byte[] {1, 2}, CodecLimits.defaults());
                EncryptBuilder encrypt = new EncryptBuilder(null);
                EncryptBuilder aliasBuilder = new EncryptBuilder(null);
                DecryptBuilder decrypt = new DecryptBuilder(null);
                SignBuilder sign = new SignBuilder(null);
                SignatureVerifyBuilder verify = new SignatureVerifyBuilder(null);
                MacVerifyBuilder macVerify = new MacVerifyBuilder(null)) {
            encrypt.uniqueIdentifier(UniqueIdentifier.enumeration(7))
                    .cryptographicParametersStructure(parameterStructure)
                    .dataValue(dataValue)
                    .ivCounterNonce(new byte[] {4})
                    .authenticatedEncryptionAdditionalData(new byte[] {5})
                    .correlationValue(new byte[] {6})
                    .initIndicator(false)
                    .finalIndicator(true);
            decrypt.authenticatedEncryptionTag(new byte[] {7});
            sign.uniqueIdentifier(UniqueIdentifier.integer(-8)).digestedData(new byte[] {9});
            verify.signatureData(new byte[] {10});
            macVerify.macData(new byte[] {11});

            assertThrows(InvalidInputException.class, () -> encrypt.cryptographicParameters(null));
            assertThrows(InvalidInputException.class, () -> encrypt.cryptographicParametersStructure(null));
            assertThrows(InvalidInputException.class, () -> encrypt.dataValue(null));
            assertThrows(InvalidInputException.class, () -> encrypt.uniqueIdentifier(null));
            assertThrows(InvalidInputException.class, () -> encrypt.execute(null));
            assertThrows(InvalidInputException.class, () -> encrypt.execute(CodecLimits.defaults()));
            try (ClientBatchItem item = aliasBuilder.uniqueIdentifier(UniqueIdentifier.textString("alias"))
                    .cryptographicParameters(parameters).data(new byte[] {3}).toBatchItem()) {
                assertEquals(31, item.operation());
            }
        }
    }

    @Test
    void nativeCryptoBridgeCoversEachByteSelectorAndRejectsInvalidSelectors() {
        NativeFluentRegistry.ensureLoaded();
        assertThrows(InvalidInputException.class,
                () -> NativeFluentRegistry.cryptoOperationBuilderCreate(7));

        int[] operationsByField = {0, 0, 0, 1, 2, 3, 5, 0};
        for (int field = 0; field < operationsByField.length; field++) {
            long builder = NativeFluentRegistry.cryptoOperationBuilderCreate(operationsByField[field]);
            try {
                NativeFluentRegistry.cryptoOperationBuilderSetBytes(builder, field, new byte[] {1});
            } finally {
                NativeExtensionRegistry.release(NativeFluentRegistry.CRYPTO_OPERATION_BUILDER, builder);
            }
        }

        long builder = NativeFluentRegistry.cryptoOperationBuilderCreate(0);
        try {
            NativeFluentRegistry.cryptoOperationBuilderSetUniqueIdentifier(builder, 1, new byte[] {65}, 0);
            NativeFluentRegistry.cryptoOperationBuilderSetUniqueIdentifier(builder, 2, null, 7);
            NativeFluentRegistry.cryptoOperationBuilderSetUniqueIdentifier(builder, 3, null, -8);
            NativeFluentRegistry.cryptoOperationBuilderSetIndicator(builder, 0, false);
            NativeFluentRegistry.cryptoOperationBuilderSetIndicator(builder, 1, true);
            assertThrows(InvalidInputException.class,
                    () -> NativeFluentRegistry.cryptoOperationBuilderSetUniqueIdentifier(builder, 4, null, 0));
            assertThrows(InvalidInputException.class,
                    () -> NativeFluentRegistry.cryptoOperationBuilderSetBytes(builder, 8, new byte[] {1}));
            assertThrows(InvalidInputException.class,
                    () -> NativeFluentRegistry.cryptoOperationBuilderSetBytes(builder, 0, null));
            assertThrows(InvalidInputException.class,
                    () -> NativeFluentRegistry.cryptoOperationBuilderSetIndicator(builder, 2, false));
        } finally {
            NativeExtensionRegistry.release(NativeFluentRegistry.CRYPTO_OPERATION_BUILDER, builder);
        }
    }

    private static void assertBuilder(String method, String type) throws Exception {
        assertEquals("org.kmipkit.fluent." + type,
                FluentRequest.class.getMethod(method).getReturnType().getName());
    }
}
