package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.time.Duration;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.concurrent.TimeUnit;
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
        assertThrows(InvalidInputException.class,
                () -> TransportConfiguration.Timeout.bounded(Duration.ofSeconds(-1)));
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
    void transportInputsValidateSourceAndEncodingAndRedactMaterial() {
        byte[] marker = "private-material-marker".getBytes(StandardCharsets.UTF_8);
        TransportConfiguration.TransportInput key = TransportConfiguration.TransportInput.memory(
                marker, TransportConfiguration.Encoding.PEM);
        Arrays.fill(marker, (byte) 0);

        assertEquals(TransportConfiguration.Encoding.PEM, key.encoding());
        assertFalse(key.toString().contains("private-material-marker"));
        assertThrows(InvalidInputException.class,
                () -> TransportConfiguration.TransportInput.memory(null, TransportConfiguration.Encoding.PEM));
        assertThrows(InvalidInputException.class,
                () -> TransportConfiguration.TransportInput.memory(new byte[0], TransportConfiguration.Encoding.PEM));
        assertThrows(InvalidInputException.class,
                () -> TransportConfiguration.TransportInput.memory(new byte[] {1}, null));
        assertThrows(ResourceLimitException.class,
                () -> TransportConfiguration.TransportInput.memory(
                        new byte[MAX_TRANSPORT_INPUT_BYTES + 1], TransportConfiguration.Encoding.PEM));
        assertThrows(InvalidInputException.class,
                () -> TransportConfiguration.TransportInput.file(null, TransportConfiguration.Encoding.DER));
        assertThrows(InvalidInputException.class,
                () -> TransportConfiguration.TransportInput.file(Path.of("key.der"), null));
    }

    @Test
    void clientIdentityConsumesItsPrivateKeyInputAndRevocationListsRemainMemoryOnly() {
        TransportConfiguration.TransportInput certificate = TransportConfiguration.TransportInput.memory(
                new byte[] {1}, TransportConfiguration.Encoding.PEM);
        TransportConfiguration.TransportInput privateKey = TransportConfiguration.TransportInput.memory(
                new byte[] {2}, TransportConfiguration.Encoding.DER);
        TransportConfiguration.Builder builder = TransportConfiguration.builder()
                .clientIdentity(List.of(certificate), privateKey);

        assertThrows(InvalidInputException.class,
                () -> builder.clientIdentity(List.of(certificate), privateKey));

        TransportConfiguration.TransportInput crlFile = TransportConfiguration.TransportInput.file(
                Path.of("revocation-list.pem"), TransportConfiguration.Encoding.PEM);
        assertThrows(InvalidInputException.class, () -> builder.revocationLists(List.of(crlFile)));
    }

    @Test
    void typedClientIdentityRejectsIncompleteAndOversizedCertificateGroups() {
        TransportConfiguration.TransportInput certificate = TransportConfiguration.TransportInput.memory(
                new byte[] {1}, TransportConfiguration.Encoding.PEM);
        TransportConfiguration.TransportInput privateKey = TransportConfiguration.TransportInput.memory(
                new byte[] {2}, TransportConfiguration.Encoding.PEM);
        TransportConfiguration.Builder builder = TransportConfiguration.builder();

        assertThrows(InvalidInputException.class, () -> builder.clientIdentity(null, privateKey));
        assertThrows(InvalidInputException.class, () -> builder.clientIdentity(List.of(), privateKey));
        assertThrows(InvalidInputException.class,
                () -> builder.clientIdentity(List.of(certificate), null));
        assertThrows(InvalidInputException.class,
                () -> builder.clientIdentity(Arrays.asList(certificate, null), privateKey));
        assertThrows(ResourceLimitException.class,
                () -> builder.clientIdentity(Collections.nCopies(4_097, certificate), privateKey));
    }

    @Test
    void typedClientIdentityCountsCertificateBytesBeforeSnapshotting() {
        TransportConfiguration.TransportInput maximumCertificate = TransportConfiguration.TransportInput.memory(
                new byte[MAX_TRANSPORT_INPUT_BYTES], TransportConfiguration.Encoding.PEM);
        TransportConfiguration.TransportInput extraCertificate = TransportConfiguration.TransportInput.memory(
                new byte[] {1}, TransportConfiguration.Encoding.PEM);
        TransportConfiguration.TransportInput privateKey = TransportConfiguration.TransportInput.memory(
                new byte[] {2}, TransportConfiguration.Encoding.PEM);

        assertThrows(ResourceLimitException.class,
                () -> TransportConfiguration.builder().clientIdentity(
                        List.of(maximumCertificate, extraCertificate), privateKey));
    }

    @Test
    void closingUnbuiltBuilderZeroizesItsOwnedPrivateKeyCopy() throws Exception {
        byte[] marker = {0x31, 0x42, 0x53};
        TransportConfiguration.TransportInput privateKey = TransportConfiguration.TransportInput.memory(
                marker, TransportConfiguration.Encoding.DER);
        TransportConfiguration.Builder builder = TransportConfiguration.builder()
                .clientIdentity(
                        List.of(TransportConfiguration.TransportInput.memory(
                                new byte[] {1}, TransportConfiguration.Encoding.PEM)),
                        privateKey);
        byte[] ownedCopy = builderPrivateKeyBytes(builder);

        assertArrayEquals(marker, ownedCopy);
        builder.close();
        builder.close();
        assertArrayEquals(new byte[marker.length], ownedCopy);
        assertArrayEquals(new byte[] {0x31, 0x42, 0x53}, marker);
        assertThrows(InvalidInputException.class, builder::build);
    }

    @Test
    void closingUnbuiltBuilderZeroizesKeyCopiedFromLegacyPemSetter() throws Exception {
        byte[] marker = {0x41, 0x52, 0x63};
        TransportConfiguration.Builder builder = TransportConfiguration.builder()
                .clientIdentityPem(new byte[][] {{1}}, marker);
        byte[] ownedCopy = builderPrivateKeyBytes(builder);

        assertArrayEquals(marker, ownedCopy);
        builder.close();
        assertArrayEquals(new byte[marker.length], ownedCopy);
        assertArrayEquals(new byte[] {0x41, 0x52, 0x63}, marker);
        assertThrows(InvalidInputException.class, () -> builder.clientIdentityPem(
                new byte[][] {{1}}, new byte[] {9}));
    }

    private static byte[] builderPrivateKeyBytes(TransportConfiguration.Builder builder)
            throws ReflectiveOperationException {
        java.lang.reflect.Field privateKeyField = builder.getClass().getDeclaredField("privateKey");
        privateKeyField.setAccessible(true);
        Object ownedInput = privateKeyField.get(builder);
        java.lang.reflect.Field dataField = ownedInput.getClass().getDeclaredField("data");
        dataField.setAccessible(true);
        return (byte[]) dataField.get(ownedInput);
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
    void acceptsAndReplacesFileBasedIdentityAndTrustInputs() {
        TransportConfiguration.Builder builder = TransportConfiguration.builder();

        assertDoesNotThrow(() -> builder.clientIdentityFiles(
                List.of(Path.of("client-cert.pem")), Path.of("client-key.pem"),
                TransportConfiguration.Encoding.PEM));
        assertDoesNotThrow(() -> builder.trustCertificateFiles(
                List.of(Path.of("trust.pem")), TransportConfiguration.Encoding.PEM));
        assertDoesNotThrow(() -> builder.clientIdentityFiles(
                List.of(Path.of("replacement-cert.der")), Path.of("replacement-key.der"),
                TransportConfiguration.Encoding.DER));
        assertDoesNotThrow(() -> builder.trustCertificateFiles(
                List.of(Path.of("replacement-trust.der")), TransportConfiguration.Encoding.DER));
    }

    @Test
    void rejectsBlankAndOversizedFilePathsBeforeEncoding() {
        TransportConfiguration.Builder builder = TransportConfiguration.builder();

        assertThrows(InvalidInputException.class,
                () -> builder.clientIdentityFiles(
                        List.of(Path.of("client-cert.pem")), Path.of(""),
                        TransportConfiguration.Encoding.PEM));
        assertThrows(ResourceLimitException.class,
                () -> builder.trustCertificateFiles(
                        List.of(Path.of("a".repeat(4_097))), TransportConfiguration.Encoding.PEM));
    }

    @Test
    void enforcesTextLimitsByUtf8BytesForTlsNamesUrisAndEndpoints() {
        TransportConfiguration.Builder builder = TransportConfiguration.builder();

        assertDoesNotThrow(() -> builder.tlsServerName("é".repeat(2_048)));
        assertThrows(ResourceLimitException.class,
                () -> builder.tlsServerName("é".repeat(2_049)));
        assertDoesNotThrow(() -> builder.targetUri("三".repeat(1_365) + "a"));
        assertThrows(ResourceLimitException.class,
                () -> builder.targetUri("三".repeat(1_365) + "ab"));
        assertDoesNotThrow(() -> builder.tlsServerName("🔐".repeat(1_024)));
        assertThrows(ResourceLimitException.class,
                () -> builder.tlsServerName("🔐".repeat(1_025)));

        TransportConfiguration.Builder oversizedEndpoint = TransportConfiguration.builder()
                .rawTls("é".repeat(2_049), 5696)
                .clientIdentityPem(new byte[][] {{1}}, new byte[] {2})
                .trustPlatform();
        assertThrows(ResourceLimitException.class, oversizedEndpoint::build);
    }

    @Test
    void preservesUtf8ReplacementLengthForUnpairedSurrogates() {
        TransportConfiguration.Builder builder = TransportConfiguration.builder();
        String unpairedSurrogate = String.valueOf((char) 0xd800);

        assertDoesNotThrow(() -> builder.tlsServerName("a".repeat(4_095) + unpairedSurrogate));
        assertThrows(ResourceLimitException.class,
                () -> builder.tlsServerName("a".repeat(4_096) + unpairedSurrogate));
    }

    @Test
    void rejectsVeryLargeTextBeforeAllocatingAnEncodedCopy() throws Exception {
        String javaExecutable = Path.of(System.getProperty("java.home"), "bin", "java").toString();
        String classpath = System.getProperty("surefire.test.class.path", System.getProperty("java.class.path"));
        Process process = new ProcessBuilder(javaExecutable, "-Xmx48m", "-cp", classpath,
                TransportConfigurationTextLimitProbe.class.getName(), "24000000")
                .redirectErrorStream(true)
                .start();

        if (!process.waitFor(30, TimeUnit.SECONDS)) {
            process.destroyForcibly();
            throw new AssertionError("bounded-heap text-limit probe did not finish");
        }

        String output = new String(process.getInputStream().readAllBytes(), StandardCharsets.UTF_8).trim();
        assertEquals(0, process.exitValue(), output);
        assertEquals("resource-limit", output);
    }

    @Test
    void countsUtf8FilePathBytesAcrossClientIdentityBeforeEncoding() {
        Path maximumLengthPath = Path.of("🔐/".repeat(500) + "é/".repeat(500)
                + "三/".repeat(23) + "三a");
        TransportConfiguration.Builder builder = TransportConfiguration.builder();

        assertEquals(4_096,
                maximumLengthPath.toString().getBytes(java.nio.charset.StandardCharsets.UTF_8).length);
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
