package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assertions.fail;

import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.net.InetAddress;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.security.KeyStore;
import java.security.PrivateKey;
import java.security.cert.X509Certificate;
import java.time.Duration;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.OptionalInt;
import java.util.OptionalLong;
import java.util.concurrent.FutureTask;
import java.util.concurrent.TimeUnit;
import javax.net.ssl.SSLContext;
import javax.net.ssl.SSLServerSocket;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;
import org.kmipkit.extensions.Client;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.ClientConfiguration;
import org.kmipkit.extensions.ClientExtensionRegistry;
import org.kmipkit.extensions.ExtensionRegistryLimits;
import org.kmipkit.extensions.TransportConfiguration;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;

/** RED contract coverage for Java/JNI Hash, MAC, and signature operations. */
final class CryptoOperationsJavaFacadeTest {
    private static final byte[] SECRET_MARKER = "java-crypto-output-must-not-leak".getBytes(StandardCharsets.UTF_8);
    private static final byte[] PENDING_CORRELATION = {0x31, 0x00, (byte) 0xe2};

    private static final List<Operation> OPERATIONS = List.of(
            new Operation("encrypt", 0x1f, new Class<?>[] {
                    UniqueIdentifier.class, TtlvStructure.class, TtlvValue.class,
                    byte[].class, byte[].class, Boolean.class, Boolean.class, byte[].class
            }),
            new Operation("decrypt", 0x20, new Class<?>[] {
                    UniqueIdentifier.class, TtlvStructure.class, TtlvValue.class,
                    byte[].class, byte[].class, byte[].class, Boolean.class, Boolean.class, byte[].class
            }),
            new Operation("hash", 0x27, new Class<?>[] {
                    TtlvStructure.class, TtlvValue.class, byte[].class, Boolean.class, Boolean.class
            }),
            new Operation("mac", 0x23, new Class<?>[] {
                    UniqueIdentifier.class, TtlvStructure.class, TtlvValue.class,
                    byte[].class, Boolean.class, Boolean.class
            }),
            new Operation("macVerify", 0x24, new Class<?>[] {
                    UniqueIdentifier.class, TtlvStructure.class, TtlvValue.class,
                    byte[].class, byte[].class, Boolean.class, Boolean.class
            }),
            new Operation("sign", 0x21, new Class<?>[] {
                    UniqueIdentifier.class, TtlvStructure.class, TtlvValue.class,
                    byte[].class, byte[].class, Boolean.class, Boolean.class
            }),
            new Operation("signatureVerify", 0x22, new Class<?>[] {
                    UniqueIdentifier.class, TtlvStructure.class, TtlvValue.class,
                    byte[].class, byte[].class, byte[].class, Boolean.class, Boolean.class
            }));

    @TempDir
    Path temporaryDirectory;

    @Test
    void operationFactoriesExposeNullableFieldsAndAcceptEveryDataTtlvForm() throws Exception {
        for (Operation operation : OPERATIONS) {
            Method factory = operationFactory(operation);
            assertEquals(ClientBatchItem.class, factory.getReturnType(),
                    operation.name() + " must create a typed batch item");
            long indicatorCount = java.util.Arrays.stream(factory.getParameterTypes())
                    .filter(type -> type == Boolean.class).count();
            assertEquals(2L, indicatorCount,
                    operation.name() + " Init and Final Indicators must distinguish absent from false");

            try (ClientBatchItem omitted = newItem(operation, TtlvValueKind.NONE, false, false);
                    ClientBatchItem present = newItem(operation, TtlvValueKind.BYTE_STRING, true, false)) {
                assertNotNull(omitted);
                assertNotNull(present);
                assertFalse(present.toString().contains(new String(SECRET_MARKER, StandardCharsets.UTF_8)),
                        operation.name() + " must redact input Data from its display form");
            }

            for (TtlvValueKind dataKind : List.of(
                    TtlvValueKind.BYTE_STRING, TtlvValueKind.ENUMERATION, TtlvValueKind.INTEGER)) {
                try (ClientBatchItem item = newItem(operation, dataKind, false, false)) {
                    assertNotNull(item, operation.name() + " must accept Data as " + dataKind);
                }
            }
        }
    }

