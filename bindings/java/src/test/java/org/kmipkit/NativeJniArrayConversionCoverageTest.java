package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertThrows;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import org.junit.jupiter.api.Test;

final class NativeJniArrayConversionCoverageTest {
    private static final long[] CODEC_LIMITS = {16_777_216, 64, 100_000};
    private static final long[] REGISTRY_LIMITS = {
            256, 16_384, 256, 4_096, 1_048_576, 4_096,
            1_048_576, 256, 16_384, 200_000, 1_048_576, 64
    };

    @Test
    void profileReadinessRejectsMalformedUtf16AtTheNativeBoundary() {
        NativeExtensionRegistry.ensureLoaded();
        String isolatedHighSurrogate = String.valueOf((char) 0xd800);
        String mismatchedSurrogatePair = new String(new char[] {(char) 0xd800, 'x'});
        String isolatedLowSurrogate = String.valueOf((char) 0xdc00);

        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.profileReadinessCreate(
                        isolatedHighSurrogate, new String[0], new boolean[0]));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.profileReadinessCreate(
                        mismatchedSurrogatePair, new String[0], new boolean[0]));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.profileReadinessCreate(
                        isolatedLowSurrogate, new String[0], new boolean[0]));
    }

    @Test
    void getAttributeListRejectsInvalidIdentifierKindsAndValuesAtTheJniBoundary() {
        NativeExtensionRegistry.ensureLoaded();

        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetAttributeList(99, null, 0),
                "unknown identifier kinds must be rejected");
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetAttributeList(3, null, 7),
                "Get Attribute List requires a Text String identifier");
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetAttributeList(
                        1, new byte[] {(byte) 0xff}, 0),
                "malformed identifier bytes must be rejected by the request bridge");
    }

    @Test
    void asynchronousQueryRejectsMalformedAndOversizedJniArraysBeforeNetwork() throws Exception {
        try (NativeClient client = NativeClient.create()) {
            assertThrows(InvalidInputException.class,
                    () -> executeQueryAsync(client.handle(), new byte[][] {null}, null),
                    "null correlation values must be rejected");
            assertThrows(ResourceLimitException.class,
                    () -> executeQueryAsync(client.handle(), new byte[100_001][], null),
                    "correlation arrays over the native count limit must be rejected");
            assertThrows(ResourceLimitException.class,
                    () -> executeQueryAsync(client.handle(), null, new long[100_001]),
                    "operation arrays over the native count limit must be rejected");
            assertThrows(InvalidInputException.class,
                    () -> executeQueryAsync(client.handle(), null, new long[] {0x1_0000_0000L}),
                    "operation values outside the 32-bit range must be rejected");
        }
    }

    private static long executeQueryAsync(long client, byte[][] correlationValues, long[] operations) {
        return NativeExtensionRegistry.clientExecuteQueryAsyncRequests(
                client, correlationValues, operations, false, 0, CODEC_LIMITS);
    }

    private record NativeClient(long registry, long configuration, long handle) implements AutoCloseable {
        private static NativeClient create() throws IOException {
            NativeExtensionRegistry.ensureLoaded();
            long registry = 0;
            long configuration = 0;
            long transport = 0;
            long client = 0;
            try {
                registry = NativeExtensionRegistry.clientExtensionRegistryCreate(
                        new long[0], REGISTRY_LIMITS);
                configuration = NativeExtensionRegistry.clientConfigurationCreate(registry);
                transport = NativeExtensionRegistry.transportConfigCreate(
                        1, "127.0.0.1", 1, 2,
                        new int[] {1}, new int[] {1}, new byte[][] {fixture("client-cert.pem")},
                        1, 1, fixture("client-key.pem"),
                        new int[] {1}, new int[] {1}, new byte[][] {fixture("server-cert.pem")},
                        new int[0], new int[0], new byte[0][],
                        "localhost", null, new long[12], 16_777_216);
                long consumedTransport = transport;
                transport = 0;
                client = NativeExtensionRegistry.clientCreate(configuration, consumedTransport);
                NativeClient result = new NativeClient(registry, configuration, client);
                registry = 0;
                configuration = 0;
                client = 0;
                return result;
            } finally {
                if (client > 0) NativeExtensionRegistry.release(NativeExtensionRegistry.CLIENT, client);
                if (transport > 0) {
                    NativeExtensionRegistry.release(
                            NativeExtensionRegistry.TRANSPORT_CONFIGURATION, transport);
                }
                if (configuration > 0) {
                    NativeExtensionRegistry.release(NativeExtensionRegistry.CONFIGURATION, configuration);
                }
                if (registry > 0) NativeExtensionRegistry.release(NativeExtensionRegistry.REGISTRY, registry);
            }
        }

        @Override
        public void close() {
            NativeExtensionRegistry.release(NativeExtensionRegistry.CLIENT, handle);
            NativeExtensionRegistry.release(NativeExtensionRegistry.CONFIGURATION, configuration);
            NativeExtensionRegistry.release(NativeExtensionRegistry.REGISTRY, registry);
        }
    }

    private static byte[] fixture(String filename) throws IOException {
        Path directory = Path.of("").toAbsolutePath();
        while (directory != null) {
            Path candidate = directory.resolve(
                    Path.of("bindings", "python", "tests", "fixtures", "login-mtls", filename));
            if (Files.isRegularFile(candidate)) return Files.readAllBytes(candidate);
            directory = directory.getParent();
        }
        throw new IOException("shared deterministic mTLS fixture is missing");
    }
}
