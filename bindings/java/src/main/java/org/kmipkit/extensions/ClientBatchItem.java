package org.kmipkit.extensions;

import java.nio.ByteBuffer;
import java.nio.CharBuffer;
import java.nio.charset.CharacterCodingException;
import java.nio.charset.CodingErrorAction;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TtlvStructure;
import org.jspecify.annotations.Nullable;

/** Typed request batch item with caller-ordered registry-validated extensions. */
public final class ClientBatchItem implements AutoCloseable {
    private static final int MAX_NATIVE_INPUT_ARRAY_BYTES = 16 * 1024 * 1024;
    private static final int MAX_UNIQUE_BATCH_ITEM_ID_BYTES = MAX_NATIVE_INPUT_ARRAY_BYTES;
    private static final int MAX_OPERATION_INPUT_BYTES = MAX_NATIVE_INPUT_ARRAY_BYTES;
    private static final int MAX_DERIVE_KEY_SOURCE_IDENTIFIERS = 100_000;
    private static final int MAX_VALIDATE_INPUT_ITEMS = 100_000;
    private static final int MAX_SPLIT_KEY_REFERENCES = 100_000;
    private static final long MAX_TTLV_ITEM_LENGTH = 0xffff_ffffL;

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

    /** Creates an empty client-initiated KMIP Ping request batch item. */
    public static ClientBatchItem ping() {
        NativeExtensionRegistry.ensureLoaded();
        return new ClientBatchItem(NativeExtensionRegistry.clientBatchItemPing(), List.of());
    }

    /** Creates a KMIP 2.1 §6.1.31 Logout request with the required caller-supplied Ticket. */
    public static ClientBatchItem logout(TtlvStructure ticket) {
        requireStructure(ticket, "Logout Ticket");
        NativeExtensionRegistry.ensureLoaded();
        long value = NativeHandle.withNativeHandles(() ->
                NativeExtensionRegistry.clientBatchItemLogout(ticket.handle()));
        return new ClientBatchItem(value, List.of());
    }

    /**
     * Creates a KMIP Query request after checking its payload against the default codec limits.
     * The complete Request Message is checked again before transmission. Raw function values and
     * their order are preserved. A null {@code objectGroups} omits Object Groups, while an empty
     * array includes it as present-empty.
     */
    public static ClientBatchItem query(int[] functions, String[] objectGroups) {
        return queryWithLimits(functions, objectGroups, CodecLimits.defaults());
    }

    /**
     * Creates a KMIP Query request after checking the Query payload against caller-selected
     * codec limits. The complete Request Message is checked again before transmission.
     */
    public static ClientBatchItem queryWithLimits(
            int[] functions, String[] objectGroups, CodecLimits limits) {
        if (functions == null || functions.length == 0) {
            throw new InvalidInputException("Query requires at least one function");
        }
        if (limits == null) {
            throw new InvalidInputException("Query codec limits are required");
        }
        preflightQuery(functions, objectGroups, limits);
        byte[][] encodedGroups = null;
        try {
            if (objectGroups != null) {
                encodedGroups = new byte[objectGroups.length][];
                for (int index = 0; index < objectGroups.length; index++) {
                    String objectGroup = objectGroups[index];
                    if (objectGroup == null) {
                        throw new InvalidInputException("Query Object Group must not be null");
                    }
                    byte[] encoded = encodeQueryObjectGroup(objectGroup);
                    encodedGroups[index] = encoded;
                }
            }
            NativeExtensionRegistry.ensureLoaded();
            return new ClientBatchItem(
                    NativeExtensionRegistry.clientBatchItemQueryWithLimits(
                            functions, encodedGroups, NativeExtensionRegistry.codecLimitValues(limits)),
                    List.of());
        } finally {
            if (encodedGroups != null) {
                for (byte[] group : encodedGroups) {
                    if (group != null) Arrays.fill(group, (byte) 0);
                }
            }
        }
    }

    private static void preflightQuery(int[] functions, String[] objectGroups, CodecLimits limits) {
        long elements = functions.length + 1L;
        long encodedBytes = 8L + 16L * functions.length;
        long objectGroupsValueLength = 0L;
        if (objectGroups != null) {
            elements = checkedQueryAdd(elements, objectGroups.length + 1L);
            encodedBytes = checkedQueryAdd(encodedBytes, 8L);
            for (String group : objectGroups) {
                if (group == null) {
                    throw new InvalidInputException("Query Object Group must not be null");
                }
                long utf8Length = requireTtlvItemLength(strictUtf8Length(group));
                long paddedLength = checkedQueryAdd(utf8Length, 7L) & ~7L;
                long itemBytes = checkedQueryAdd(8L, paddedLength);
                objectGroupsValueLength = checkedQueryAdd(objectGroupsValueLength, itemBytes);
                requireTtlvItemLength(objectGroupsValueLength);
                encodedBytes = checkedQueryAdd(encodedBytes, itemBytes);
                if (encodedBytes > limits.maxMessageBytes()) {
                    throw new ResourceLimitException("Query exceeds the configured message limit");
                }
            }
        }
        requireTtlvItemLength(encodedBytes - 8L);
        if (elements > limits.maxElements() || encodedBytes > limits.maxMessageBytes()) {
            throw new ResourceLimitException("Query exceeds the configured codec limits");
        }
        long requiredDepth = objectGroups == null ? 3L : 4L;
        if (requiredDepth > limits.maxStructureDepth()) {
            throw new ResourceLimitException("Query exceeds the configured structure depth");
        }
    }