    @Test
    void successfulPendingAndFailedResponsesPreserveOnlyTypedOutputsAndCopyBytes() throws Exception {
        for (Operation operation : OPERATIONS) {
            operationFactory(operation);
        }

        Path serverStorePath = temporaryDirectory.resolve("crypto-server.p12");
        Path clientStorePath = temporaryDirectory.resolve("crypto-client.p12");
        invokeClientFixture("createKeyStore", new Class<?>[] {Path.class, String.class},
                serverStorePath, "server");
        invokeClientFixture("createKeyStore", new Class<?>[] {Path.class, String.class},
                clientStorePath, "client");
        KeyStore serverStore = (KeyStore) invokeClientFixture(
                "loadKeyStore", new Class<?>[] {Path.class}, serverStorePath);
        KeyStore clientStore = (KeyStore) invokeClientFixture(
                "loadKeyStore", new Class<?>[] {Path.class}, clientStorePath);
        X509Certificate serverCertificate = (X509Certificate) invokeClientFixture(
                "certificate", new Class<?>[] {KeyStore.class, String.class}, serverStore, "server");
        X509Certificate clientCertificate = (X509Certificate) invokeClientFixture(
                "certificate", new Class<?>[] {KeyStore.class, String.class}, clientStore, "client");
        PrivateKey clientKey = (PrivateKey) clientStore.getKey("client", "kmipkit-test".toCharArray());

        SSLContext serverContext = (SSLContext) invokeClientFixture(
                "serverContext", new Class<?>[] {KeyStore.class, X509Certificate.class},
                serverStore, clientCertificate);
        SSLServerSocket listener = (SSLServerSocket) serverContext.getServerSocketFactory()
                .createServerSocket(0, 1, InetAddress.getByName("127.0.0.1"));
        listener.setEnabledProtocols(new String[] {"TLSv1.3"});
        listener.setNeedClientAuth(true);
        listener.setSoTimeout(10_000);

        byte[] responseMessage = mixedOperationResponseMessage();
        FutureTask<byte[]> peer = new FutureTask<>(() -> (byte[]) invokeClientFixture(
                "serveOneResponse", new Class<?>[] {SSLServerSocket.class, byte[].class},
                listener, responseMessage));
        Thread peerThread = new Thread(peer, "kmipkit-java-crypto-result-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityDer(new byte[][] {clientCertificate.getEncoded()}, clientKey.getEncoded())
                        .trustCertificatesDer(new byte[][] {serverCertificate.getEncoded()})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                Client client = Client.create(extensions, transport);
                ClientBatch batch = ClientBatch.create()) {
            batch.asynchronousIndicator(2);
            int index = 0;
            for (Operation operation : OPERATIONS) {
                for (int outcome = 0; outcome < 3; outcome++) {
                    ClientBatchItem item = newItem(operation, TtlvValueKind.BYTE_STRING, false, true)
                            .withUniqueBatchItemId(uniqueBatchItemId(index));
                    batch.add(item);
                    index++;
                }
            }

            try (ClientBatchResponse response = client.execute(batch, CodecLimits.defaults())) {
                assertEquals(OPERATIONS.size() * 3L, response.itemCount());
                index = 0;
                for (Operation operation : OPERATIONS) {
                    for (int outcome = 0; outcome < 3; outcome++) {
                        try (ClientBatchItemResult result = response.itemAt(index)) {
                            assertEquals(outcome == 0 ? 0 : outcome == 1 ? 2 : 1, result.resultStatus());
                            for (Operation candidate : OPERATIONS) {
                                Object typedResult = publicMethod(ClientBatchItemResult.class,
                                        responseGetter(candidate.name())).invoke(result);
                                Optional<?> typedOutput = (Optional<?>) typedResult;
                                if (outcome == 0 && candidate.equals(operation)) {
                                    Object operationResponse = typedOutput.orElseThrow();
                                    assertSuccessfulOutput(operation, operationResponse);
                                } else {
                                    assertTrue(typedOutput.isEmpty(), candidate.name()
                                            + " must not fabricate output for a different operation, Pending, or failure");
                                }
                            }
                        }
                        index++;
                    }
                }
            }
            assertTrue(peer.get(12, TimeUnit.SECONDS).length >= 8);
        } finally {
            listener.close();
        }
    }

