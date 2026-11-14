package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.ByteArrayOutputStream;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.net.InetAddress;
import java.net.ServerSocket;
import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.util.List;
import java.util.concurrent.FutureTask;
import java.util.concurrent.TimeUnit;
import javax.net.ssl.SSLServerSocket;
import org.junit.jupiter.api.Test;
import org.kmipkit.client.KmipClient;
import org.kmipkit.credentials.SecretBytes;
import org.kmipkit.credentials.SecretText;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.ClientConfiguration;
import org.kmipkit.extensions.ClientExtensionRegistry;
import org.kmipkit.extensions.ExtensionRegistryLimits;
import org.kmipkit.extensions.ProtocolVersion;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.TransportConfiguration;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.fluent.CreateCredentialObject;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.TtlvStructure;

/** Executes KMIP 3.0 fluent builders through a controlled TLS 1.3 peer. */
final class Kmip30FluentExecutionTest {
    private static final int CREATE_GROUP = 0x3c;
    private static final byte[] CORRELATION = "pending-correlation".getBytes(StandardCharsets.US_ASCII);
    private static final String FAILURE_MESSAGE = "private-response-diagnostic";
    private static final byte[] DECAPSULATE_SENTINEL =
            "server-managed-shared-secret".getBytes(StandardCharsets.US_ASCII);

