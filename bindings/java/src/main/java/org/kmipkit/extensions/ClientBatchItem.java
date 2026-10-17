package org.kmipkit.extensions;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TtlvStructure;
import org.jspecify.annotations.Nullable;

/** Typed request batch item with caller-ordered registry-validated extensions. */
public final class ClientBatchItem implements AutoCloseable {
    private static final int MAX_NATIVE_INPUT_ARRAY_BYTES = 16 * 1024 * 1024;
    private static final int MAX_UNIQUE_BATCH_ITEM_ID_BYTES = MAX_NATIVE_INPUT_ARRAY_BYTES;
    private static final int MAX_OPERATION_INPUT_BYTES = MAX_NATIVE_INPUT_ARRAY_BYTES;

    private final NativeHandle handle;
    private final List<ClientRequestMessageExtension> extensions;

    private ClientBatchItem(long nativeHandle, List<ClientRequestMessageExtension> extensions) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.BATCH_ITEM);
        this.extensions = List.copyOf(extensions);
    }

    /** Creates the supported Discover Versions request batch item. */
    public static ClientBatchItem discoverVersions() {
        NativeExtensionRegistry.ensureLoaded();
        return new ClientBatchItem(NativeExtensionRegistry.clientBatchItemDiscoverVersions(), List.of());
    }

    /** Creates a Register request batch item and consumes the request handle. */
    public static ClientBatchItem register(RegisterRequest request) {
        NativeExtensionRegistry.ensureLoaded();
        if (request == null) {
            throw new InvalidInputException("Register request is invalid");
        }
        long consumed = NativeHandle.transferAll(request.nativeHandleOwner())[0];
        return new ClientBatchItem(NativeExtensionRegistry.clientBatchItemRegister(consumed), List.of());
    }

    /** Creates a Create request while borrowing and copying the supplied TTLV structures. */
    public static ClientBatchItem create(
            long objectType, TtlvStructure attributes, TtlvStructure protectionStorageMasks) {
        NativeExtensionRegistry.ensureLoaded();
        requireStructure(attributes, "Create Attributes");
        requireU32(objectType, "Object Type");
        long value = NativeHandle.withNativeHandles(() -> NativeExtensionRegistry.clientBatchItemCreate(
                objectType, attributes.handle(), handleOrNull(protectionStorageMasks)));
        return new ClientBatchItem(value, List.of());
    }

    /** Creates a Create Key Pair request with distinct, independently ordered attribute groups. */
    public static ClientBatchItem createKeyPair(
            TtlvStructure commonAttributes,
            TtlvStructure privateKeyAttributes,
            TtlvStructure publicKeyAttributes,
            TtlvStructure commonProtectionStorageMasks,
            TtlvStructure privateProtectionStorageMasks,
            TtlvStructure publicProtectionStorageMasks) {
        NativeExtensionRegistry.ensureLoaded();
        long value = NativeHandle.withNativeHandles(() -> NativeExtensionRegistry.clientBatchItemCreateKeyPair(
                handleOrNull(commonAttributes), handleOrNull(privateKeyAttributes),
                handleOrNull(publicKeyAttributes), handleOrNull(commonProtectionStorageMasks),
                handleOrNull(privateProtectionStorageMasks), handleOrNull(publicProtectionStorageMasks)));
        return new ClientBatchItem(value, List.of());
    }

    /** Creates a Create Split Key request, preserving signed fields and optional byte strings. */
    public static ClientBatchItem createSplitKey(
            long objectType,
            UniqueIdentifier uniqueIdentifier,
            int splitKeyParts,
            int splitKeyThreshold,
            long splitKeyMethod,
            byte[] primeFieldSize,
            TtlvStructure attributes,
            TtlvStructure protectionStorageMasks) {
        NativeExtensionRegistry.ensureLoaded();
        requireU32(objectType, "Object Type");
        requireU32(splitKeyMethod, "Split Key Method");
        requireStructure(attributes, "Create Split Key Attributes");
        byte[] identifierText = uniqueIdentifier == null ? null : uniqueIdentifier.nativeText();
        int identifierKind = uniqueIdentifier == null ? 0 : uniqueIdentifier.nativeKind();
        long identifierNumber = uniqueIdentifier == null ? 0 : uniqueIdentifier.nativeNumber();
        byte[] primeFieldCopy = primeFieldSize == null ? null : primeFieldSize.clone();
        long value = NativeHandle.withNativeHandles(() -> NativeExtensionRegistry.clientBatchItemCreateSplitKey(
                objectType, splitKeyParts, splitKeyThreshold, splitKeyMethod, identifierKind,
                identifierText, identifierNumber, primeFieldCopy, attributes.handle(),
                handleOrNull(protectionStorageMasks)));
        return new ClientBatchItem(value, List.of());
    }

    /** Creates a Certify request, preserving raw certificate request type and byte values. */
    public static ClientBatchItem certify(
            @Nullable UniqueIdentifier uniqueIdentifier,
            @Nullable Long certificateRequestType,
            @Nullable byte[] certificateRequestValue,
            @Nullable TtlvStructure attributes,
            @Nullable TtlvStructure protectionStorageMasks) {
        requireOperationInputWithinLimit(certificateRequestValue);
        byte[] identifierText = uniqueIdentifier == null ? null : uniqueIdentifier.nativeText();
        byte[] requestValueCopy = copyForNative(certificateRequestValue);
        try {
            long requestType = certificateRequestType == null ? 0 : certificateRequestType;
            if (certificateRequestType != null) {
                requireU32(requestType, "Certificate Request Type");
            }
            NativeExtensionRegistry.ensureLoaded();
            long value = NativeHandle.withNativeHandles(() -> NativeExtensionRegistry.clientBatchItemCertify(
                    identifierKind(uniqueIdentifier), identifierText, identifierNumber(uniqueIdentifier),
                    certificateRequestType != null, requestType, requestValueCopy,
                    handleOrNull(attributes), handleOrNull(protectionStorageMasks)));
            return new ClientBatchItem(value, List.of());
        } finally {
            clearNativeCopies(identifierText, requestValueCopy);
        }
    }

    /** Creates a Check request with optional §6.1.7 Usage Limits Count, usage mask, and lease. */
    public static ClientBatchItem check(
            @Nullable UniqueIdentifier uniqueIdentifier,
            @Nullable Long usageLimitsCount,
            @Nullable Integer cryptographicUsageMask,
            @Nullable Long leaseTime) {
        byte[] identifierText = uniqueIdentifier == null ? null : uniqueIdentifier.nativeText();
        try {
            long lease = leaseTime == null ? 0 : leaseTime;
            if (leaseTime != null) requireU32(lease, "Lease Time");
            NativeExtensionRegistry.ensureLoaded();
            long value = NativeHandle.withNativeHandles(() -> NativeExtensionRegistry.clientBatchItemCheck(
                    identifierKind(uniqueIdentifier), identifierText, identifierNumber(uniqueIdentifier),
                    usageLimitsCount != null, usageLimitsCount == null ? 0 : usageLimitsCount,
                    cryptographicUsageMask != null,
                    cryptographicUsageMask == null ? 0 : cryptographicUsageMask,
                    leaseTime != null, lease));
            return new ClientBatchItem(value, List.of());
        } finally {
            clearNativeCopies(identifierText);
        }
    }

    /** Creates a §6.1.12 Delegated Login request with generic Rights and Usage Limits TTLV. */
    public static ClientBatchItem delegatedLogin(
            TtlvStructure rights,
            @Nullable Long leaseTimeInterval,
            @Nullable Long leaseTimeDateTime,
            @Nullable Integer requestCount,
            @Nullable TtlvStructure usageLimits) {
        requireStructure(rights, "Delegated Login Rights");
        if (leaseTimeInterval != null && leaseTimeDateTime != null) {
            throw new InvalidInputException("Delegated Login Lease Time forms are mutually exclusive");
        }
        final int leaseTimeKind;
        final long leaseTimeValue;
        if (leaseTimeInterval != null) {
            leaseTimeKind = 1;
            requireU32(leaseTimeInterval, "Delegated Login Lease Time Interval");
            leaseTimeValue = leaseTimeInterval;
        } else if (leaseTimeDateTime != null) {
            leaseTimeKind = 2;
            leaseTimeValue = leaseTimeDateTime;
        } else {
            leaseTimeKind = 0;
            leaseTimeValue = 0;
        }
        NativeExtensionRegistry.ensureLoaded();
        long value = NativeHandle.withNativeHandles(() ->
                NativeExtensionRegistry.clientBatchItemDelegatedLogin(
                        rights.handle(), leaseTimeKind, leaseTimeValue,
                        requestCount != null, requestCount == null ? 0 : requestCount,
                        handleOrNull(usageLimits)));
        return new ClientBatchItem(value, List.of());
    }

    /** Creates a Get Usage Allocation request with an optional object identifier and exact count. */
    public static ClientBatchItem getUsageAllocation(
            @Nullable UniqueIdentifier uniqueIdentifier, long usageLimitsCount) {
        byte[] identifierText = uniqueIdentifier == null ? null : uniqueIdentifier.nativeText();
        try {
            NativeExtensionRegistry.ensureLoaded();
            long value = NativeHandle.withNativeHandles(() ->
                    NativeExtensionRegistry.clientBatchItemGetUsageAllocation(
                            identifierKind(uniqueIdentifier), identifierText,
                            identifierNumber(uniqueIdentifier), usageLimitsCount));
            return new ClientBatchItem(value, List.of());
        } finally {
            clearNativeCopies(identifierText);
        }
    }

    /** Creates a Hash request with required Cryptographic Parameters and optional Data/framing fields. */
    public static ClientBatchItem hash(
            TtlvStructure parameters,
            @Nullable TtlvValue data,
            @Nullable byte[] correlationValue,
            @Nullable Boolean initIndicator,
            @Nullable Boolean finalIndicator) {
        requireStructure(parameters, "Hash Cryptographic Parameters");
        byte[] correlationCopy = copyForNative(correlationValue);
        try {
            NativeExtensionRegistry.ensureLoaded();
            long value = NativeHandle.withNativeHandles(() -> NativeExtensionRegistry.clientBatchItemHash(
                    parameters.handle(), handleOrNull(data), correlationCopy,
                    optionalIndicator(initIndicator), optionalIndicator(finalIndicator)));
            return new ClientBatchItem(value, List.of());
        } finally {
            clearNativeCopies(correlationCopy);
        }
    }

    /** Creates a MAC request, preserving absent and present-empty optional fields. */
    public static ClientBatchItem mac(
            @Nullable UniqueIdentifier uniqueIdentifier,
            @Nullable TtlvStructure parameters,
            @Nullable TtlvValue data,
            @Nullable byte[] correlationValue,
            @Nullable Boolean initIndicator,
            @Nullable Boolean finalIndicator) {
        byte[] identifierText = uniqueIdentifier == null ? null : uniqueIdentifier.nativeText();
        byte[] correlationCopy = copyForNative(correlationValue);
        try {
            NativeExtensionRegistry.ensureLoaded();
            long value = NativeHandle.withNativeHandles(() -> NativeExtensionRegistry.clientBatchItemMac(
                    identifierKind(uniqueIdentifier), identifierText, identifierNumber(uniqueIdentifier),
                    handleOrNull(parameters), handleOrNull(data), correlationCopy,
                    optionalIndicator(initIndicator), optionalIndicator(finalIndicator)));
            return new ClientBatchItem(value, List.of());
        } finally {
            clearNativeCopies(identifierText, correlationCopy);
        }
    }

    /** Creates a MAC Verify request with optional MAC data and asynchronous framing fields. */
    public static ClientBatchItem macVerify(
            @Nullable UniqueIdentifier uniqueIdentifier,
            @Nullable TtlvStructure parameters,
            @Nullable TtlvValue data,
            @Nullable byte[] macData,
            @Nullable byte[] correlationValue,
            @Nullable Boolean initIndicator,
            @Nullable Boolean finalIndicator) {
        requireOperationInputWithinLimit(macData);
        byte[] identifierText = uniqueIdentifier == null ? null : uniqueIdentifier.nativeText();
        byte[] macCopy = copyForNative(macData);
        byte[] correlationCopy = copyForNative(correlationValue);
        try {
            NativeExtensionRegistry.ensureLoaded();
            long value = NativeHandle.withNativeHandles(() -> NativeExtensionRegistry.clientBatchItemMacVerify(
                    identifierKind(uniqueIdentifier), identifierText, identifierNumber(uniqueIdentifier),
                    handleOrNull(parameters), handleOrNull(data), macCopy, correlationCopy,
                    optionalIndicator(initIndicator), optionalIndicator(finalIndicator)));
            return new ClientBatchItem(value, List.of());
        } finally {
            clearNativeCopies(identifierText, macCopy, correlationCopy);
        }
    }

    /** Creates a Sign request with optional Digested Data and asynchronous framing fields. */
    public static ClientBatchItem sign(
            @Nullable UniqueIdentifier uniqueIdentifier,
            @Nullable TtlvStructure parameters,
            @Nullable TtlvValue data,
            @Nullable byte[] digestedData,
            @Nullable byte[] correlationValue,
            @Nullable Boolean initIndicator,
            @Nullable Boolean finalIndicator) {
        requireOperationInputWithinLimit(digestedData);
        byte[] identifierText = uniqueIdentifier == null ? null : uniqueIdentifier.nativeText();
        byte[] digestedCopy = copyForNative(digestedData);
        byte[] correlationCopy = copyForNative(correlationValue);
        try {
            NativeExtensionRegistry.ensureLoaded();
            long value = NativeHandle.withNativeHandles(() -> NativeExtensionRegistry.clientBatchItemSign(
                    identifierKind(uniqueIdentifier), identifierText, identifierNumber(uniqueIdentifier),
                    handleOrNull(parameters), handleOrNull(data), digestedCopy, correlationCopy,
                    optionalIndicator(initIndicator), optionalIndicator(finalIndicator)));
            return new ClientBatchItem(value, List.of());
        } finally {
            clearNativeCopies(identifierText, digestedCopy, correlationCopy);
        }
    }

    /** Creates a Signature Verify request with optional digest, signature, and framing fields. */
    public static ClientBatchItem signatureVerify(
            @Nullable UniqueIdentifier uniqueIdentifier,
            @Nullable TtlvStructure parameters,
            @Nullable TtlvValue data,
            @Nullable byte[] digestedData,
            @Nullable byte[] signatureData,
            @Nullable byte[] correlationValue,
            @Nullable Boolean initIndicator,
            @Nullable Boolean finalIndicator) {
        requireOperationInputWithinLimit(digestedData);
        requireOperationInputWithinLimit(signatureData);
        byte[] identifierText = uniqueIdentifier == null ? null : uniqueIdentifier.nativeText();
        byte[] digestedCopy = copyForNative(digestedData);
        byte[] signatureCopy = copyForNative(signatureData);
        byte[] correlationCopy = copyForNative(correlationValue);
        try {
            NativeExtensionRegistry.ensureLoaded();
            long value = NativeHandle.withNativeHandles(() -> NativeExtensionRegistry.clientBatchItemSignatureVerify(
                    identifierKind(uniqueIdentifier), identifierText, identifierNumber(uniqueIdentifier),
                    handleOrNull(parameters), handleOrNull(data), digestedCopy, signatureCopy,
                    correlationCopy, optionalIndicator(initIndicator), optionalIndicator(finalIndicator)));
            return new ClientBatchItem(value, List.of());
        } finally {
            clearNativeCopies(identifierText, digestedCopy, signatureCopy, correlationCopy);
        }
    }

    private static void requireStructure(TtlvStructure structure, String label) {
        if (structure == null) {
            throw new InvalidInputException(label + " are required");
        }
    }

    private static long handleOrNull(TtlvStructure structure) {
        return structure == null ? 0 : structure.handle();
    }

    private static long handleOrNull(TtlvValue value) {
        return value == null ? 0 : value.handle();
    }

    private static int optionalIndicator(Boolean indicator) {
        return indicator == null ? 0 : indicator ? 2 : 1;
    }

    private static int identifierKind(UniqueIdentifier identifier) {
        return identifier == null ? 0 : identifier.nativeKind();
    }

    private static long identifierNumber(UniqueIdentifier identifier) {
        return identifier == null ? 0 : identifier.nativeNumber();
    }

    private static byte[] copyForNative(byte[] value) {
        return value == null ? null : value.clone();
    }

    private static void requireOperationInputWithinLimit(byte[] value) {
        if (value != null && value.length > MAX_OPERATION_INPUT_BYTES) {
            throw new ResourceLimitException("operation input exceeds the configured limit");
        }
    }

    private static void clearNativeCopies(byte[]... values) {
        for (byte[] value : values) {
            if (value != null) {
                Arrays.fill(value, (byte) 0);
            }
        }
    }

    private static void requireU32(long value, String label) {
        if (value < 0 || value > 0xffff_ffffL) {
            throw new InvalidInputException(label + " is outside uint32");
        }
    }

    public ClientBatchItem withExtension(ClientRequestMessageExtension extension) {
        NativeExtensionRegistry.ensureLoaded();
        long[] consumed = NativeHandle.transferAll(handle, extension.handleOwner());
        long nextHandle = NativeExtensionRegistry.clientBatchItemWithExtension(
                consumed[0], consumed[1]);
        ArrayList<ClientRequestMessageExtension> next = new ArrayList<>(extensions);
        next.add(extension);
        return new ClientBatchItem(nextHandle, next);
    }

    /** Returns a new item carrying the caller-selected opaque KMIP identifier bytes. */
    public ClientBatchItem withUniqueBatchItemId(byte[] identifier) {
        if (identifier == null) {
            throw new InvalidInputException("unique batch item ID is invalid");
        }
        if (identifier.length > MAX_UNIQUE_BATCH_ITEM_ID_BYTES) {
            throw new ResourceLimitException("unique batch item ID exceeds the configured limit");
        }
        byte[] copy = identifier.clone();
        NativeExtensionRegistry.ensureLoaded();
        long consumedItem = handle.transfer();
        long nextHandle = NativeExtensionRegistry.clientBatchItemWithUniqueBatchItemId(
                consumedItem, copy);
        return new ClientBatchItem(nextHandle, extensions);
    }

    List<ClientRequestMessageExtension> extensions() {
        return extensions;
    }

    NativeHandle nativeHandleOwner() {
        return handle;
    }

    /** Returns the number of attached vendor extensions. */
    public long extensionCount() {
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(NativeExtensionRegistry::clientBatchItemExtensionCount);
    }

    /** Returns a copied identity for the extension at {@code index}. */
    public ExtensionIdentity extensionIdentityAt(long index) {
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(value -> ExtensionIdentity.fromNative(
                NativeExtensionRegistry.clientBatchItemExtensionIdentityAt(value, index)));
    }

    /** Returns the extension's caller-selected Criticality Indicator. */
    public boolean extensionCriticalityIndicatorAt(long index) {
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(value ->
                NativeExtensionRegistry.clientBatchItemExtensionCriticalityIndicatorAt(value, index));
    }
    @Override
    public void close() {
        handle.close();
    }
}
