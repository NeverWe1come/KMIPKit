package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.net.Socket;
import java.time.Duration;
import java.util.List;
import java.util.concurrent.FutureTask;
import java.util.concurrent.TimeUnit;
import javax.net.ssl.SSLServerSocket;
import java.security.cert.X509Certificate;
import org.junit.jupiter.api.Test;
import org.kmipkit.client.KmipClient;
import org.kmipkit.fluent.LocateBuilder;
import org.kmipkit.fluent.RevokeBuilder;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.ClientConfiguration;
import org.kmipkit.extensions.ClientExtensionRegistry;
import org.kmipkit.extensions.ExtensionRegistryLimits;
import org.kmipkit.extensions.RevocationReason;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.TransportConfiguration;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.TtlvStructure;

/** Runs the task-oriented Java direct lifecycle methods through the TLS client. */
final class FluentLifecycleNativeIntegrationTest {
    @Test
    void lifecycleHelpersRoundtripThroughControlledMutualTls() throws Exception {
        byte[] serverCertificate = fixture("server-cert.pem");
        byte[] serverKey = fixture("server-key.pem");
        byte[] clientCertificate = fixture("client-cert.pem");
        byte[] clientKey = fixture("client-key.pem");
        X509Certificate parsedClientCertificate = (X509Certificate) fixtureCall(
                "testCertificate", new Class<?>[] {byte[].class}, (Object) clientCertificate);
        SSLServerSocket listener = (SSLServerSocket) fixtureCall(
                "deterministicMtlsListener",
                new Class<?>[] {byte[].class, byte[].class, X509Certificate.class},
                serverCertificate, serverKey, parsedClientCertificate);
        int[] operations = {0x12, 0x15, 0x08, 0x16, 0x13,
                0x12, 0x15, 0x08, 0x16, 0x13};
        byte[][] responses = new byte[operations.length][];
        for (int index = 0; index < operations.length; index++) {
            responses[index] = (byte[]) fixtureCall("fluentClientResponseMessage",
                    new Class<?>[] {int.class}, operations[index]);
        }
        FutureTask<List<byte[]>> peer = new FutureTask<>(() -> {
            @SuppressWarnings("unchecked")
            List<byte[]> requests = (List<byte[]>) fixtureCall(
                    "serveResponses", new Class<?>[] {SSLServerSocket.class, byte[][].class},
                    listener, (Object) responses);
            return requests;
        });
        Thread peerThread = new Thread(peer, "kmipkit-java-fluent-lifecycle-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration configuration = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityPem(new byte[][] {clientCertificate}, clientKey)
                        .trustCertificatesPem(new byte[][] {serverCertificate})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                KmipClient client = KmipClient.create(configuration, transport);
                TtlvStructure attributesTtlv = TtlvStructure.create();
                AttributeSet attributes = AttributeSet.tryNew(attributesTtlv);
                RevocationReason reason = RevocationReason.create(2, "scheduled rotation")) {
            CodecLimits limits = CodecLimits.defaults();
            UniqueIdentifier identifier = UniqueIdentifier.textString("lifecycle-target");
            assertThrows(InvalidInputException.class, () -> client.request().revoke(null, reason));
            assertThrows(InvalidInputException.class, () -> client.request().locate(null));
            Class<?>[] byId = {UniqueIdentifier.class, CodecLimits.class};
            int index = 0;
            for (String method : List.of("activateById", "archiveById")) {
                assertResult(invoke(client, method, byId, identifier, limits), operations[index++]);
            }
            assertResult(invoke(client, "locate",
                    new Class<?>[] {AttributeSet.class, CodecLimits.class}, attributes, limits),
                    operations[index++]);
            assertResult(invoke(client, "recoverById", byId, identifier, limits), operations[index++]);
            assertResult(invoke(client, "revokeById",
                    new Class<?>[] {UniqueIdentifier.class, RevocationReason.class,
                            Long.class, CodecLimits.class},
                    identifier, reason, 1_800_000_000L, limits), operations[index]);
            index++;

            var activate = client.request().activate(identifier);
            activate.validate();
            assertResult(activate.execute(limits), operations[index++]);
            assertResult(client.request().archive(identifier).execute(limits), operations[index++]);
            LocateBuilder locate = client.request().locate(attributes)
                    .maximumItems(4).offsetItems(1).withOptions(RequestOptions.defaults());
            LocateBuilder coverageLocate = client.request().locate(attributes)
                    .storageStatusMask(1).objectGroupMember(1)
                    .withOptions(RequestOptions.defaults()).asynchronousIndicator(0);
            assertThrows(InvalidInputException.class, () -> locate.withOptions(null));
            assertThrows(InvalidInputException.class,
                    () -> coverageLocate.withOptions(null));
            assertThrows(InvalidInputException.class,
                    () -> coverageLocate.asynchronousIndicator(-1));
            assertThrows(InvalidInputException.class,
                    () -> coverageLocate.asynchronousIndicator(0x1_0000_0000L));
            assertThrows(InvalidInputException.class, () -> locate.execute(null));
            assertThrows(InvalidInputException.class, () -> coverageLocate.execute(null));
            try (ClientBatchItem ignored = coverageLocate.toBatchItem()) { }
            coverageLocate.validate();
            assertResult(locate.execute(limits), operations[index++]);
            assertResult(client.request().recover(identifier).execute(limits), operations[index++]);
            try (RevocationReason builderReason = RevocationReason.create(1, "builder audit secret")) {
                RevokeBuilder revoke = client.request().revoke(identifier, builderReason);
                RevokeBuilder coverageRevoke = client.request().revoke(identifier, builderReason);
                coverageRevoke.withOptions(RequestOptions.defaults()).asynchronousIndicator(0);
                assertThrows(InvalidInputException.class, () -> revoke.withOptions(null));
                assertThrows(InvalidInputException.class,
                        () -> coverageRevoke.withOptions(null));
                assertThrows(InvalidInputException.class,
                        () -> coverageRevoke.asynchronousIndicator(-1));
                assertThrows(InvalidInputException.class,
                        () -> coverageRevoke.asynchronousIndicator(0x1_0000_0000L));
                assertThrows(InvalidInputException.class, () -> revoke.execute(null));
                assertThrows(InvalidInputException.class, () -> coverageRevoke.execute(null));
                try (ClientBatchItem ignored = coverageRevoke.toBatchItem()) { }
                coverageRevoke.validate();
                assertTrue(!revoke.toString().contains("builder audit secret"));
                builderReason.close();
                assertResult(revoke.execute(limits), operations[index]);
            }
        } finally {
            listener.close();
        }

        List<byte[]> requests = peer.get(12, TimeUnit.SECONDS);
        assertEquals(operations.length, requests.size());
        for (int index = 0; index < operations.length; index++) {
            assertTrue(contains(requests.get(index), enumeration(0x42005c, operations[index])),
                    "fixture must observe operation " + operations[index]);
        }
    }