    @Test
    void everyAssignedRouteValidatesAndBuildsItsTypedOperation() throws Exception {
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration configuration = ClientConfiguration.create(registry);
                TransportConfiguration transport = transport(1);
                KmipClient client = KmipClient.create(configuration, transport);
                TtlvStructure source = TtlvStructure.create();
                AttributeSet attributes = AttributeSet.tryNew(source);
                SecretText password = SecretText.fromString("credential-secret");
                CreateCredentialObject credential = CreateCredentialObject.password(
                        password, null, null, null, null)) {
            UniqueIdentifier identifier = UniqueIdentifier.textString("kmip30-target");
            try (var createCredential = client.request().createCredential(attributes, credential);
                    var createGroup = client.request().createGroup(attributes);
                    var createUser = client.request().createUser(attributes);
                    var deactivate = client.request().deactivate(identifier);
                    var decapsulate = client.request().decapsulate(identifier, new byte[] {1, 2, 3});
                    var encapsulate = client.request().encapsulate(identifier);
                    var obliterate = client.request().obliterate(identifier)) {
                List<org.kmipkit.fluent.Kmip30BuilderBase<?>> builders = List.of(
                        createCredential, createGroup, createUser, deactivate,
                        decapsulate, encapsulate, obliterate);
                int[] operations = {0x3f, 0x3c, 0x3e, 0x40, 0x42, 0x41, 0x3d};
                for (int index = 0; index < builders.size(); index++) {
                    builders.get(index).validate();
                    try (var item = builders.get(index).toBatchItem()) {
                        assertEquals(operations[index], item.operation());
                    }
                }
            }
            assertFalse(credential.toString().contains("credential-secret"));
        }
    }

    @Test
    void defaultVersionRejectsKmip30RouteBeforeSending() throws Exception {
        int unusedPort;
        try (ServerSocket reservation = new ServerSocket(0, 1, InetAddress.getByName("127.0.0.1"))) {
            unusedPort = reservation.getLocalPort();
        }
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration configuration = ClientConfiguration.create(registry);
                TransportConfiguration transport = transport(unusedPort);
                KmipClient client = KmipClient.create(configuration, transport);
                TtlvStructure source = TtlvStructure.create();
                AttributeSet attributes = AttributeSet.tryNew(source)) {
            ClientOperationException failure = assertThrows(ClientOperationException.class,
                    () -> client.request().createGroup(attributes).execute(CodecLimits.defaults()));
            assertEquals(1, failure.deliveryStateCode(), "version gate must report NotSent");
            assertFalse(failure.getMessage().contains(FAILURE_MESSAGE));
        }
    }

    @Test
    void executeUsesOneExchangeAndPreservesSuccessPendingFailureMetadata() throws Exception {
        byte[] serverCertificate = fixture("server-cert.pem");
        byte[] serverKey = fixture("server-key.pem");
        byte[] clientCertificate = fixture("client-cert.pem");
        byte[] clientKey = fixture("client-key.pem");
        SSLServerSocket listener = (SSLServerSocket) fixtureCall(
                "deterministicMtlsListener",
                new Class<?>[] {byte[].class, byte[].class, java.security.cert.X509Certificate.class},
                serverCertificate, serverKey,
                fixtureCall("testCertificate", new Class<?>[] {byte[].class}, (Object) clientCertificate));
        int[] operations = {0x3f, 0x3c, 0x3e, 0x40, 0x42, 0x41, 0x3d, CREATE_GROUP, CREATE_GROUP};
        byte[][] responses = new byte[operations.length][];
        for (int index = 0; index < 7; index++) {
            responses[index] = response(operations[index], 0, null, null,
                    operations[index] != 0x3d);
        }
        responses[4] = response(0x42, 0, null, null, true);
        responses[7] = response(CREATE_GROUP, 2, null, null, false);
        responses[8] = response(CREATE_GROUP, 1, 1, FAILURE_MESSAGE, false);
        FutureTask<List<byte[]>> peer = new FutureTask<>(() -> {
            @SuppressWarnings("unchecked")
            List<byte[]> requests = (List<byte[]>) fixtureCall("serveResponses",
                    new Class<?>[] {SSLServerSocket.class, byte[][].class}, listener, (Object) responses);
            return requests;
        });
        Thread peerThread = new Thread(peer, "kmipkit-java-kmip30-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration base = ClientConfiguration.create(registry);
                ClientConfiguration configuration = base.withProtocolVersion(new ProtocolVersion(3, 0));
                TransportConfiguration transport = transport(listener.getLocalPort());
                KmipClient client = KmipClient.create(configuration, transport);
                TtlvStructure source = TtlvStructure.create();
                AttributeSet attributes = AttributeSet.tryNew(source)) {
            CodecLimits limits = CodecLimits.defaults();
            UniqueIdentifier identifier = UniqueIdentifier.textString("kmip30-target");
            SecretText password = SecretText.fromString("credential-secret");
            try (SecretText ownedPassword = password;
                    CreateCredentialObject credential = CreateCredentialObject.password(
                            ownedPassword, null, null, null, null);
                    var builder = client.request().createCredential(attributes, credential)) {
                assertSuccessful(builder.execute(limits), 0x3f, true);
            }
            try (ClientBatchResponse response = client.request().createGroup(attributes)
                            .withOptions(RequestOptions.defaults()).execute(limits)) {
                assertSuccessful(response, CREATE_GROUP, true);
            }
            try (ClientBatchResponse response = client.request().createUser(attributes).execute(limits)) {
                assertSuccessful(response, 0x3e, true);
            }
            try (ClientBatchResponse response = client.request().deactivate(identifier).execute(limits)) {
                assertSuccessful(response, 0x40, true);
            }
            ClientOperationException decapsulateFailure = assertThrows(ClientOperationException.class,
                    () -> client.request().decapsulate(identifier, new byte[] {4, 5, 6}).execute(limits));
            assertEquals(3, decapsulateFailure.deliveryStateCode());
            assertFalse(decapsulateFailure.getMessage().contains(
                    new String(DECAPSULATE_SENTINEL, StandardCharsets.US_ASCII)));
            assertTrue(decapsulateFailure.getMessage().contains("redacted"));
            try (ClientBatchResponse response = client.request().encapsulate(identifier).execute(limits);
                    ClientBatchItemResult result = response.itemAt(0)) {
                assertEquals(0x41, result.operation());
                assertEquals("created-group-id", result.kmip30Identifier().orElseThrow().text());
                assertArrayEquals("public-encapsulated-data".getBytes(StandardCharsets.US_ASCII),
                        result.encapsulateData().orElseThrow());
            }
            try (ClientBatchResponse response = client.request().obliterate(identifier).execute(limits);
                    ClientBatchItemResult result = response.itemAt(0)) {
                assertEquals(0x3d, result.operation());
                assertTrue(result.kmip30Identifier().isEmpty());
            }
            try (ClientBatchResponse response = client.request().createGroup(attributes)
                            .asynchronousIndicator(1).execute(limits);
                    ClientBatchItemResult result = response.itemAt(0)) {
                assertEquals(CREATE_GROUP, result.operation());
                assertEquals(2, result.resultStatus());
                assertTrue(result.outcomeKind().isPending());
                assertArrayEquals(CORRELATION, result.pendingCorrelationValue().orElseThrow());
                assertTrue(result.resultReason().isEmpty());
            }
            try (ClientBatchResponse response = client.request().createGroup(attributes).execute(limits);
                    ClientBatchItemResult result = response.itemAt(0)) {
                assertEquals(CREATE_GROUP, result.operation());
                assertEquals(1, result.resultStatus());
                assertTrue(result.outcomeKind().isCompleted());
                assertEquals(1, result.resultReason().orElseThrow());
                assertEquals(FAILURE_MESSAGE, result.resultMessage().orElseThrow());
                assertFalse(result.toString().contains(FAILURE_MESSAGE));
            }
        } finally {
            listener.close();
        }
        List<byte[]> requests = peer.get(12, TimeUnit.SECONDS);
        assertEquals(9, requests.size(), "each execute call must use exactly one exchange");
        for (int index = 0; index < requests.size(); index++) {
            assertTrue(contains(requests.get(index), enumItem(0x42005c, operations[index])));
        }
        assertTrue(contains(requests.get(7), enumItem(0x420007, 1)),
                "the selected asynchronous indicator must reach the Request Header");
    }

    @Test
    void credentialBuilderOwnsItsNativeCopyAndNeverFormatsSecretInput() throws Exception {
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration configuration = ClientConfiguration.create(registry);
                TransportConfiguration transport = transport(1);
                KmipClient client = KmipClient.create(configuration, transport);
                TtlvStructure source = TtlvStructure.create();
                AttributeSet attributes = AttributeSet.tryNew(source);
                SecretText password = SecretText.fromString("credential-secret");
                SecretBytes salt = SecretBytes.fromBytes(new byte[] {9, 8, 7})) {
            CreateCredentialObject object = CreateCredentialObject.password(password, salt, null, null, null);
            assertFalse(object.toString().contains("credential-secret"));
            var builder = client.request().createCredential(attributes, object);
            object.close();
            password.close();
            salt.close();
            try (var item = builder.toBatchItem()) {
                assertEquals(0x3f, item.operation());
            }
            builder.close();
        }
    }

    private static TransportConfiguration transport(int port) throws Exception {
        byte[] clientCertificate = fixture("client-cert.pem");
        byte[] clientKey = fixture("client-key.pem");
        byte[] serverCertificate = fixture("server-cert.pem");
        return TransportConfiguration.builder()
                .rawTls("127.0.0.1", port)
                .clientIdentityPem(new byte[][] {clientCertificate}, clientKey)
                .trustCertificatesPem(new byte[][] {serverCertificate})
                .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(3)),
                        TransportConfiguration.Timeout.bounded(Duration.ofSeconds(3)),
                        TransportConfiguration.Timeout.bounded(Duration.ofSeconds(3)),
                        TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)))
                .build();
    }

    private static byte[] response(int operation, int status, Integer reason, String message,
            boolean includeIdentifier) {
        byte[] version = structure(0x420069, integer(0x42006a, 3), integer(0x42006b, 0));
        byte[] header = structure(0x42007a, version, longItem(0x420092, 1), integer(0x42000d, 1));
        ByteArrayOutputStream responseFields = new ByteArrayOutputStream();
        write(responseFields, enumItem(0x42005c, operation));
        write(responseFields, enumItem(0x42007f, status));
        if (status == 2) write(responseFields, byteItem(0x420006, CORRELATION));
        if (reason != null) write(responseFields, enumItem(0x42007e, reason));
        if (message != null) write(responseFields, textItem(0x42007d, message));
        if (status == 2 || includeIdentifier || operation == 0x41
                || operation == 0x42 || operation == 0x3d) {
            ByteArrayOutputStream payload = new ByteArrayOutputStream();
            if (includeIdentifier) write(payload, textItem(0x420094, "created-group-id"));
            if (operation == 0x41) write(payload, byteItem(0x4200c2,
                    "public-encapsulated-data".getBytes(StandardCharsets.US_ASCII)));
            if (operation == 0x42) write(payload, byteItem(0x4200c2, DECAPSULATE_SENTINEL));
            write(responseFields, structure(0x42007c, payload.toByteArray()));
        }
        return structure(0x42007b, header,
                structure(0x42000f, responseFields.toByteArray()));
    }

    private static byte[] structure(int tag, byte[]... children) {
        ByteArrayOutputStream body = new ByteArrayOutputStream();
        for (byte[] child : children) write(body, child);
        return item(tag, 1, body.toByteArray());
    }

    private static byte[] integer(int tag, int value) {
        return item(tag, 2, ByteBuffer.allocate(4).putInt(value).array());
    }

    private static byte[] enumItem(int tag, int value) {
        return item(tag, 5, ByteBuffer.allocate(4).putInt(value).array());
    }

    private static byte[] longItem(int tag, long value) {
        return item(tag, 9, ByteBuffer.allocate(8).putLong(value).array());
    }

    private static byte[] byteItem(int tag, byte[] value) { return item(tag, 8, value); }
    private static byte[] textItem(int tag, String value) {
        return item(tag, 7, value.getBytes(StandardCharsets.UTF_8));
    }

    private static byte[] item(int tag, int type, byte[] value) {
        ByteBuffer output = ByteBuffer.allocate(8 + value.length + (-value.length & 7));
        output.put((byte) (tag >>> 16)).put((byte) (tag >>> 8)).put((byte) tag).put((byte) type);
        output.putInt(value.length).put(value);
        return output.array();
    }

    private static void write(ByteArrayOutputStream output, byte[] value) { output.writeBytes(value); }
    private static boolean contains(byte[] source, byte[] sought) {
        outer: for (int offset = 0; offset <= source.length - sought.length; offset++) {
            for (int index = 0; index < sought.length; index++) {
                if (source[offset + index] != sought[index]) continue outer;
            }
            return true;
        }
        return false;
    }

    private static void assertSuccessful(ClientBatchResponse response, int operation,
            boolean hasIdentifier) {
        try (response; ClientBatchItemResult result = response.itemAt(0)) {
            assertEquals(1, response.itemCount());
            assertEquals(operation, result.operation());
            assertEquals(0, result.resultStatus());
            assertTrue(result.outcomeKind().isCompleted());
            assertEquals(hasIdentifier, result.kmip30Identifier().isPresent());
        }
    }

    private static byte[] fixture(String name) throws Exception {
        return (byte[]) fixtureCall("testFixture", new Class<?>[] {String.class}, name);
    }

    private static Object fixtureCall(String name, Class<?>[] parameterTypes, Object... arguments)
            throws Exception {
        Method method = ClientFacadeTlsIntegrationTest.class.getDeclaredMethod(name, parameterTypes);
        method.setAccessible(true);
        try {
            return method.invoke(null, arguments);
        } catch (InvocationTargetException error) {
            Throwable cause = error.getCause();
            if (cause instanceof Exception exception) throw exception;
            if (cause instanceof Error fatal) throw fatal;
            throw new IllegalStateException("TLS fixture call failed", cause);
        }
    }
}