    private static ClientBatchItem newItem(
            Operation operation, TtlvValueKind dataKind, boolean includeOptionalSpans, boolean validSinglePart)
            throws Exception {
        Method factory = operationFactory(operation);
        Object[] arguments = new Object[factory.getParameterCount()];
        TtlvStructure parameters = null;
        TtlvValue data = null;
        try {
            Class<?>[] parameterTypes = factory.getParameterTypes();
            for (int index = 0; index < parameterTypes.length; index++) {
                Class<?> type = parameterTypes[index];
                if (type == UniqueIdentifier.class && operation.name().matches("encrypt|decrypt")) {
                    arguments[index] = UniqueIdentifier.textString(operation.name() + "-key");
                } else if (type == TtlvStructure.class && operation.name().equals("hash")) {
                    parameters = hashParameters();
                    arguments[index] = parameters;
                } else if (type == TtlvStructure.class) {
                    parameters = TtlvStructure.create();
                    arguments[index] = parameters;
                } else if (type == TtlvValue.class) {
                    if (dataKind != TtlvValueKind.NONE) {
                        data = dataValue(dataKind);
                        arguments[index] = data;
                    }
                } else if (type == byte[].class && includeOptionalSpans) {
                    arguments[index] = new byte[0];
                } else if (type == byte[].class && validSinglePart && requiredByteInput(operation, index)) {
                    arguments[index] = SECRET_MARKER.clone();
                } else if (type == Boolean.class && validSinglePart) {
                    arguments[index] = Boolean.TRUE;
                }
            }
            return (ClientBatchItem) factory.invoke(null, arguments);
        } catch (InvocationTargetException error) {
            throw new AssertionError(operation.name() + " factory rejected a contract-valid Java input", error.getCause());
        } finally {
            if (data != null) {
                data.close();
            }
            if (parameters != null) {
                parameters.close();
            }
        }
    }

    private static TtlvValue dataValue(TtlvValueKind kind) {
        return switch (kind) {
            case BYTE_STRING -> TtlvValue.ByteString(SECRET_MARKER, CodecLimits.defaults());
            case ENUMERATION -> TtlvValue.Enumeration(0xffff_fffeL);
            case INTEGER -> TtlvValue.Integer(-17);
            case NONE -> throw new IllegalArgumentException("a Data value kind is required");
        };
    }

    private static TtlvStructure hashParameters() {
        try (RawTag rawTag = RawTag.fromRaw(0x420038);
                Tag tag = rawTag.checked();
                TtlvStructure empty = TtlvStructure.create();
                TtlvValue algorithm = TtlvValue.Enumeration(6);
                TtlvItem item = TtlvItem.create(tag, algorithm, CodecLimits.defaults())) {
            return TtlvStructure.withItem(empty, item, CodecLimits.defaults());
        }
    }

    private static Method operationFactory(Operation operation) {
        return publicMethod(ClientBatchItem.class, operation.name(), operation.parameters());
    }

    private static String responseGetter(String operationName) {
        return switch (operationName) {
            case "encrypt" -> "encryptResponse";
            case "decrypt" -> "decryptResponse";
            case "hash" -> "hashResponse";
            case "mac" -> "macResponse";
            case "macVerify" -> "macVerifyResponse";
            case "sign" -> "signResponse";
            case "signatureVerify" -> "signatureVerifyResponse";
            default -> throw new IllegalArgumentException("unknown operation");
        };
    }

    private static Method publicMethod(Class<?> owner, String name, Class<?>... parameterTypes) {
        try {
            return owner.getMethod(name, parameterTypes);
        } catch (NoSuchMethodException missing) {
            fail(owner.getSimpleName() + " must expose public " + name + " with the approved Java contract");
            throw new AssertionError(missing);
        }
    }

