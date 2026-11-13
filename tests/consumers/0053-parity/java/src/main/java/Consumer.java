import java.nio.file.Path;
import java.util.Arrays;
import java.util.List;
import org.kmipkit.client.KmipClient;
import org.kmipkit.client.SymmetricKeyRequest;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.ClientConfiguration;
import org.kmipkit.extensions.ClientExtensionRegistry;
import org.kmipkit.extensions.ExtensionRegistryLimits;
import org.kmipkit.extensions.TransportConfiguration;
import org.kmipkit.credentials.Authentication;
import org.kmipkit.credentials.Credential;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.message.ResponseMessage;
import org.kmipkit.protocol.CryptographicParameters;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.KeyMaterial;
import org.kmipkit.ttlv.ManagedObject;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.StandardAttribute;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TypedAttribute;

public final class Consumer {
    private Consumer() {}

    private static TtlvItem item(int rawTag, TtlvValue value, CodecLimits limits) {
        try (RawTag raw = RawTag.fromRaw(rawTag); Tag tag = raw.checked()) {
            return TtlvItem.create(tag, value, limits);
        }
    }

    private static TtlvStructure append(
            TtlvStructure structure, int rawTag, TtlvValue value, CodecLimits limits) {
        return TtlvStructure.withItem(structure, item(rawTag, value, limits), limits);
    }

    private static TtlvStructure syntheticPingResponse(CodecLimits limits) {
        TtlvStructure version = TtlvStructure.create();
        version = append(version, 0x42006a, TtlvValue.Integer(2), limits);
        version = append(version, 0x42006b, TtlvValue.Integer(1), limits);
        TtlvStructure header = TtlvStructure.create();
        header = append(header, 0x420069, TtlvValue.structure(version, limits), limits);
        header = append(header, 0x420092, TtlvValue.DateTime(1_751_100_123L), limits);
        header = append(header, 0x42000d, TtlvValue.Integer(1), limits);
        TtlvStructure responseItem = TtlvStructure.create();
        responseItem = append(responseItem, 0x42005c, TtlvValue.Enumeration(0x3b), limits);
        responseItem = append(responseItem, 0x42007f, TtlvValue.Enumeration(0), limits);
        responseItem = append(
                responseItem, 0x42007c,
                TtlvValue.structure(TtlvStructure.create(), limits), limits);
        TtlvStructure root = TtlvStructure.create();
        root = append(root, 0x42007a, TtlvValue.structure(header, limits), limits);
        return append(root, 0x42000f, TtlvValue.structure(responseItem, limits), limits);
    }

    private static Authentication demoAuthentication(CodecLimits limits) {
        TtlvStructure credentialValue = TtlvStructure.create();
        credentialValue = append(
                credentialValue, 0x4200bb,
                TtlvValue.TextString("demo-user".getBytes(), limits), limits);
        TtlvStructure credential = TtlvStructure.create();
        credential = append(credential, 0x420024, TtlvValue.Enumeration(1), limits);
        credential = append(
                credential, 0x420025,
                TtlvValue.structure(credentialValue, limits), limits);
        return Authentication.create(List.of(Credential.fromTtlv(credential)));
    }

