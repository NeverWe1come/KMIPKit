import java.nio.file.Path;
import java.util.List;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.ClientConfiguration;
import org.kmipkit.extensions.ClientExtensionRegistry;
import org.kmipkit.extensions.ExtensionRegistryLimits;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.TransportConfiguration;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.fluent.SymmetricKeyBuilder;
import org.kmipkit.protocol.KmipValues;
import org.kmipkit.ttlv.CodecLimits;

public final class Consumer {
    private Consumer() {}

    public static void main(String[] args) {
        int port = Integer.parseInt(System.getenv("KMIPKIT_CONSUMER_TEST_PORT"));
        Path fixtures = Path.of(System.getenv("KMIPKIT_CONSUMER_TEST_FIXTURES"));
        CodecLimits limits = CodecLimits.defaults();
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
            try (SymmetricKeyBuilder builder = kmipClient.request().createSymmetricKey();
                ClientBatchResponse created = builder.aes256().usageMask((int) (
                                KmipValues.CryptographicUsageMask.ENCRYPT
                                        | KmipValues.CryptographicUsageMask.DECRYPT))
                        .name("fluent-key")
                            .withOptions(RequestOptions.defaults()).execute(limits);
                    ClientBatchItemResult item = created.itemAt(0)) {
                if (!item.outcomeKind().isCompleted() || item.resultStatus() != 0) {
                    throw new IllegalStateException("Create did not complete successfully");
                }
                identifier = item.createResponse().orElseThrow().uniqueIdentifier();
            }

            UniqueIdentifier retrievedIdentifier;
            try (ClientBatchResponse retrieved = kmipClient.request().get(identifier)
                            .withOptions(RequestOptions.defaults()).execute(limits);
                    ClientBatchItemResult item = retrieved.itemAt(0);
                    var response = item.getResponse().orElseThrow()) {
                if (!item.outcomeKind().isCompleted() || item.resultStatus() != 0
                        || !response.uniqueIdentifier().orElseThrow().text().equals(identifier.text())
                        || response.object().isEmpty()) {
                    throw new IllegalStateException("Get did not return the created object");
                }
                retrievedIdentifier = response.uniqueIdentifier().orElseThrow();
            }

            try (ClientBatchResponse destroyed = kmipClient.request().destroy(retrievedIdentifier)
                            .withOptions(RequestOptions.defaults()).execute(limits);
                    ClientBatchItemResult item = destroyed.itemAt(0)) {
                if (!item.outcomeKind().isCompleted() || item.resultStatus() != 0
                        || !item.destroyResponse().orElseThrow().uniqueIdentifier()
                                .text().equals(retrievedIdentifier.text())) {
                    throw new IllegalStateException("Destroy did not return the retrieved identifier");
                }
            }
            System.out.println("java: fluent Create/Get/Destroy journey verified");
        }
    }
}