    private static boolean requiredByteInput(Operation operation, int parameterIndex) {
        return operation.name().equals("macVerify") && parameterIndex == 3
                || operation.name().equals("signatureVerify") && parameterIndex == 4;
    }

    private static void assertSuccessfulOutput(Operation operation, Object response) throws Exception {
        assertSuccessfulOutput(operation, response, true);
    }

    private static void assertSuccessfulOutput(
            Operation operation, Object response, boolean validityIndicatorPresent) throws Exception {
        assertFalse(response.toString().contains(new String(SECRET_MARKER, StandardCharsets.UTF_8)),
                operation.name() + " response display must redact output bytes");
        if (!operation.name().equals("hash")) {
            Method accessor = publicMethod(response.getClass(), "uniqueIdentifier");
            assertEquals(UniqueIdentifier.class, accessor.getReturnType(),
                    operation.name() + " successful response must require its Unique Identifier");
            Object identifier = accessor.invoke(response);
            assertNotNull(identifier, operation.name() + " success response must expose its Unique Identifier");
            assertFalse(response.toString().contains("response-id-secret"),
                    operation.name() + " response display must redact its Unique Identifier");
            assertFalse(identifier.toString().contains("response-id-secret"),
                    operation.name() + " response display must redact its identifier");
        }

        switch (operation.name()) {
            case "encrypt" -> {
                assertCopiedResponseBytes(response, "data", SECRET_MARKER);
                assertCopiedResponseBytes(response, "ivCounterNonce", "response-iv".getBytes(StandardCharsets.UTF_8));
                assertCopiedResponseBytes(response, "correlationValue", new byte[] {0x01, 0x00, (byte) 0xfd});
                assertCopiedResponseBytes(response, "authenticatedEncryptionTag",
                        "response-tag".getBytes(StandardCharsets.UTF_8));
            }
            case "decrypt" -> {
                assertCopiedResponseBytes(response, "data", SECRET_MARKER);
                assertCopiedResponseBytes(response, "correlationValue", new byte[] {0x01, 0x00, (byte) 0xfd});
            }
            case "hash" -> {
                assertCopiedResponseBytes(response, "data", SECRET_MARKER);
                assertCopiedResponseBytes(response, "correlationValue", new byte[] {0x01, 0x00, (byte) 0xfd});
            }
            case "mac" -> assertCopiedResponseBytes(response, "macData", SECRET_MARKER);
            case "macVerify" -> assertValidityIndicatorPresence(response, validityIndicatorPresent);
            case "sign" -> assertCopiedResponseBytes(response, "signatureData", SECRET_MARKER);
            case "signatureVerify" -> {
                assertValidityIndicatorPresence(response, validityIndicatorPresent);
                assertCopiedResponseBytes(response, "recoveredData", SECRET_MARKER);
            }
            default -> throw new AssertionError("unknown crypto response");
        }
        if (operation.name().matches("encrypt|decrypt")) {
            assertResponseCloseWipesOwnedBytes(response);
        }
    }

    private static void assertCopiedResponseBytes(Object response, String fieldName, byte[] expected) throws Exception {
        Method accessor = publicMethod(response.getClass(), fieldName);
        Object value = accessor.invoke(response);
        assertTrue(value instanceof Optional<?>, fieldName + " must be an optional byte field");
        Optional<?> first = (Optional<?>) value;
        assertArrayEquals(expected, (byte[]) first.orElseThrow());
        byte[] callerCopy = (byte[]) first.orElseThrow();
        callerCopy[0] ^= 0x7f;
        Optional<?> second = (Optional<?>) accessor.invoke(response);
        assertArrayEquals(expected, (byte[]) second.orElseThrow(),
                fieldName + " must be copied from the borrowed JNI span");
    }

    private static void assertResponseCloseWipesOwnedBytes(Object response) throws Exception {
        Method close = publicMethod(response.getClass(), "close");
        close.invoke(response);
        for (String fieldName : response.getClass().getSimpleName().equals("EncryptResponse")
                ? List.of("data", "ivCounterNonce", "correlationValue", "authenticatedEncryptionTag")
                : List.of("data", "correlationValue")) {
            var field = response.getClass().getDeclaredField(fieldName);
            field.setAccessible(true);
            byte[] value = (byte[]) field.get(response);
            if (value != null) assertArrayEquals(new byte[value.length], value,
                    fieldName + " must be wiped when the typed response is closed");
        }
    }