    public static void main(String[] args) {
        CodecLimits limits = CodecLimits.defaults();
        try (TtlvValue commentValue = TtlvValue.TextString("local example".getBytes(), limits);
                TypedAttribute attribute = TypedAttribute.create(
                        StandardAttribute.CONTACT_INFORMATION, commentValue);
                CryptographicParameters.Builder parameterBuilder = CryptographicParameters.builder();
                CryptographicParameters parameters = parameterBuilder
                        .cryptographicAlgorithm(0x8000_0001L).build()) {
            if (attribute.attribute() != StandardAttribute.CONTACT_INFORMATION
                    || parameters.cryptographicAlgorithm().orElseThrow() != 0x8000_0001L) {
                throw new IllegalStateException("typed package example failed");
            }

            try (TtlvItem dataType = item(0x420059, TtlvValue.Enumeration(1), limits);
                    TtlvItem dataValue = item(0x42005a, TtlvValue.ByteString("demo bytes".getBytes(), limits), limits);
                    ManagedObject managedObject = ManagedObject.opaqueObject(List.of(dataType, dataValue))) {
                if (managedObject.objectType() != 8) throw new IllegalStateException("object type mismatch");
            }

            try (TtlvItem material = item(0x420043, TtlvValue.ByteString("demo key bytes".getBytes(), limits), limits);
                    KeyMaterial keyMaterial = KeyMaterial.fromItem(1, material)) {
                if (keyMaterial.formatRaw() != 1) throw new IllegalStateException("key format mismatch");
            }

            ClientBatchItem ping = ClientBatchItem.ping();
            try (var report = ping.validationReport()) {
                if (!report.isValid()) throw new IllegalStateException("Ping did not validate");
            }
            try (ClientBatch batch = ClientBatch.create()) {
                batch.add(ping).setAuthentication(demoAuthentication(limits));
            }

            try (ResponseMessage response = ResponseMessage.fromTtlv(syntheticPingResponse(limits));
                    var item = response.batchItemAt(0)) {
                if (response.batchItemCount() != 1 || item.resultStatus().orElseThrow() != 0) {
                    throw new IllegalStateException("synthetic response did not parse");
                }
                try (TtlvStructure responseTtlv = item.ttlv(); var view = responseTtlv.view()) {
                    if (view.itemCount() != 3) throw new IllegalStateException("response TTLV mismatch");
                }
            }

            int port = Integer.parseInt(System.getenv("KMIPKIT_CONSUMER_TEST_PORT"));
            Path fixtures = Path.of(System.getenv("KMIPKIT_CONSUMER_TEST_FIXTURES"));
            try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                            List.of(), ExtensionRegistryLimits.defaults());
                    ClientConfiguration configuration = ClientConfiguration.create(registry);
                    TransportConfiguration transport = TransportConfiguration.builder()
                            .rawTls("127.0.0.1", port)
                            .clientIdentityFiles(List.of(fixtures.resolve("client-cert.pem")),
                                    fixtures.resolve("client-key.pem"),
                                    TransportConfiguration.Encoding.PEM)
                            .trustCertificateFiles(List.of(fixtures.resolve("server-cert.pem")),
                                    TransportConfiguration.Encoding.PEM)
                            .tlsServerName("localhost")
                            .build();
                    KmipClient kmipClient = KmipClient.create(configuration, transport)) {
                UniqueIdentifier identifier;
                try (ClientBatchResponse created = kmipClient.createSymmetricKey(
                                SymmetricKeyRequest.named(3, 256, 12, "fluent-key"), limits,
                                RequestOptions.defaults());
                        ClientBatchItemResult itemResult = created.itemAt(0)) {
                    if (!itemResult.outcomeKind().isCompleted() || itemResult.resultStatus() != 0) {
                        throw new IllegalStateException("Create did not complete successfully");
                    }
                    identifier = itemResult.createResponse().orElseThrow().uniqueIdentifier();
                }
                try (ClientBatchResponse retrieved = kmipClient.request().get(identifier)
                                .asynchronousIndicator(1).withOptions(RequestOptions.defaults()).execute(limits);
                        ClientBatchItemResult itemResult = retrieved.itemAt(0);
                        var response = itemResult.getResponse().orElseThrow()) {
                    if (!itemResult.outcomeKind().isCompleted() || itemResult.resultStatus() != 0
                            || !response.uniqueIdentifier().orElseThrow().text().equals(identifier.text())
                            || response.object().isEmpty()) {
                        throw new IllegalStateException("Get did not return the created object");
                    }
                }
                try (ClientBatchResponse destroyed = kmipClient.destroyById(
                                identifier, limits, RequestOptions.defaults());
                        ClientBatchItemResult itemResult = destroyed.itemAt(0)) {
                    if (!itemResult.outcomeKind().isCompleted() || itemResult.resultStatus() != 0
                            || !itemResult.destroyResponse().orElseThrow().uniqueIdentifier()
                                    .text().equals(identifier.text())) {
                        throw new IllegalStateException("Destroy did not return the created identifier");
                    }
                }
                try (ClientBatchResponse pending = kmipClient.request().destroy(identifier)
                                .asynchronousIndicator(1).execute(limits);
                        ClientBatchItemResult itemResult = pending.itemAt(0)) {
                    byte[] correlation = itemResult.pendingCorrelationValue().orElseThrow();
                    try {
                        if (!itemResult.outcomeKind().isPending() || itemResult.resultStatus() != 2
                                || !Arrays.equals(correlation, "destroy-pending-id".getBytes())) {
                            throw new IllegalStateException("Pending outcome or correlation was not preserved");
                        }
                    } finally {
                        Arrays.fill(correlation, (byte) 0);
                    }
                }
                try (ClientBatchResponse failed = kmipClient.request().destroy(identifier)
                                .asynchronousIndicator(1).execute(limits);
                        ClientBatchItemResult itemResult = failed.itemAt(0)) {
                    if (!itemResult.outcomeKind().isCompleted() || itemResult.resultStatus() != 1
                            || itemResult.destroyResponse().isPresent()
                            || !itemResult.resultMessage().orElseThrow().equals("fixture rejected destroy")) {
                        throw new IllegalStateException("Failure outcome was not preserved");
                    }
                }
                System.out.println("java: fluent lifecycle Success/Pending/Failure outcomes verified");
            }
        }
    }
}
