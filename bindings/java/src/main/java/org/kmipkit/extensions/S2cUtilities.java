package org.kmipkit.extensions;

import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.Optional;
import org.jspecify.annotations.Nullable;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.message.RequestBatchItemView;
import org.kmipkit.protocol.VersionGate;
import org.kmipkit.ttlv.TtlvStructure;

/**
 * Public standalone utilities for KMIP Server-to-Client (S2C) protocol operations.
 *
 * <p>Exposes independent version negotiation, effective Put function computation,
 * Query response payload synthesis, and individual request item validators without
 * requiring execution through the full {@link ServerRequestDispatcher}.</p>
 */
public final class S2cUtilities {
    private static final long MAX_UINT32 = 0xFFFF_FFFFL;
    private static final int MAX_UINT16 = 0xFFFF;

    private S2cUtilities() {}

    /**
     * Negotiates the best mutually supported protocol version for an incoming DiscoverVersions request.
     *
     * @param requestedVersion client-requested protocol version
     * @param supportedVersions list of locally supported protocol versions
     * @return highest negotiated protocol version, or {@link Optional#empty()} if no version matches
     * @throws InvalidInputException if inputs are null or version numbers are outside [0..65535]
     */
    public static Optional<VersionGate.ProtocolVersion> negotiateDiscoverVersions(
            VersionGate.ProtocolVersion requestedVersion,
            List<VersionGate.ProtocolVersion> supportedVersions) {
        if (requestedVersion == null || supportedVersions == null) {
            throw new InvalidInputException("DiscoverVersions negotiation inputs must not be null");
        }
        validateVersionBounds(requestedVersion.major(), requestedVersion.minor(), "requestedVersion");

        int[] packed = new int[supportedVersions.size()];
        for (int i = 0; i < supportedVersions.size(); i++) {
            VersionGate.ProtocolVersion pv = supportedVersions.get(i);
            if (pv == null) {
                throw new InvalidInputException("supportedVersions contains a null entry at index " + i);
            }
            validateVersionBounds(pv.major(), pv.minor(), "supportedVersions[" + i + "]");
            packed[i] = (pv.major() << 16) | (pv.minor() & MAX_UINT16);
        }

        NativeExtensionRegistry.ensureLoaded();
        int[] negotiated = NativeExtensionRegistry.s2cDiscoverVersionsNegotiate(
                requestedVersion.major(), requestedVersion.minor(), packed);
        if (negotiated == null || negotiated.length != 2) {
            return Optional.empty();
        }
        return Optional.of(new VersionGate.ProtocolVersion(negotiated[0], negotiated[1]));
    }

    /**
     * Computes the effective Put function based on whether the replaced object exists locally (§6.2.3).
     *
     * <p>If {@code requestedPutFunction} is Replace (2) but {@code replacedObjectExistsLocally} is false,
     * the effective function is degraded to New (1); otherwise the requested function is preserved.</p>
     *
     * @param requestedPutFunction raw Put Function enumeration value
     * @param replacedObjectExistsLocally true if the target object exists locally
     * @return effective raw Put Function enumeration value
     * @throws InvalidInputException if {@code requestedPutFunction} is outside [0..0xFFFFFFFF]
     */
    public static long effectivePutFunction(long requestedPutFunction, boolean replacedObjectExistsLocally) {
        if (requestedPutFunction < 0 || requestedPutFunction > MAX_UINT32) {
            throw new InvalidInputException("requestedPutFunction out of uint32 range: " + requestedPutFunction);
        }
        NativeExtensionRegistry.ensureLoaded();
        return NativeExtensionRegistry.s2cPutEffectiveFunction(requestedPutFunction, replacedObjectExistsLocally);
    }

    /**
     * Builds a Query response payload TTLV structure using default client capabilities without vendor identification.
     *
     * @param queryFunctionFlags bitmask of requested Query functions (bit 0=Operations, 1=Objects, etc.)
     * @return owned {@link TtlvStructure} containing the response payload
     * @throws InvalidInputException if flags are outside [0..0xFFFFFFFF]
     */
    public static TtlvStructure buildQueryResponseTtlv(long queryFunctionFlags) {
        return buildQueryResponseTtlv(queryFunctionFlags, (String) null);
    }

    /**
     * Builds a Query response payload TTLV structure using default client capabilities.
     *
     * @param queryFunctionFlags bitmask of requested Query functions (bit 0=Operations, 1=Objects, etc.)
     * @param vendorIdentification optional vendor identification text
     * @return owned {@link TtlvStructure} containing the response payload
     * @throws InvalidInputException if flags are outside [0..0xFFFFFFFF]
     */
    public static TtlvStructure buildQueryResponseTtlv(
            long queryFunctionFlags,
            @Nullable String vendorIdentification) {
        if (queryFunctionFlags < 0 || queryFunctionFlags > MAX_UINT32) {
            throw new InvalidInputException("queryFunctionFlags out of uint32 range: " + queryFunctionFlags);
        }
        NativeExtensionRegistry.ensureLoaded();
        byte[] vendorBytes = vendorIdentification == null
                ? null
                : vendorIdentification.getBytes(StandardCharsets.UTF_8);
        long handle = NativeExtensionRegistry.s2cQueryRespondTtlv(queryFunctionFlags, vendorBytes);
        return TtlvStructure.fromNative(handle, null);
    }

