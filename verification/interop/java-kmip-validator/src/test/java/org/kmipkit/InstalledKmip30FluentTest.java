package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.net.InetAddress;
import java.net.ServerSocket;
import java.time.Duration;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import org.junit.jupiter.api.Test;
import org.kmipkit.client.KmipClient;
import org.kmipkit.credentials.SecretBytes;
import org.kmipkit.credentials.SecretText;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.extensions.ClientConfiguration;
import org.kmipkit.extensions.ClientExtensionRegistry;
import org.kmipkit.extensions.ExtensionRegistryLimits;
import org.kmipkit.extensions.TransportConfiguration;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.fluent.CreateCredentialObject;
import org.kmipkit.protocol.CryptographicParameters;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;


/** Exercises fluent builders from the independently installed Java package. */
final class InstalledKmip30FluentTest {
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
    void credentialAlternativesAndOptionalKmip30InputsReachTypedAbi() throws Exception {
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration configuration = ClientConfiguration.create(registry);
                TransportConfiguration transport = transport(1);
                KmipClient client = KmipClient.create(configuration, transport);
                TtlvStructure source = TtlvStructure.create();
                AttributeSet attributes = AttributeSet.tryNew(source);
                SecretBytes firstSecret = SecretBytes.fromBytes(new byte[] {1, 2, 3});
                SecretBytes secondSecret = SecretBytes.fromBytes(new byte[] {4, 5, 6});
                SecretBytes otpSeed = SecretBytes.fromBytes(new byte[] {7, 8, 9});
                SecretText passwordText = SecretText.fromString("credential-password");
                TtlvItem certificateItem = certificateItem();
                CreateCredentialObject password = CreateCredentialObject.password(
                        passwordText, null, null, null, null);
                CreateCredentialObject certificate = CreateCredentialObject.certificate(certificateItem);
                CreateCredentialObject device = CreateCredentialObject.device(
                        "serial", "device-id", "network-id", null, "media-id");
                CreateCredentialObject hashedPassword = CreateCredentialObject.hashedPassword(
                        firstSecret, secondSecret, 1L);
                CreateCredentialObject oneTimePassword = CreateCredentialObject.oneTimePassword(
                        1, 1L, "otp-serial", otpSeed, 30L, 6)) {
            for (CreateCredentialObject object : List.of(
                    password, certificate, device, hashedPassword, oneTimePassword)) {
                try (var builder = client.request().createCredential(attributes, object)) {
                    builder.validate();
                    try (var item = builder.toBatchItem()) {
                        assertEquals(0x3f, item.operation());
                    }
                }
            }

            UniqueIdentifier identifier = UniqueIdentifier.textString("kmip30-target");
            try (TtlvStructure reason = TtlvStructure.create();
                    CryptographicParameters parameters = CryptographicParameters.builder()
                            .cryptographicAlgorithm(1).build();
                    var deactivate = client.request().deactivate(identifier)
                            .deactivationReason(reason).deactivationDate(1_700_000_000L);
                    var encapsulate = client.request().encapsulate(identifier)
                            .attributes(attributes).cryptographicParameters(parameters)) {
                deactivate.validate();
                encapsulate.validate();
                try (var deactivateItem = deactivate.toBatchItem();
                        var encapsulateItem = encapsulate.toBatchItem()) {
                    assertEquals(0x40, deactivateItem.operation());
                    assertEquals(0x41, encapsulateItem.operation());
                }
            }
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
            assertFalse(failure.getMessage().contains("credential-secret"));
        }
    }

    private static TtlvItem certificateItem() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure contents = TtlvStructure.create();
        try {
            contents = TtlvStructure.withItem(contents,
                    ttlvItem(0x0042_001d, TtlvValue.Enumeration(1), limits), limits);
            contents = TtlvStructure.withItem(contents,
                    ttlvItem(0x0042_001e, TtlvValue.ByteString(
                            new byte[] {1, 2, 3}, limits), limits), limits);
            try (TtlvValue value = TtlvValue.structure(contents, limits)) {
                contents.close();
                return ttlvItem(0x0042_0013, value, limits);
            }
        } finally {
            contents.close();
        }
    }

    private static TtlvItem ttlvItem(int rawTag, TtlvValue value, CodecLimits limits) {
        try (RawTag raw = RawTag.fromRaw(rawTag); Tag tag = raw.checked(); value) {
            return TtlvItem.create(tag, value, limits);
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

    private static byte[] fixture(String name) throws Exception {
        return Files.readAllBytes(Path.of("../../../bindings/shared/fixtures/login-mtls", name));
    }
}