    private static void assertUnknownValidityIndicator(Object response) throws Exception {
        Object raw = publicMethod(response.getClass(), "validityIndicatorRaw").invoke(response);
        if (raw instanceof Optional<?> optional) {
            raw = optional.orElseThrow();
        } else if (raw instanceof OptionalInt optional) {
            raw = optional.orElseThrow();
        } else if (raw instanceof OptionalLong optional) {
            raw = optional.orElseThrow();
        }
        long unsigned = raw instanceof Integer value
                ? Integer.toUnsignedLong(value) : ((Number) raw).longValue();
        assertEquals(0xffff_fffeL, unsigned, "unknown Validity Indicator must remain a raw unsigned value");
    }

    private static void assertValidityIndicatorPresence(Object response, boolean expectedPresent) throws Exception {
        if (expectedPresent) {
            assertUnknownValidityIndicator(response);
            return;
        }
        Object raw = publicMethod(response.getClass(), "validityIndicatorRaw").invoke(response);
        if (raw instanceof Optional<?> optional) {
            assertTrue(optional.isEmpty(), "final multipart response may omit the Validity Indicator");
        } else if (raw instanceof OptionalInt optional) {
            assertTrue(optional.isEmpty(), "final multipart response may omit the Validity Indicator");
        } else if (raw instanceof OptionalLong optional) {
            assertTrue(optional.isEmpty(), "final multipart response may omit the Validity Indicator");
        } else {
            fail("Validity Indicator accessor must preserve optionality");
        }
    }

    private byte[] mixedOperationResponseMessage() throws Exception {
        byte[] version = ttlvStructure(0x420069,
                ttlvItem(0x42006a, 2, ttlvInteger(2)),
                ttlvItem(0x42006b, 2, ttlvInteger(1)));
        byte[] header = ttlvStructure(0x42007a, version,
                ttlvItem(0x420092, 9, ttlvLongInteger(1)),
                ttlvItem(0x42000d, 2, ttlvInteger(OPERATIONS.size() * 3)));
        List<byte[]> responses = new ArrayList<>();
        int index = 0;
        for (Operation operation : OPERATIONS) {
            responses.add(successResponse(operation, index++));
            responses.add(ttlvStructure(0x42000f,
                    ttlvEnumeration(0x42005c, operation.code()),
                    ttlvItem(0x420093, 8, uniqueBatchItemId(index++)),
                    ttlvEnumeration(0x42007f, 2),
                    ttlvItem(0x420006, 8, PENDING_CORRELATION),
                    ttlvStructure(0x42007c)));
            responses.add(ttlvStructure(0x42000f,
                    ttlvEnumeration(0x42005c, operation.code()),
                    ttlvItem(0x420093, 8, uniqueBatchItemId(index++)),
                    ttlvEnumeration(0x42007f, 1),
                    ttlvEnumeration(0x42007e, 1)));
        }
        List<byte[]> messageItems = new ArrayList<>();
        messageItems.add(header);
        messageItems.addAll(responses);
        return ttlvStructure(0x42007b, messageItems.toArray(byte[][]::new));
    }

    private byte[] successResponse(Operation operation, int index) throws Exception {
        return successResponse(operation, index, true);
    }

    private byte[] successResponse(Operation operation, int index, boolean includeValidityIndicator) throws Exception {
        return successResponse(operation, index, includeValidityIndicator, true);
    }

