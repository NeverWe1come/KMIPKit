package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.nio.file.Path;
import java.time.Duration;
import java.util.Collections;
import java.util.List;
import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ResourceLimitException;

final class TransportConfigurationBoundaryTest {
    private static final int MAX_TRANSPORT_INPUT_BYTES = 16 * 1024 * 1024;

    @Test
    void timeoutValuesRejectInvalidRangesAndKeepExactNanoseconds() {
        assertEquals(new TransportConfiguration.Timeout(0, 0, 0),
                TransportConfiguration.Timeout.defaultValue());
        assertEquals(new TransportConfiguration.Timeout(1, 3, 5),
                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(3).plusNanos(5)));
        assertEquals(new TransportConfiguration.Timeout(2, 0, 0),
                TransportConfiguration.Timeout.unbounded());

        assertInvalidTimeout(-1, 0, 0);
        assertInvalidTimeout(3, 0, 0);
        assertInvalidTimeout(1, -1, 0);
        assertInvalidTimeout(1, 0, -1);
        assertInvalidTimeout(1, 0, 1_000_000_000);
        assertInvalidTimeout(2, 1, 0);
        assertInvalidTimeout(2, 0, 1);
    }

    @Test
    void builderRejectsInvalidEndpointsAndIncompleteMutualTlsConfiguration() {
        TransportConfiguration.Builder builder = TransportConfiguration.builder();
        assertThrows(InvalidInputException.class, () -> builder.rawTls(null, 5696));
        assertThrows(InvalidInputException.class, () -> builder.rawTls("kms.example.test", 0));
        assertThrows(InvalidInputException.class, () -> builder.rawTls("kms.example.test", 65_536));
        assertThrows(InvalidInputException.class, () -> builder.https(" "));
        assertThrows(InvalidInputException.class, () -> builder.tlsServerName(" "));
        assertThrows(InvalidInputException.class, () -> builder.targetUri(null));
        assertThrows(InvalidInputException.class,
                () -> builder.timeouts(null, TransportConfiguration.Timeout.defaultValue(),
                        TransportConfiguration.Timeout.defaultValue(), TransportConfiguration.Timeout.defaultValue()));
        assertThrows(InvalidInputException.class, () -> builder.maxRequestBytes(0));

        TransportConfiguration.Builder missingEndpoint = TransportConfiguration.builder();
        assertThrows(InvalidInputException.class, missingEndpoint::build);
        assertThrows(InvalidInputException.class, () -> missingEndpoint.maxRequestBytes(1));

        TransportConfiguration.Builder missingIdentity = TransportConfiguration.builder()
                .rawTls("kms.example.test", 5696);
        assertThrows(InvalidInputException.class, missingIdentity::build);

        TransportConfiguration.Builder missingTrust = TransportConfiguration.builder()
                .rawTls("kms.example.test", 5696)
                .clientIdentityPem(new byte[][] {{1}}, new byte[] {2});
        assertThrows(InvalidInputException.class, missingTrust::build);

        TransportConfiguration.Builder wrongRawTlsTarget = TransportConfiguration.builder()
                .rawTls("kms.example.test", 5696)
                .clientIdentityPem(new byte[][] {{1}}, new byte[] {2})
                .trustPlatform()
                .targetUri("/kmip");
        assertThrows(InvalidInputException.class, wrongRawTlsTarget::build);

        TransportConfiguration.Builder wrongHttpsServerName = TransportConfiguration.builder()
                .https("https://kms.example.test/kmip")
                .clientIdentityDer(new byte[][] {{1}}, new byte[] {2})
                .trustPlatform()
                .tlsServerName("kms.example.test");
        assertThrows(InvalidInputException.class, wrongHttpsServerName::build);
    }

    @Test
    void builderRejectsInvalidMaterialCountsElementsAndPathsBeforeNativeCalls() {
        TransportConfiguration.Builder builder = TransportConfiguration.builder();
        assertThrows(InvalidInputException.class, () -> builder.clientIdentityPem(null, new byte[] {1}));
        assertThrows(InvalidInputException.class, () -> builder.clientIdentityPem(new byte[0][], new byte[] {1}));
        assertThrows(InvalidInputException.class,
                () -> builder.clientIdentityPem(new byte[][] {null}, new byte[] {1}));
        assertThrows(InvalidInputException.class,
                () -> builder.clientIdentityPem(new byte[][] {new byte[0]}, new byte[] {1}));
        assertThrows(InvalidInputException.class,
                () -> builder.clientIdentityPem(new byte[][] {{1}}, null));
        assertThrows(ResourceLimitException.class,
                () -> builder.clientIdentityPem(new byte[4_097][], new byte[] {1}));

        assertThrows(InvalidInputException.class,
                () -> builder.clientIdentityFiles(null, Path.of("key.pem"), TransportConfiguration.Encoding.PEM));
        assertThrows(InvalidInputException.class,
                () -> builder.clientIdentityFiles(List.of(), Path.of("key.pem"), TransportConfiguration.Encoding.PEM));
        assertThrows(InvalidInputException.class,
                () -> builder.clientIdentityFiles(List.of(Path.of("cert.pem")), null,
                        TransportConfiguration.Encoding.PEM));
        assertThrows(InvalidInputException.class,
                () -> builder.clientIdentityFiles(Collections.nCopies(4_097, Path.of("cert.pem")),
                        Path.of("key.pem"), TransportConfiguration.Encoding.PEM));

        assertThrows(InvalidInputException.class, () -> builder.trustCertificatesPem(null));
        assertThrows(InvalidInputException.class, () -> builder.trustCertificatesDer(new byte[0][]));
        assertThrows(InvalidInputException.class, () -> builder.trustCertificatesPem(new byte[][] {null}));
        assertThrows(InvalidInputException.class, () -> builder.trustCertificateFiles(null,
                TransportConfiguration.Encoding.DER));
        assertThrows(InvalidInputException.class, () -> builder.trustCertificateFiles(List.of(),
                TransportConfiguration.Encoding.DER));
        assertThrows(InvalidInputException.class,
                () -> builder.revocationListsPem(new byte[][] {new byte[0]}));
        assertThrows(InvalidInputException.class, () -> builder.revocationListsDer(null));
    }

    @Test
    void rejectsSingleOversizedMemoryInputAtSetterBoundary() {
        TransportConfiguration.Builder builder = TransportConfiguration.builder();

        assertThrows(ResourceLimitException.class,
                () -> builder.clientIdentityPem(
                        new byte[][] {new byte[MAX_TRANSPORT_INPUT_BYTES + 1]}, new byte[] {1}));
    }

    @Test
    void enforcesAggregateInputLimitAcrossIdentityTrustAndRevocationGroups() {
        TransportConfiguration.Builder builder = TransportConfiguration.builder()
                .clientIdentityPem(new byte[][] {new byte[4 * 1024 * 1024]}, new byte[1])
                .trustCertificatesPem(new byte[][] {new byte[5 * 1024 * 1024]});

        assertThrows(ResourceLimitException.class,
                () -> builder.revocationListsPem(new byte[][] {new byte[7 * 1024 * 1024]}));
    }

    @Test
    void replacesExistingMaterialGroupsWhenCheckingAggregateInputLimit() {
        TransportConfiguration.Builder builder = TransportConfiguration.builder()
                .clientIdentityPem(new byte[][] {new byte[12 * 1024 * 1024 - 1]}, new byte[] {1})
                .trustCertificatesPem(new byte[][] {new byte[2 * 1024 * 1024]})
                .revocationListsPem(new byte[][] {new byte[2 * 1024 * 1024]});

        assertDoesNotThrow(() -> builder.clientIdentityPem(new byte[][] {{1}}, new byte[] {2}));
        assertDoesNotThrow(() -> builder.trustCertificatesPem(new byte[][] {{3}}));
        assertDoesNotThrow(() -> builder.revocationListsPem(new byte[][] {{4}}));
    }

    @Test
    void countsUtf8FilePathBytesAcrossClientIdentityBeforeEncoding() {
        Path maximumLengthPath = Path.of("a/".repeat(2_047) + "ab");
        TransportConfiguration.Builder builder = TransportConfiguration.builder();

        assertEquals(4_096, maximumLengthPath.toString().length());
        assertThrows(ResourceLimitException.class,
                () -> builder.clientIdentityFiles(
                        Collections.nCopies(4_096, maximumLengthPath), Path.of("k"),
                        TransportConfiguration.Encoding.PEM));
    }

    private static void assertInvalidTimeout(int kind, long seconds, int nanoseconds) {
        assertThrows(InvalidInputException.class,
                () -> new TransportConfiguration.Timeout(kind, seconds, nanoseconds));
    }
}
