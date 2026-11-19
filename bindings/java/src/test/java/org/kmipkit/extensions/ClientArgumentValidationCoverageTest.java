package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertThrows;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.OptionalLong;
import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ttlv.CodecLimits;

final class ClientArgumentValidationCoverageTest {
    @Test
    void rejectsInvalidBatchProfileAndAsynchronousInputsBeforeAnyExchange() throws Exception {
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration configuration = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", 5696)
                        .clientIdentityPem(new byte[][] {testFixture("client-cert.pem")},
                                testFixture("client-key.pem"))
                        .trustCertificatesPem(new byte[][] {testFixture("server-cert.pem")})
                        .build();
                Client client = Client.create(configuration, transport)) {
            CodecLimits limits = CodecLimits.defaults();
            RequestOptions options = RequestOptions.defaults();
            ProfileValidationSelection selection = new ProfileValidationSelection(List.of("baseline"));

            assertThrows(InvalidInputException.class, () -> client.execute(null, limits));
            assertThrows(InvalidInputException.class, () -> client.execute(null, null));
            assertThrows(InvalidInputException.class,
                    () -> client.executeWithOptions(null, limits, options));
            assertThrows(InvalidInputException.class,
                    () -> client.executeWithProfiles(null, limits, selection));
            assertThrows(InvalidInputException.class,
                    () -> client.executeWithProfilesAndOptions(null, limits, selection, options));
            assertThrows(InvalidInputException.class,
                    () -> client.executeWithProfilesAndContext(null, limits, selection, new ProfileValidationContext()));
            assertThrows(InvalidInputException.class,
                    () -> client.executeWithProfilesAndContextAndOptions(
                            null, limits, selection, new ProfileValidationContext(), options));

            assertThrows(InvalidInputException.class, () -> client.executePoll(null, limits));
            assertThrows(InvalidInputException.class,
                    () -> client.executePollWithOptions(new byte[0], limits, null));
            assertThrows(InvalidInputException.class, () -> client.executeCancel(new byte[0], null));
            assertThrows(InvalidInputException.class,
                    () -> client.executeCancelWithOptions(new byte[0], limits, null));
            assertThrows(InvalidInputException.class,
                    () -> client.executeProcess(new byte[0], limits, null));
            assertThrows(InvalidInputException.class,
                    () -> client.executeProcessWithOptions(new byte[0], limits, OptionalLong.empty(), null));
            assertThrows(InvalidInputException.class,
                    () -> client.executeProcess(new byte[0], limits, OptionalLong.of(0x1_0000_0000L)));
            assertThrows(InvalidInputException.class,
                    () -> client.executeGuardedProtectionItemWithAsynchronousIndicator(
                            null, null, 1, 0x1_0000_0000L, limits));
            assertThrows(InvalidInputException.class,
                    () -> client.executeGuardedProtectionItem(null, null, 0, limits));
        }
    }

    @Test
    void queryAsyncFiltersRejectNullsOversizedListsAndOutOfRangeOperations() throws Exception {
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration configuration = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", 5696)
                        .clientIdentityPem(new byte[][] {testFixture("client-cert.pem")},
                                testFixture("client-key.pem"))
                        .trustCertificatesPem(new byte[][] {testFixture("server-cert.pem")})
                        .build();
                Client client = Client.create(configuration, transport)) {
            CodecLimits limits = CodecLimits.defaults();
            OptionalLong noIndicator = OptionalLong.empty();
            assertThrows(InvalidInputException.class,
                    () -> client.executeQueryAsyncRequests(null, null, null, noIndicator));
            assertThrows(InvalidInputException.class,
                    () -> client.executeQueryAsyncRequests(limits, null, null, null));
            assertThrows(InvalidInputException.class,
                    () -> client.executeQueryAsyncRequests(limits, Arrays.asList(new byte[] {1}, null),
                            null, noIndicator));
            assertThrows(InvalidInputException.class,
                    () -> client.executeQueryAsyncRequests(limits, null,
                            List.of(0x1_0000_0000L), noIndicator));
            assertThrows(InvalidInputException.class,
                    () -> client.executeQueryAsyncRequests(limits, null, null,
                            OptionalLong.of(0x1_0000_0000L)));
            assertThrows(InvalidInputException.class,
                    () -> client.executeQueryAsyncRequests(limits,
                            Collections.nCopies(100_001, new byte[0]), null, noIndicator));
            assertThrows(InvalidInputException.class,
                    () -> client.executeQueryAsyncRequests(limits, null,
                            Collections.nCopies(100_001, 1L), noIndicator));
            assertThrows(InvalidInputException.class,
                    () -> client.executeQueryAsyncRequests(limits,
                            List.of(new byte[16 * 1024 * 1024 + 1]), null, noIndicator));
        }
    }

    private static byte[] testFixture(String filename) throws Exception {
        Path directory = Path.of("").toAbsolutePath();
        while (directory != null) {
            Path candidate = directory.resolve(Path.of(
                    "bindings", "python", "tests", "fixtures", "login-mtls", filename));
            if (Files.isRegularFile(candidate)) {
                return Files.readAllBytes(candidate);
            }
            directory = directory.getParent();
        }
        throw new IllegalStateException("the shared deterministic mTLS fixture is missing");
    }
}