    /**
     * Builds a Query response payload TTLV structure reflecting explicit advertised capabilities.
     *
     * @param queryFunctionFlags bitmask of requested Query functions
     * @param capabilities capabilities advertised by the client
     * @return owned {@link TtlvStructure} containing the response payload
     * @throws InvalidInputException if capabilities is null or flags are outside [0..0xFFFFFFFF]
     */
    public static TtlvStructure buildQueryResponseTtlv(
            long queryFunctionFlags,
            S2cCapabilities capabilities) {
        if (capabilities == null) {
            throw new InvalidInputException("capabilities must not be null");
        }
        if (queryFunctionFlags < 0 || queryFunctionFlags > MAX_UINT32) {
            throw new InvalidInputException("queryFunctionFlags out of uint32 range: " + queryFunctionFlags);
        }

        long[] ops = toLongArray(capabilities.operations());
        long[] ots = toLongArray(capabilities.objectTypes());
        long[] exts = toLongArray(capabilities.extensionTags());
        byte[] vendorBytes = capabilities.vendorIdentification() == null
                ? null
                : capabilities.vendorIdentification().getBytes(StandardCharsets.UTF_8);

        NativeExtensionRegistry.ensureLoaded();
        long handle = NativeExtensionRegistry.s2cQueryRespondWithCapabilities(
                queryFunctionFlags, ops, ots, vendorBytes, exts);
        return TtlvStructure.fromNative(handle, null);
    }

    /**
     * Validates whether a request batch item conforms to a KMIP Server-to-Client Notify request.
     *
     * @param item request batch item to validate
     * @return true if valid, false if malformed or mismatched
     * @throws InvalidInputException if item is null
     */
    public static boolean validateNotifyRequest(RequestBatchItemView item) {
        if (item == null) {
            throw new InvalidInputException("request item must not be null");
        }
        NativeExtensionRegistry.ensureLoaded();
        return item.nativeHandleOwner().withValue(NativeExtensionRegistry::s2cNotifyRequestValidate);
    }

    /**
     * Validates whether a request batch item conforms to a KMIP Server-to-Client Put request.
     *
     * @param item request batch item to validate
     * @return true if valid, false if malformed or mismatched
     * @throws InvalidInputException if item is null
     */
    public static boolean validatePutRequest(RequestBatchItemView item) {
        if (item == null) {
            throw new InvalidInputException("request item must not be null");
        }
        NativeExtensionRegistry.ensureLoaded();
        return item.nativeHandleOwner().withValue(NativeExtensionRegistry::s2cPutRequestValidate);
    }

    /**
     * Validates whether a request batch item conforms to a KMIP Server-to-Client Query request.
     *
     * @param item request batch item to validate
     * @return true if valid, false if malformed or mismatched
     * @throws InvalidInputException if item is null
     */
    public static boolean validateQueryRequest(RequestBatchItemView item) {
        if (item == null) {
            throw new InvalidInputException("request item must not be null");
        }
        NativeExtensionRegistry.ensureLoaded();
        return item.nativeHandleOwner().withValue(NativeExtensionRegistry::s2cQueryRequestValidate);
    }

    /**
     * Validates whether a request batch item conforms to a KMIP Server-to-Client DiscoverVersions request.
     *
     * @param item request batch item to validate
     * @return true if valid, false if malformed or mismatched
     * @throws InvalidInputException if item is null
     */
    public static boolean validateDiscoverVersionsRequest(RequestBatchItemView item) {
        if (item == null) {
            throw new InvalidInputException("request item must not be null");
        }
        NativeExtensionRegistry.ensureLoaded();
        return item.nativeHandleOwner().withValue(NativeExtensionRegistry::s2cDiscoverVersionsRequestValidate);
    }

    /**
     * Builds a complete S2C response message TTLV structure from a protocol version and response batch items.
     *
     * @param protocolVersion negotiated response protocol version
     * @param batchItems list of response batch item structures
     * @return owned {@link TtlvStructure} containing the complete response message TTLV
     * @throws InvalidInputException if inputs are null or version bounds are invalid
     */
    public static TtlvStructure buildResponseTtlv(
            VersionGate.ProtocolVersion protocolVersion,
            List<TtlvStructure> batchItems) {
        if (protocolVersion == null || batchItems == null) {
            throw new InvalidInputException("protocolVersion and batchItems must not be null");
        }
        validateVersionBounds(protocolVersion.major(), protocolVersion.minor(), "protocolVersion");

        long[] handles = new long[batchItems.size()];
        for (int i = 0; i < batchItems.size(); i++) {
            TtlvStructure st = batchItems.get(i);
            if (st == null) {
                throw new InvalidInputException("batchItems contains null entry at index " + i);
            }
            handles[i] = st.handle();
        }

        NativeExtensionRegistry.ensureLoaded();
        long handle = NativeExtensionRegistry.s2cBuildResponseTtlv(
                protocolVersion.major(), protocolVersion.minor(), handles);
        return TtlvStructure.fromNative(handle, null);
    }

    private static void validateVersionBounds(int major, int minor, String fieldName) {
        if (major < 0 || major > MAX_UINT16 || minor < 0 || minor > MAX_UINT16) {
            throw new InvalidInputException(fieldName + " version components out of [0..65535] range: "
                    + major + "." + minor);
        }
    }

    private static long[] toLongArray(List<Long> list) {
        if (list == null || list.isEmpty()) {
            return new long[0];
        }
        long[] arr = new long[list.size()];
        for (int i = 0; i < list.size(); i++) {
            arr[i] = list.get(i);
        }
        return arr;
    }
}