    private byte[] successResponse(Operation operation, int index, boolean includeValidityIndicator,
            boolean includeOptionalFields) throws Exception {
        List<byte[]> fields = new ArrayList<>();
        if (!operation.name().equals("hash")) {
            fields.add(ttlvItem(0x420094, 7, "response-id-secret".getBytes(StandardCharsets.UTF_8)));
        }
        switch (operation.name()) {
            case "encrypt" -> {
                fields.add(ttlvItem(0x4200c2, 8, SECRET_MARKER));
                if (includeOptionalFields) {
                    fields.add(ttlvItem(0x42003d, 8, "response-iv".getBytes(StandardCharsets.UTF_8)));
                    fields.add(ttlvItem(0x4200d6, 8, new byte[] {0x01, 0x00, (byte) 0xfd}));
                    fields.add(ttlvItem(0x4200ff, 8, "response-tag".getBytes(StandardCharsets.UTF_8)));
                }
            }
            case "decrypt" -> {
                fields.add(ttlvItem(0x4200c2, 8, SECRET_MARKER));
                if (includeOptionalFields) {
                    fields.add(ttlvItem(0x4200d6, 8, new byte[] {0x01, 0x00, (byte) 0xfd}));
                }
            }
            case "hash" -> {
                fields.add(ttlvItem(0x4200c2, 8, SECRET_MARKER));
                if (includeOptionalFields) {
                    fields.add(ttlvItem(0x4200d6, 8, new byte[] {0x01, 0x00, (byte) 0xfd}));
                }
            }
            case "mac" -> fields.add(ttlvItem(0x4200c4, 8, SECRET_MARKER));
            case "macVerify" -> {
                if (includeValidityIndicator) fields.add(ttlvEnumeration(0x420128, 0xffff_fffe));
            }
            case "sign" -> fields.add(ttlvItem(0x4200c7, 8, SECRET_MARKER));
            case "signatureVerify" -> {
                if (includeValidityIndicator) fields.add(ttlvEnumeration(0x420128, 0xffff_fffe));
                fields.add(ttlvItem(0x4200c2, 8, SECRET_MARKER));
            }
            default -> throw new AssertionError("unknown crypto response");
        }
        return ttlvStructure(0x42000f,
                ttlvEnumeration(0x42005c, operation.code()),
                ttlvItem(0x420093, 8, uniqueBatchItemId(index)),
                ttlvEnumeration(0x42007f, 0),
                ttlvStructure(0x42007c, fields.toArray(byte[][]::new)));
    }

    private byte[] ttlvStructure(int tag, byte[]... values) throws Exception {
        return (byte[]) invokeClientFixture("structure", new Class<?>[] {int.class, byte[][].class},
                tag, (Object) values);
    }

    private byte[] ttlvItem(int tag, int type, byte[] value) throws Exception {
        return (byte[]) invokeClientFixture("item", new Class<?>[] {int.class, int.class, byte[].class},
                tag, type, value);
    }

    private byte[] ttlvInteger(int value) throws Exception {
        return (byte[]) invokeClientFixture("integer", new Class<?>[] {int.class}, value);
    }

    private byte[] ttlvLongInteger(long value) throws Exception {
        return (byte[]) invokeClientFixture("longInteger", new Class<?>[] {long.class}, value);
    }

    private byte[] ttlvEnumeration(int tag, int value) throws Exception {
        return (byte[]) invokeClientFixture("enumeration", new Class<?>[] {int.class, int.class}, tag, value);
    }

    private byte[] uniqueBatchItemId(int index) throws Exception {
        return (byte[]) invokeClientFixture("uniqueBatchItemId", new Class<?>[] {int.class}, index);
    }

    private static Object invokeClientFixture(String name, Class<?>[] parameterTypes, Object... arguments)
            throws Exception {
        Method method = ClientFacadeTlsIntegrationTest.class.getDeclaredMethod(name, parameterTypes);
        if (!method.trySetAccessible()) {
            throw new AssertionError("TLS integration fixture must remain accessible to crypto facade tests");
        }
        try {
            return method.invoke(null, arguments);
        } catch (InvocationTargetException failure) {
            Throwable cause = failure.getCause();
            if (cause instanceof Exception exception) {
                throw exception;
            }
            throw new AssertionError("TLS integration fixture failed", cause);
        }
    }

    private enum TtlvValueKind { NONE, BYTE_STRING, ENUMERATION, INTEGER }

    private record Operation(String name, int code, Class<?>[] parameters) { }
}
