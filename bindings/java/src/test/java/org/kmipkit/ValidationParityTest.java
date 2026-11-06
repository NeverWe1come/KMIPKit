package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.net.InetAddress;
import java.net.ServerSocket;
import java.net.Socket;
import java.net.SocketTimeoutException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.concurrent.FutureTask;
import java.util.concurrent.TimeUnit;
import org.junit.jupiter.api.Test;
import org.kmipkit.ClientOperationException;
import org.kmipkit.extensions.Client;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientConfiguration;
import org.kmipkit.extensions.ClientExtensionRegistry;
import org.kmipkit.extensions.ExtensionRegistryLimits;
import org.kmipkit.extensions.TransportConfiguration;
import org.kmipkit.extensions.ValidationIssue;
import org.kmipkit.extensions.ValidationReport;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;

final class ValidationParityTest {
    @Test
    void genericValidationDoesNotGuessAnOperationWithoutAnExplicitSchema() {
        try (TtlvStructure structure = TtlvStructure.create();
                ValidationReport report = ValidationReport.validateStructure(structure, null)) {
            assertTrue(report.isValid());
            assertEquals(0, report.issues().size());
        }
    }

    @Test
    void explicitCreateSchemaReportsOnlySafeStableMetadata() {
        try (TtlvStructure structure = TtlvStructure.create();
                ValidationReport report = ValidationReport.validateStructure(
                        structure, "RequestPayload/Create")) {
            assertFalse(report.isValid());
            assertEquals(2, report.issues().size());
            ValidationIssue issue = report.issues().get(0);
            assertEquals("KMIPKIT-0051-R019", issue.code());
            assertEquals("Request Payload.Object Type", issue.path());
            assertEquals("ERROR", issue.severity());
            assertEquals("KMIP v2.1", issue.sourceIdentifier());
            assertEquals("§6.1.8, Table 186", issue.sourceReference());
            assertFalse(issue.toString().contains("SECRET"));
        }
    }

    @Test
    void typedRequestUsesTheSharedRustValidationReport() {
        try (ClientBatchItem item = ClientBatchItem.ping();
                ValidationReport report = item.validationReport()) {
            assertTrue(report.isValid());
            assertEquals(0, report.issues().size());
        }
    }

    @Test
    void invalidTypedRequestReportsSafeMetadataAndNeverConnects() throws Exception {
        String canary = "VALIDATION_SECRET_CANARY";
        byte[] clientCertificate = testFixture("client-cert.pem");
        byte[] clientPrivateKey = testFixture("client-key.pem");
        byte[] serverCertificate = testFixture("server-cert.pem");
        CodecLimits limits = CodecLimits.defaults();
        try (ServerSocket listener = new ServerSocket(0, 1, InetAddress.getByName("127.0.0.1"))) {
            listener.setSoTimeout(1_000);
            FutureTask<Boolean> accepted = new FutureTask<>(() -> {
                try (Socket ignored = listener.accept()) {
                    return true;
                } catch (SocketTimeoutException timeout) {
                    return false;
                }
            });
            Thread observer = new Thread(accepted, "kmipkit-validation-no-connect-observer");
            observer.setDaemon(true);
            observer.start();

            try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                            List.of(), ExtensionRegistryLimits.defaults());
                    ClientConfiguration configuration = ClientConfiguration.create(registry);
                    TransportConfiguration transport = TransportConfiguration.builder()
                            .rawTls("127.0.0.1", listener.getLocalPort())
                            .clientIdentityPem(new byte[][] {clientCertificate}, clientPrivateKey)
                            .trustCertificatesPem(new byte[][] {serverCertificate})
                            .build();
                    Client client = Client.create(configuration, transport);
                    RawTag rawComment = RawTag.fromRaw(0x0042_00fd);
                    Tag comment = rawComment.checked();
                    TtlvStructure attributes = repeatedCommentAttributes(comment, limits, canary);
                    ClientBatchItem item = ClientBatchItem.create(7, attributes, null);
                    ValidationReport report = ValidationReport.validateRequest(item);
                    ClientBatch batch = ClientBatch.create().add(item)) {
                assertFalse(report.isValid());
                ValidationIssue issue = report.issues().get(0);
                assertEquals("KMIPKIT-0051-R190", issue.code());
                assertEquals("Request Payload.Attributes", issue.path());
                assertEquals("§4, Table 46", issue.sourceReference());
                assertFalse(issue.toString().contains(canary));
                ClientOperationException failure = assertThrows(
                        ClientOperationException.class, () -> client.execute(batch, limits));
                assertEquals(1, failure.categoryCode());
                assertEquals(1, failure.deliveryStateCode());
                assertFalse(failure.getMessage().contains(canary));
                assertFalse(failure.toString().contains(canary));
                assertFalse(accepted.get(2, TimeUnit.SECONDS),
                        "invalid local requests must make zero TCP connections");
            }
        }
    }

    private static TtlvStructure repeatedCommentAttributes(Tag comment, CodecLimits limits, String value) {
        TtlvStructure attributes = TtlvStructure.create();
        for (int index = 0; index < 2; index++) {
            TtlvValue text = TtlvValue.TextString(value.getBytes(java.nio.charset.StandardCharsets.UTF_8), limits);
            TtlvItem item = TtlvItem.create(comment, text, limits);
            attributes = TtlvStructure.withItem(attributes, item, limits);
        }
        return attributes;
    }

    private static byte[] testFixture(String filename) throws Exception {
        Path directory = Path.of("").toAbsolutePath();
        while (directory != null) {
            Path candidate = directory.resolve(Path.of(
                    "bindings", "python", "tests", "fixtures", "login-mtls", filename));
            if (Files.isRegularFile(candidate)) return Files.readAllBytes(candidate);
            directory = directory.getParent();
        }
        throw new IllegalStateException("the shared deterministic mTLS fixture is missing");
    }
}