    private static ClientBatchResponse invoke(
            KmipClient client, String name, Class<?>[] parameterTypes, Object... arguments)
            throws Exception {
        Method method = KmipClient.class.getMethod(name, parameterTypes);
        try {
            return (ClientBatchResponse) method.invoke(client, arguments);
        } catch (InvocationTargetException exception) {
            Throwable cause = exception.getCause();
            if (cause instanceof Exception checked) throw checked;
            if (cause instanceof Error error) throw error;
            throw new IllegalStateException("lifecycle helper invocation failed", cause);
        }
    }

    private static void assertResult(ClientBatchResponse response, int operation) {
        try (response; ClientBatchItemResult result = response.itemAt(0)) {
            assertEquals(1, response.itemCount());
            assertEquals(operation, result.operation());
            assertEquals(0, result.resultStatus());
        }
    }

    private static byte[] fixture(String name) throws Exception {
        return (byte[]) fixtureCall("testFixture", new Class<?>[] {String.class}, name);
    }

    private static Object fixtureCall(String name, Class<?>[] parameterTypes, Object... arguments)
            throws Exception {
        Method method = ClientFacadeTlsIntegrationTest.class.getDeclaredMethod(name, parameterTypes);
        if (!method.trySetAccessible()) throw new IllegalStateException("TLS fixture is inaccessible");
        try {
            return method.invoke(null, arguments);
        } catch (InvocationTargetException exception) {
            Throwable cause = exception.getCause();
            if (cause instanceof Exception checked) throw checked;
            if (cause instanceof Error error) throw error;
            throw new IllegalStateException("TLS fixture failed", cause);
        }
    }

    private static byte[] enumeration(int tag, int value) {
        byte[] item = new byte[16];
        item[0] = (byte) (tag >>> 16);
        item[1] = (byte) (tag >>> 8);
        item[2] = (byte) tag;
        item[3] = 5;
        item[7] = 4;
        item[8] = (byte) (value >>> 24);
        item[9] = (byte) (value >>> 16);
        item[10] = (byte) (value >>> 8);
        item[11] = (byte) value;
        return item;
    }

    private static boolean contains(byte[] input, byte[] pattern) {
        outer: for (int offset = 0; offset <= input.length - pattern.length; offset++) {
            for (int index = 0; index < pattern.length; index++) {
                if (input[offset + index] != pattern[index]) continue outer;
            }
            return true;
        }
        return false;
    }
}