    private static long strictUtf8Length(String value) {
        long length = 0;
        for (int index = 0; index < value.length(); index++) {
            char current = value.charAt(index);
            if (Character.isHighSurrogate(current)) {
                if (index + 1 >= value.length() || !Character.isLowSurrogate(value.charAt(index + 1))) {
                    throw new InvalidInputException("Query Object Group is not valid Unicode");
                }
                index++;
                length = checkedQueryAdd(length, 4L);
            } else if (Character.isLowSurrogate(current)) {
                throw new InvalidInputException("Query Object Group is not valid Unicode");
            } else {
                length = checkedQueryAdd(length,
                        current <= 0x7f ? 1L : current <= 0x7ff ? 2L : 3L);
            }
        }
        return length;
    }

    private static long checkedQueryAdd(long left, long right) {
        try {
            return Math.addExact(left, right);
        } catch (ArithmeticException exception) {
            throw new ResourceLimitException("Query exceeds the configured codec limits");
        }
    }

    static long requireTtlvItemLength(long length) {
        if (length < 0 || length > MAX_TTLV_ITEM_LENGTH) {
            throw new ResourceLimitException("Query Object Group exceeds the TTLV Item Length limit");
        }
        return length;
    }

    private static byte[] encodeQueryObjectGroup(String value) {
        try {
            ByteBuffer encoded = StandardCharsets.UTF_8.newEncoder()
                    .onMalformedInput(CodingErrorAction.REPORT)
                    .onUnmappableCharacter(CodingErrorAction.REPORT)
                    .encode(CharBuffer.wrap(value));
            byte[] bytes = new byte[encoded.remaining()];
            encoded.get(bytes);
            return bytes;
        } catch (CharacterCodingException exception) {
            throw new InvalidInputException("Query Object Group is not valid Unicode");
        }
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

    /** Creates a Revoke request batch item and consumes the request handle. */
    public static ClientBatchItem revoke(RevokeRequest request) {
        NativeExtensionRegistry.ensureLoaded();
        if (request == null) {
            throw new InvalidInputException("Revoke request is invalid");
        }
        long consumed = NativeHandle.transferAll(request.nativeHandleOwner())[0];
        return new ClientBatchItem(NativeExtensionRegistry.clientBatchItemRevoke(consumed), List.of());
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

    /** Creates a Re-certify request using the fields in §6.1.45, Table 300. */
    public static ClientBatchItem recertify(
            @Nullable UniqueIdentifier uniqueIdentifier,
            @Nullable UniqueIdentifier certificateRequestUniqueIdentifier,
            @Nullable Long certificateRequestType,
            @Nullable byte[] certificateRequestValue,
            @Nullable Long offset,
            @Nullable TtlvStructure attributes,
            @Nullable TtlvStructure protectionStorageMasks) {
        requireOperationInputWithinLimit(certificateRequestValue);
        byte[] identifierText = uniqueIdentifier == null ? null : uniqueIdentifier.nativeText();
        byte[] certificateRequestIdentifierText = certificateRequestUniqueIdentifier == null
                ? null : certificateRequestUniqueIdentifier.nativeText();
        byte[] requestValueCopy = copyForNative(certificateRequestValue);
        try {
            long requestType = certificateRequestType == null ? 0 : certificateRequestType;
            if (certificateRequestType != null) {
                requireU32(requestType, "Certificate Request Type");
            }
            long offsetValue = offset == null ? 0 : offset;
            if (offset != null) {
                requireU32(offsetValue, "Offset");
            }
            NativeExtensionRegistry.ensureLoaded();
            long value = NativeHandle.withNativeHandles(() ->
                    NativeExtensionRegistry.clientBatchItemRecertify(
                            identifierKind(uniqueIdentifier), identifierText,
                            identifierNumber(uniqueIdentifier),
                            identifierKind(certificateRequestUniqueIdentifier),
                            certificateRequestIdentifierText,
                            identifierNumber(certificateRequestUniqueIdentifier),
                            certificateRequestType != null, requestType, requestValueCopy,
                            offset != null, offsetValue, handleOrNull(attributes),
                            handleOrNull(protectionStorageMasks)));
            return new ClientBatchItem(value, List.of());
        } finally {
            clearNativeCopies(identifierText, certificateRequestIdentifierText, requestValueCopy);
        }
    }

    /** Creates a Re-key request using the fields in KMIP 2.1 §6.1.46, Table 305. */
    public static ClientBatchItem rekey(
            @Nullable UniqueIdentifier uniqueIdentifier,
            @Nullable Long offset,
            @Nullable TtlvStructure attributes,
            @Nullable TtlvStructure protectionStorageMasks) {
        byte[] identifierText = uniqueIdentifier == null ? null : uniqueIdentifier.nativeText();
        try {
            long offsetValue = offset == null ? 0 : offset;
            if (offset != null) requireU32(offsetValue, "Offset");
            NativeExtensionRegistry.ensureLoaded();
            long value = NativeHandle.withNativeHandles(() ->
                    NativeExtensionRegistry.clientBatchItemRekey(
                            identifierKind(uniqueIdentifier), identifierText,
                            identifierNumber(uniqueIdentifier), offset != null, offsetValue,
                            handleOrNull(attributes), handleOrNull(protectionStorageMasks)));
            return new ClientBatchItem(value, List.of());
        } finally {
            clearNativeCopies(identifierText);
        }
    }

    /** Creates a Re-key Key Pair request from the optional fields in KMIP 2.1 §6.1.47, Table 310. */
    public static ClientBatchItem rekeyKeyPair(
            @Nullable UniqueIdentifier privateKeyUniqueIdentifier,
            @Nullable Long offset,
            @Nullable TtlvStructure commonAttributes,
            @Nullable TtlvStructure privateKeyAttributes,
            @Nullable TtlvStructure publicKeyAttributes,
            @Nullable TtlvStructure commonProtectionStorageMasks,
            @Nullable TtlvStructure privateProtectionStorageMasks,
            @Nullable TtlvStructure publicProtectionStorageMasks) {
        byte[] identifierText = privateKeyUniqueIdentifier == null ? null : privateKeyUniqueIdentifier.nativeText();
        try {
            long offsetValue = offset == null ? 0 : offset;
            if (offset != null) requireU32(offsetValue, "Offset");
            NativeExtensionRegistry.ensureLoaded();
            long value = NativeHandle.withNativeHandles(() -> NativeExtensionRegistry.clientBatchItemRekeyKeyPair(
                    identifierKind(privateKeyUniqueIdentifier), identifierText,
                    identifierNumber(privateKeyUniqueIdentifier), offset != null, offsetValue,
                    handleOrNull(commonAttributes), handleOrNull(privateKeyAttributes),
                    handleOrNull(publicKeyAttributes), handleOrNull(commonProtectionStorageMasks),
                    handleOrNull(privateProtectionStorageMasks), handleOrNull(publicProtectionStorageMasks)));
            return new ClientBatchItem(value, List.of());
        } finally {
            clearNativeCopies(identifierText);
        }
    }

    /** Creates a Re-Provision request using KMIP 2.1 §6.1.48, Tables 313–315. */
    public static ClientBatchItem reprovision(
            @Nullable TtlvStructure certificateRequest,
            @Nullable TtlvStructure certificate) {
        if (certificateRequest != null && certificate != null) {
            throw new InvalidInputException(
                    "Re-Provision accepts at most one Certificate Request or Certificate");
        }
        NativeExtensionRegistry.ensureLoaded();
        long value = NativeHandle.withNativeHandles(() -> NativeExtensionRegistry.clientBatchItemReprovision(
                handleOrNull(certificateRequest), handleOrNull(certificate)));
        return new ClientBatchItem(value, List.of());
    }

    /** Creates a §6.1.49 RNG Retrieve request with the caller's signed Data Length. */
    public static ClientBatchItem rngRetrieve(int dataLength) {
        NativeExtensionRegistry.ensureLoaded();
        long value = NativeHandle.withNativeHandles(
                () -> NativeExtensionRegistry.clientBatchItemRngRetrieve(dataLength));
        return new ClientBatchItem(value, List.of());
    }

    /** Creates a §6.1.50 RNG Seed request from required secret seed bytes. */
    public static ClientBatchItem rngSeed(byte[] data) {
        if (data == null) throw new InvalidInputException("RNG Seed Data is required");
        NativeExtensionRegistry.ensureLoaded();
        long value = NativeHandle.withNativeHandles(
                () -> NativeExtensionRegistry.clientBatchItemRngSeed(data));
        return new ClientBatchItem(value, List.of());
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

    /** Creates a §6.1.30 Login request with optional Lease Time, Request Count, and Usage Limits. */
    public static ClientBatchItem login(
            @Nullable Long leaseTimeInterval,
            @Nullable Long leaseTimeDateTime,
            @Nullable Integer requestCount,
            @Nullable TtlvStructure usageLimits) {
        if (leaseTimeInterval != null && leaseTimeDateTime != null) {
            throw new InvalidInputException("Login Lease Time forms are mutually exclusive");
        }
        final int leaseTimeKind;
        final long leaseTimeValue;
        if (leaseTimeInterval != null) {
            leaseTimeKind = 1;
            requireU32(leaseTimeInterval, "Login Lease Time Interval");
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
                NativeExtensionRegistry.clientBatchItemLogin(
                        leaseTimeKind, leaseTimeValue, requestCount != null,
                        requestCount == null ? 0 : requestCount, handleOrNull(usageLimits)));
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

    /** Creates an Obtain Lease request; a null identifier uses the KMIP batch placeholder. */
    public static ClientBatchItem obtainLease(@Nullable UniqueIdentifier uniqueIdentifier) {
        byte[] identifierText = uniqueIdentifier == null ? null : uniqueIdentifier.nativeText();
        try {
            NativeExtensionRegistry.ensureLoaded();
            long value = NativeHandle.withNativeHandles(() ->
                    NativeExtensionRegistry.clientBatchItemObtainLease(
                            identifierKind(uniqueIdentifier), identifierText,
                            identifierNumber(uniqueIdentifier)));
            return new ClientBatchItem(value, List.of());
        } finally {
            clearNativeCopies(identifierText);
        }
    }

    /** Creates an Import request from caller-supplied Attributes and opaque Any Object data. */
    public static ClientBatchItem importObject(
            UniqueIdentifier uniqueIdentifier,
            long objectType,
            @Nullable Boolean replaceExisting,
            @Nullable Long keyWrapType,
            TtlvStructure attributes,
            TtlvStructure anyObject) {
        if (uniqueIdentifier == null) {
            throw new InvalidInputException("Import Unique Identifier is required");
        }
        requireU32(objectType, "Import Object Type");
        if (keyWrapType != null) requireU32(keyWrapType, "Import Key Wrap Type");
        requireStructure(attributes, "Import Attributes");
        requireStructure(anyObject, "Import Any Object");
        byte[] identifierText = uniqueIdentifier.nativeText();
        try {
            NativeExtensionRegistry.ensureLoaded();
            long value = NativeHandle.withNativeHandles(() ->
                    NativeExtensionRegistry.clientBatchItemImport(
                            identifierKind(uniqueIdentifier), identifierText,
                            identifierNumber(uniqueIdentifier), objectType,
                            replaceExisting != null, Boolean.TRUE.equals(replaceExisting),
                            keyWrapType != null, keyWrapType == null ? 0 : keyWrapType,
                            handleOrNull(attributes), handleOrNull(anyObject)));
            return new ClientBatchItem(value, List.of());
        } finally {
            clearNativeCopies(identifierText);
        }
    }

    /** Creates a Join Split Key request with an individual local threshold for each reference. */
    public static ClientBatchItem joinSplitKey(
            long objectType,
            List<SplitKeyReference> references,
            @Nullable Long secretDataType,
            @Nullable TtlvStructure attributes,
            @Nullable TtlvStructure protectionStorageMasks) {
        requireU32(objectType, "Join Split Key Object Type");
        if (references == null || references.isEmpty() || references.size() > MAX_SPLIT_KEY_REFERENCES) {
            throw new InvalidInputException("Join Split Key references are invalid");
        }
        long maximumThreshold = 0;
        SplitKeyReference[] snapshot = references.toArray(SplitKeyReference[]::new);
        for (SplitKeyReference reference : snapshot) {
            if (reference == null) {
                throw new InvalidInputException("Join Split Key reference is invalid");
            }
            maximumThreshold = Math.max(maximumThreshold, reference.splitKeyThreshold());
        }
        if (snapshot.length < maximumThreshold) {
            throw new InvalidInputException("Join Split Key reference count is below its threshold");
        }
        if (secretDataType != null) requireU32(secretDataType, "Join Split Key Secret Data Type");
        NativeHandle[] owners = new NativeHandle[snapshot.length + 2];
        for (int index = 0; index < snapshot.length; index++) {
            owners[index] = snapshot[index].nativeHandleOwner();
        }
        owners[snapshot.length] = attributes == null ? null : attributes.nativeHandleOwner();
        owners[snapshot.length + 1] = protectionStorageMasks == null
                ? null : protectionStorageMasks.nativeHandleOwner();
        NativeExtensionRegistry.ensureLoaded();
        long nativeHandle = NativeHandle.withNativeHandles(() -> NativeHandle.withValues(
                java.util.Arrays.stream(owners).filter(java.util.Objects::nonNull)
                        .toArray(NativeHandle[]::new), values -> {
                    long[] nativeReferences = java.util.Arrays.copyOf(values, snapshot.length);
                    int structureIndex = snapshot.length;
                    long nativeAttributes = attributes == null ? 0 : values[structureIndex++];
                    long nativeMasks = protectionStorageMasks == null ? 0 : values[structureIndex];
                    return NativeExtensionRegistry.clientBatchItemJoinSplitKey(
                            objectType, nativeReferences, secretDataType != null,
                            secretDataType == null ? 0 : secretDataType,
                            nativeAttributes, nativeMasks);
                }));
        return new ClientBatchItem(nativeHandle, List.of());
    }

    /** Creates a Join Split Key request with no optional Table 244 values. */
    public static ClientBatchItem joinSplitKey(long objectType, List<SplitKeyReference> references) {
        return joinSplitKey(objectType, references, null, null, null);
    }

    /** Creates an Activate request with the optional §6.1.1 Unique Identifier. */
    public static ClientBatchItem activate(@Nullable UniqueIdentifier uniqueIdentifier) {
        byte[] identifierText = uniqueIdentifier == null ? null : uniqueIdentifier.nativeText();
        try {
            NativeExtensionRegistry.ensureLoaded();
            long value = NativeExtensionRegistry.clientBatchItemActivate(
                    identifierKind(uniqueIdentifier), identifierText,
                    identifierNumber(uniqueIdentifier));
            return new ClientBatchItem(value, List.of());
        } finally {
            clearNativeCopies(identifierText);
        }
    }

    /** Creates an Archive request with the optional §6.1.4 Unique Identifier. */
    public static ClientBatchItem archive(@Nullable UniqueIdentifier uniqueIdentifier) {
        byte[] identifierText = uniqueIdentifier == null ? null : uniqueIdentifier.nativeText();
        try {
            NativeExtensionRegistry.ensureLoaded();
            long value = NativeExtensionRegistry.clientBatchItemArchive(
                    identifierKind(uniqueIdentifier), identifierText,
                    identifierNumber(uniqueIdentifier));
            return new ClientBatchItem(value, List.of());
        } finally {
            clearNativeCopies(identifierText);
        }
    }

    /** Creates a Destroy request with the optional §6.1.15 Unique Identifier. */
    public static ClientBatchItem destroy(@Nullable UniqueIdentifier uniqueIdentifier) {
        byte[] identifierText = uniqueIdentifier == null ? null : uniqueIdentifier.nativeText();
        try {
            NativeExtensionRegistry.ensureLoaded();
            long value = NativeExtensionRegistry.clientBatchItemDestroy(
                    identifierKind(uniqueIdentifier), identifierText,
                    identifierNumber(uniqueIdentifier));
            return new ClientBatchItem(value, List.of());
        } finally {
            clearNativeCopies(identifierText);
        }
    }

    /** Creates a Recover request with the optional §6.1.42 Unique Identifier. */
    public static ClientBatchItem recover(@Nullable UniqueIdentifier uniqueIdentifier) {
        byte[] identifierText = uniqueIdentifier == null ? null : uniqueIdentifier.nativeText();
        try {
            NativeExtensionRegistry.ensureLoaded();
            long value = NativeExtensionRegistry.clientBatchItemRecover(
                    identifierKind(uniqueIdentifier), identifierText,
                    identifierNumber(uniqueIdentifier));
            return new ClientBatchItem(value, List.of());
        } finally {
            clearNativeCopies(identifierText);
        }
    }

    /** Creates a Get request with the optional Table 220 fields in their canonical order. */
    public static ClientBatchItem get(
            @Nullable UniqueIdentifier uniqueIdentifier,
            @Nullable Long keyFormatType,
            @Nullable Long keyWrapType,
            @Nullable Long keyCompressionType,
            @Nullable TtlvStructure keyWrappingSpecification) {
        byte[] identifierText = uniqueIdentifier == null ? null : uniqueIdentifier.nativeText();
        try {
            if (keyFormatType != null) requireU32(keyFormatType, "Key Format Type");
            if (keyWrapType != null) requireU32(keyWrapType, "Key Wrap Type");
            if (keyCompressionType != null) requireU32(keyCompressionType, "Key Compression Type");
            NativeExtensionRegistry.ensureLoaded();
            long value = NativeHandle.withNativeHandles(() ->
                    NativeExtensionRegistry.clientBatchItemGet(
                            identifierKind(uniqueIdentifier), identifierText,
                            identifierNumber(uniqueIdentifier),
                            keyFormatType != null, keyFormatType == null ? 0 : keyFormatType,
                            keyWrapType != null, keyWrapType == null ? 0 : keyWrapType,
                            keyCompressionType != null,
                            keyCompressionType == null ? 0 : keyCompressionType,
                            handleOrNull(keyWrappingSpecification)));
            return new ClientBatchItem(value, List.of());
        } finally {
            clearNativeCopies(identifierText);
        }
    }

    /** Creates a Locate request, preserving optional Table 247 values and required Attributes. */
    public static ClientBatchItem locate(
            @Nullable Integer maximumItems,
            @Nullable Integer offsetItems,
            @Nullable Long storageStatusMask,
            @Nullable Long objectGroupMember,
            TtlvStructure attributes) {
        requireStructure(attributes, "Locate Attributes");
        if (storageStatusMask != null) requireU32(storageStatusMask, "Storage Status Mask");
        if (objectGroupMember != null) requireU32(objectGroupMember, "Object Group Member");
        NativeExtensionRegistry.ensureLoaded();
        long value = NativeHandle.withNativeHandles(() ->
                NativeExtensionRegistry.clientBatchItemLocate(
                        maximumItems != null, maximumItems == null ? 0 : maximumItems,
                        offsetItems != null, offsetItems == null ? 0 : offsetItems,
                        storageStatusMask != null, storageStatusMask == null ? 0 : storageStatusMask,
                        objectGroupMember != null, objectGroupMember == null ? 0 : objectGroupMember,
                        handleOrNull(attributes)));
        return new ClientBatchItem(value, List.of());
    }

    /** Creates the empty Get Constraints request from KMIP 2.1 §6.1.22, Table 229. */
    public static ClientBatchItem getConstraints() {
        NativeExtensionRegistry.ensureLoaded();
        long value = NativeExtensionRegistry.clientBatchItemGetConstraints();
        return new ClientBatchItem(value, List.of());
    }

    /** Creates a Set Constraints request from the required Constraints structure. */
    public static ClientBatchItem setConstraints(TtlvStructure constraints) {
        requireStructure(constraints, "Set Constraints");
        NativeExtensionRegistry.ensureLoaded();
        long value = NativeHandle.withNativeHandles(() ->
                NativeExtensionRegistry.clientBatchItemSetConstraints(handleOrNull(constraints)));
        return new ClientBatchItem(value, List.of());
    }

    /** Creates a KMIP 2.1 §6.1.53 Set Defaults request with omitted Defaults Information. */
    public static ClientBatchItem setDefaults() {
        return setDefaults(null);
    }

    /** Creates Set Defaults; null omits Defaults Information and an empty structure is present-empty. */
    public static ClientBatchItem setDefaults(@Nullable TtlvStructure defaultsInformation) {
        if (defaultsInformation != null) requireStructure(defaultsInformation, "Defaults Information");
        NativeExtensionRegistry.ensureLoaded();
        long value = NativeHandle.withNativeHandles(() ->
                NativeExtensionRegistry.clientBatchItemSetDefaults(handleOrNull(defaultsInformation)));
        return new ClientBatchItem(value, List.of());
    }

    /** Creates an Interop item with an open Function Enumeration and required UTF-8 Identifier. */
    public static ClientBatchItem interop(long interopFunction, byte[] interopIdentifier) {
        requireU32(interopFunction, "Interop Function");
        if (interopIdentifier == null) {
            throw new InvalidInputException("Interop Identifier is required");
        }
        requireOperationInputWithinLimit(interopIdentifier);
        byte[] identifierCopy = interopIdentifier.clone();
        try {
            NativeExtensionRegistry.ensureLoaded();
            long value = NativeExtensionRegistry.clientBatchItemInterop(interopFunction, identifierCopy);
            return new ClientBatchItem(value, List.of());
        } finally {
            clearNativeCopies(identifierCopy);
        }
    }

    /**
     * Creates a KMIP 2.1 §6.1.37 PKCS#11 item. Function is required; optional structures and
     * Correlation Value preserve absent versus present-empty input. The native request owns copies.
     */
    public static ClientBatchItem pkcs11(
            TtlvStructure function,
            @Nullable TtlvStructure interfaceStructure,
            @Nullable byte[] correlationValue,
            @Nullable TtlvStructure inputParameters) {
        requireStructure(function, "PKCS#11 Function");
        byte[] correlationCopy = correlationValue == null ? null : correlationValue.clone();
        if (correlationCopy != null) requireOperationInputWithinLimit(correlationCopy);
        try {
            NativeExtensionRegistry.ensureLoaded();
            long value = NativeHandle.withNativeHandles(() ->
                    NativeExtensionRegistry.clientBatchItemPkcs11(
                            function.handle(), handleOrNull(interfaceStructure), correlationCopy,
                            handleOrNull(inputParameters)));
            return new ClientBatchItem(value, List.of());
        } finally {
            clearNativeCopies(correlationCopy);
        }
    }

    /** Creates a §6.1.29 Log item with a required UTF-8 message. */
    public static ClientBatchItem log(byte[] logMessage) {
        if (logMessage == null) {
            throw new InvalidInputException("Log Message is required");
        }
        requireOperationInputWithinLimit(logMessage);
        byte[] messageCopy = logMessage.clone();
        try {
            NativeExtensionRegistry.ensureLoaded();
            long value = NativeExtensionRegistry.clientBatchItemLog(messageCopy);
            return new ClientBatchItem(value, List.of());
        } finally {
            clearNativeCopies(messageCopy);
        }
    }

    /**
     * Creates a KMIP 2.1 §6.1.57 Validate item. The ordered certificate and identifier lists
     * form one chain; a null validity date preserves the server's current-time default.
     */
    public static ClientBatchItem validate(
            List<TtlvStructure> certificates,
            List<UniqueIdentifier> identifiers,
            @Nullable Long validityDate) {
        return validate(certificates, identifiers, validityDate, List.of());
    }

    /**
     * Creates a Validate item with caller-ordered unknown payload fields. Each extension is
     * copied into the request; the caller retains ownership of every supplied TTLV item.
     */
    public static ClientBatchItem validate(
            List<TtlvStructure> certificates,
            List<UniqueIdentifier> identifiers,
            @Nullable Long validityDate,
            List<TtlvItem> payloadExtensions) {
        if (certificates == null || identifiers == null || payloadExtensions == null) {
            throw new InvalidInputException("Validate chain lists are required");
        }
        long totalCount = (long) certificates.size() + identifiers.size() + payloadExtensions.size();
        if (totalCount > MAX_VALIDATE_INPUT_ITEMS) {
            throw new ResourceLimitException("Validate chain entry count exceeds the configured limit");
        }
        List<TtlvStructure> certificateSnapshot = boundedSnapshot(certificates, MAX_VALIDATE_INPUT_ITEMS);
        List<UniqueIdentifier> identifierSnapshot = boundedSnapshot(
                identifiers, MAX_VALIDATE_INPUT_ITEMS - certificateSnapshot.size());
        List<TtlvItem> extensionSnapshot = boundedSnapshot(
                payloadExtensions,
                MAX_VALIDATE_INPUT_ITEMS - certificateSnapshot.size() - identifierSnapshot.size());
        for (TtlvStructure certificate : certificateSnapshot) {
            requireStructure(certificate, "Certificate");
        }
        long totalTextBytes = 0;
        for (UniqueIdentifier identifier : identifierSnapshot) {
            if (identifier == null) {
                throw new InvalidInputException("Validate Unique Identifier is invalid");
            }
            if (identifier.kind() == UniqueIdentifier.Kind.TEXT_STRING) {
                totalTextBytes += utf8Length(identifier.text());
                if (totalTextBytes > MAX_OPERATION_INPUT_BYTES) {
                    throw new ResourceLimitException("Validate identifier bytes exceed the configured limit");
                }
            }
        }
        for (TtlvItem extension : extensionSnapshot) {
            if (extension == null) {
                throw new InvalidInputException("Validate payload extension is invalid");
            }
        }

        int count = identifierSnapshot.size();
        int[] kinds = new int[count];
        byte[][] texts = new byte[count][];
        long[] numbers = new long[count];
        try {
            for (int index = 0; index < count; index++) {
                UniqueIdentifier identifier = identifierSnapshot.get(index);
                kinds[index] = identifier.nativeKind();
                texts[index] = identifier.nativeText();
                numbers[index] = identifier.nativeNumber();
            }
            long[] certificateHandles = new long[certificateSnapshot.size()];
            long[] payloadExtensionHandles = new long[extensionSnapshot.size()];
            long nativeDate = validityDate == null ? 0 : validityDate;
            NativeExtensionRegistry.ensureLoaded();
            long value = NativeHandle.withNativeHandles(() -> {
                for (int index = 0; index < certificateSnapshot.size(); index++) {
                    certificateHandles[index] = certificateSnapshot.get(index).handle();
                }
                for (int index = 0; index < extensionSnapshot.size(); index++) {
                    payloadExtensionHandles[index] = extensionSnapshot.get(index)
                            .nativeHandleOwner().get();
                }
                return NativeExtensionRegistry.clientBatchItemValidate(
                        certificateHandles, kinds, texts, numbers, payloadExtensionHandles,
                        validityDate != null, nativeDate);
            });
            return new ClientBatchItem(value, List.of());
        } finally {
            for (byte[] text : texts) {
                clearNativeCopies(text);
            }
        }
    }

    private static <T> List<T> boundedSnapshot(List<T> values, int maximum) {
        int declaredSize = values.size();
        if (declaredSize > maximum) {
            throw new ResourceLimitException("Validate chain entry count exceeds the configured limit");
        }
        List<T> snapshot = new ArrayList<>(declaredSize);
        for (T value : values) {
            if (snapshot.size() == maximum) {
                throw new ResourceLimitException("Validate chain entry count exceeds the configured limit");
            }
            snapshot.add(value);
        }
        return snapshot;
    }

    /** Creates an Export request, preserving unknown Table 217 selector values. */
    public static ClientBatchItem export(
            @Nullable UniqueIdentifier uniqueIdentifier,
            @Nullable Long keyFormatType,
            @Nullable Long keyWrapType,
            @Nullable Long keyCompressionType,
            @Nullable TtlvStructure keyWrappingSpecification) {
        byte[] identifierText = uniqueIdentifier == null ? null : uniqueIdentifier.nativeText();
        try {
            if (keyFormatType != null) requireU32(keyFormatType, "Key Format Type");
            if (keyWrapType != null) requireU32(keyWrapType, "Key Wrap Type");
            if (keyCompressionType != null) requireU32(keyCompressionType, "Key Compression Type");
            NativeExtensionRegistry.ensureLoaded();
            long value = NativeHandle.withNativeHandles(() ->
                    NativeExtensionRegistry.clientBatchItemExport(
                            identifierKind(uniqueIdentifier), identifierText,
                            identifierNumber(uniqueIdentifier), keyFormatType != null,
                            keyFormatType == null ? 0 : keyFormatType,
                            keyWrapType != null, keyWrapType == null ? 0 : keyWrapType,
                            keyCompressionType != null, keyCompressionType == null ? 0 : keyCompressionType,
                            handleOrNull(keyWrappingSpecification)));
            return new ClientBatchItem(value, List.of());
        } finally {
            clearNativeCopies(identifierText);
        }
    }

    /** Creates a Derive Key request preserving ordered and repeated source identifiers. */
    public static ClientBatchItem deriveKey(
            long objectType,
            List<UniqueIdentifier> uniqueIdentifiers,
            long derivationMethod,
            TtlvStructure derivationParameters,
            TtlvStructure attributes) {
        requireU32(objectType, "Object Type");
        requireU32(derivationMethod, "Derivation Method");
        requireStructure(derivationParameters, "Derivation Parameters");
        requireStructure(attributes, "Derive Key Attributes");
        if (uniqueIdentifiers == null) {
            throw new InvalidInputException("Derive Key source identifiers are required");
        }
        int count = uniqueIdentifiers.size();
        if (count == 0) {
            throw new InvalidInputException("Derive Key requires at least one source identifier");
        }
        if (count > MAX_DERIVE_KEY_SOURCE_IDENTIFIERS) {
            throw new ResourceLimitException("Derive Key source identifier count exceeds the configured limit");
        }

        List<UniqueIdentifier> identifiers;
        try {
            identifiers = List.copyOf(uniqueIdentifiers);
        } catch (NullPointerException exception) {
            throw new InvalidInputException("Derive Key source identifiers contain a null value");
        }
        if (identifiers.size() != count) {
            throw new InvalidInputException("Derive Key source identifier list changed during validation");
        }

        long totalTextBytes = 0;
        for (UniqueIdentifier identifier : identifiers) {
            if (identifier == null) {
                throw new InvalidInputException("Derive Key source identifiers contain a null value"); // COVERAGE-EXCLUDE: List.copyOf above rejects null entries before this immutable loop.
            }
            if (identifier.kind() == UniqueIdentifier.Kind.TEXT_STRING) {
                totalTextBytes += utf8Length(identifier.text());
                if (totalTextBytes > MAX_OPERATION_INPUT_BYTES) {
                    throw new ResourceLimitException("Derive Key source identifiers exceed the configured limit");
                }
            }
        }

        int[] kinds = new int[count];
        byte[][] texts = new byte[count][];
        long[] numbers = new long[count];
        try {
            for (int index = 0; index < count; index++) {
                UniqueIdentifier identifier = identifiers.get(index);
                kinds[index] = identifier.nativeKind();
                texts[index] = identifier.nativeText();
                numbers[index] = identifier.nativeNumber();
            }
            NativeExtensionRegistry.ensureLoaded();
            long value = NativeHandle.withNativeHandles(() -> NativeExtensionRegistry.clientBatchItemDeriveKey(
                    objectType, kinds, texts, numbers, derivationMethod,
                    derivationParameters.handle(), attributes.handle()));
            return new ClientBatchItem(value, List.of());
        } finally {
            for (byte[] text : texts) {
                clearNativeCopies(text);
            }
        }
    }

    /** Creates an Encrypt request without performing local cryptography. */
    public static ClientBatchItem encrypt(
            @Nullable UniqueIdentifier uniqueIdentifier,
            @Nullable TtlvStructure parameters,
            @Nullable TtlvValue data,
            @Nullable byte[] ivCounterNonce,
            @Nullable byte[] correlationValue,
            @Nullable Boolean initIndicator,
            @Nullable Boolean finalIndicator,
            @Nullable byte[] authenticatedEncryptionAdditionalData) {
        byte[] identifierText = uniqueIdentifier == null ? null : uniqueIdentifier.nativeText();
        byte[] ivCopy = copyForNative(ivCounterNonce);
        byte[] correlationCopy = copyForNative(correlationValue);
        byte[] additionalDataCopy = copyForNative(authenticatedEncryptionAdditionalData);
        try {
            NativeExtensionRegistry.ensureLoaded();
            long value = NativeHandle.withNativeHandles(() -> NativeExtensionRegistry.clientBatchItemEncrypt(
                    identifierKind(uniqueIdentifier), identifierText, identifierNumber(uniqueIdentifier),
                    handleOrNull(parameters), handleOrNull(data), ivCopy, correlationCopy,
                    optionalIndicator(initIndicator), optionalIndicator(finalIndicator), additionalDataCopy));
            return new ClientBatchItem(value, List.of());
        } finally {
            clearNativeCopies(identifierText, ivCopy, correlationCopy, additionalDataCopy);
        }
    }

    /** Creates a Decrypt request without performing local cryptography. */
    public static ClientBatchItem decrypt(
            @Nullable UniqueIdentifier uniqueIdentifier,
            @Nullable TtlvStructure parameters,
            @Nullable TtlvValue data,
            @Nullable byte[] ivCounterNonce,
            @Nullable byte[] authenticatedEncryptionTag,
            @Nullable byte[] correlationValue,
            @Nullable Boolean initIndicator,
            @Nullable Boolean finalIndicator,
            @Nullable byte[] authenticatedEncryptionAdditionalData) {
        byte[] identifierText = uniqueIdentifier == null ? null : uniqueIdentifier.nativeText();
        byte[] ivCopy = copyForNative(ivCounterNonce);
        byte[] tagCopy = copyForNative(authenticatedEncryptionTag);
        byte[] correlationCopy = copyForNative(correlationValue);
        byte[] additionalDataCopy = copyForNative(authenticatedEncryptionAdditionalData);
        try {
            NativeExtensionRegistry.ensureLoaded();
            long value = NativeHandle.withNativeHandles(() -> NativeExtensionRegistry.clientBatchItemDecrypt(
                    identifierKind(uniqueIdentifier), identifierText, identifierNumber(uniqueIdentifier),
                    handleOrNull(parameters), handleOrNull(data), ivCopy, tagCopy, correlationCopy,
                    optionalIndicator(initIndicator), optionalIndicator(finalIndicator), additionalDataCopy));
            return new ClientBatchItem(value, List.of());
        } finally {
            clearNativeCopies(identifierText, ivCopy, tagCopy, correlationCopy, additionalDataCopy);
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

    private static int utf8Length(String value) {
        int length = 0;
        for (int index = 0; index < value.length(); index++) {
            char current = value.charAt(index);
            if (current <= 0x7f) {
                length++;
            } else if (current <= 0x7ff) {
                length += 2;
            } else if (Character.isHighSurrogate(current)) {
                length += 4;
                index++;
            } else {
                length += 3;
            }
            if (length > MAX_OPERATION_INPUT_BYTES) {
                return length;
            }
        }
        return length;
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
