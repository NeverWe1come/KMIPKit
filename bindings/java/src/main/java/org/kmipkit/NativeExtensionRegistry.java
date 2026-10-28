package org.kmipkit;

import org.jspecify.annotations.Nullable;
import org.kmipkit.ttlv.CodecLimits;

/**
 * Internal JNI bridge for the KMIPKit registry API.
 *
 * <p>Applications should use the typed classes in {@code org.kmipkit.extensions}
 * and {@code org.kmipkit.ttlv}. The native library is loaded lazily on the first
 * bridge call. The packaged native library is selected and verified from the
 * current platform resources. Set {@code -Dkmipkit.nativeLibrary=/absolute/path/to/library}
 * to load an administrator-provided library instead.</p>
 */
public final class NativeExtensionRegistry {
    public static final int IDENTITY = 1;
    public static final int COMPATIBILITY = 2;
    public static final int PATH = 3;
    public static final int DISCRIMINATOR = 4;
    public static final int SCHEMA = 5;
    public static final int INFORMATION = 6;
    public static final int DEFINITION = 7;
    public static final int REGISTRY = 8;
    public static final int CONFIGURATION = 9;
    public static final int VALIDATED_VALUE = 10;
    public static final int REGISTERED_VALUE = 11;
    public static final int REQUEST_EXTENSION = 12;
    public static final int BATCH_ITEM = 13;
    public static final int TTLV_VALUE = 14;
    public static final int TTLV_STRUCTURE = 15;
    public static final int TTLV_ITEM = 16;
    public static final int TTLV_STRUCTURE_VIEW = 17;
    public static final int TTLV_ITEM_VIEW = 18;
    public static final int TTLV_VALUE_VIEW = 19;
    public static final int TTLV_PATH = 20;
    public static final int RAW_TAG = 21;
    public static final int TAG = 22;
    public static final int CHILD_RULE = 23;
    public static final int ORDER_CONSTRAINT = 24;
    public static final int RECOGNITION = 25;
    public static final int CODEC_LIMITS = 26;
    public static final int TRANSPORT_CONFIGURATION = 27;
    public static final int CLIENT = 28;
    public static final int CLIENT_BATCH = 29;
    public static final int CLIENT_BATCH_RESPONSE = 30;
    public static final int CLIENT_BATCH_ITEM_RESPONSE = 31;
    public static final int REGISTER_REQUEST = 32;
    public static final int SPLIT_KEY_REFERENCE = 33;
    public static final int USAGE_ALLOCATION_CONTEXT = 34;
    public static final int REVOCATION_REASON = 35;
    public static final int REVOKE_REQUEST = 36;

    private static volatile boolean loaded;

    static {
        ensureLoaded();
    }

    private NativeExtensionRegistry() {}

    public static void ensureLoaded() {
        if (loaded) {
            return;
        }
        synchronized (NativeExtensionRegistry.class) {
            if (!loaded) {
                NativeLibraryLoader.load();
                loaded = true;
            }
        }
    }

    public static long[] codecLimitValues(CodecLimits limits) {
        return new long[] {
                limits.maxMessageBytes(), limits.maxStructureDepth(), limits.maxElements()
        };
    }

    static native int nativeAbiVersion();

    public static native void release(int handleKind, long handle);

    public static native long extensionIdentityCreate(String vendorIdentifier, String name, String version);
    public static native long compatibilityCreate(int kmipMinMajor, int kmipMinMinor,
            int kmipMaxMajor, int kmipMaxMinor, String kmipkitMinimum, String kmipkitMaximum);
    public static native long ttlvPathCreate(long firstTag);
    public static native long ttlvPathWithChildTag(long path, long tag);
    public static native long discriminatorCreate(long path, long scalarValue);
    public static native long extensionSchemaScalar(int itemType);
    public static native long extensionSchemaStructure(long[] children, long[] orderConstraints,
            boolean preserveUndeclaredChildren);
    public static native long extensionChildRuleRequired(long tag, long schema);
    public static native long extensionChildRuleOptional(long tag, long schema);
    public static native long extensionChildRuleRepeated(long tag, long schema);
    public static native long extensionOrderConstraintCreate(long beforeTag, long afterTag);
    public static native long extensionSchemaMinimumLength(long schema, long value);
    public static native long extensionSchemaMaximumLength(long schema, long value);
    public static native long extensionSchemaSignedRange(long schema, long minimum, long maximum);
    public static native long extensionSchemaUnsignedRange(long schema, long minimum, long maximum);
    public static native long extensionSchemaAllowedEnumeration(long schema, int value);
    public static native long extensionSchemaAllowedBitMask(long schema, int value);
    public static native long extensionSchemaRequiredBitMask(long schema, int value);
    public static native long extensionDefinitionCreate(long identity, long compatibility,
            long discriminator, long schema);
    public static native long extensionDefinitionValidate(long definition, long value, long[] codecLimits);
    public static native long clientConfigurationCreate(long registry);
    public static native long clientConfigurationExtensionRegistry(long configuration);
    public static native long clientExtensionRegistryCreate(long[] definitions, long[] limits);
    public static native long clientExtensionRegistryInspect(long registry, String vendorIdentifier,
            long value, long[] codecLimits);
    public static native long clientExtensionRegistryDefinitionCount(long registry);
    public static native long clientExtensionRegistryDefinitionAt(long registry, long index);
    public static native long clientExtensionRegistryValidate(long registry, long identity,
            long value, long[] codecLimits);
    public static native long clientRequestMessageExtensionCreate(long registeredValue,
            boolean criticalityIndicator);
    public static native long clientBatchItemWithExtension(long item, long requestExtension);
    public static native long clientBatchItemDiscoverVersions();
    public static native long clientBatchItemPing();
    public static native long clientBatchItemQuery(int[] functions, @Nullable byte[][] objectGroups);
    public static native long clientBatchItemQueryWithLimits(
            int[] functions, @Nullable byte[][] objectGroups, long[] codecLimits);
    public static native long clientBatchItemCreate(long objectType, long attributes, long protectionStorageMasks);
    public static native long clientBatchItemCreateKeyPair(long commonAttributes, long privateKeyAttributes,
            long publicKeyAttributes, long commonProtectionStorageMasks,
            long privateProtectionStorageMasks, long publicProtectionStorageMasks);
    public static native long clientBatchItemCreateSplitKey(long objectType, int parts, int threshold,
            long method, int identifierKind, byte[] identifierText, long identifierNumber,
            byte[] primeFieldSize, long attributes, long protectionStorageMasks);
    public static native long clientBatchItemCertify(int identifierKind, @Nullable byte[] identifierText,
            long identifierNumber, boolean hasCertificateRequestType, long certificateRequestType,
            @Nullable byte[] certificateRequestValue, long attributes, long protectionStorageMasks);
    public static native long clientBatchItemRecertify(int identifierKind,
            @Nullable byte[] identifierText, long identifierNumber,
            int certificateRequestIdentifierKind, @Nullable byte[] certificateRequestIdentifierText,
            long certificateRequestIdentifierNumber, boolean hasCertificateRequestType,
            long certificateRequestType, @Nullable byte[] certificateRequestValue,
            boolean hasOffset, long offset, long attributes, long protectionStorageMasks);
    public static native long clientBatchItemRekey(int identifierKind,
            @Nullable byte[] identifierText, long identifierNumber,
            boolean hasOffset, long offset, long attributes, long protectionStorageMasks);
    public static native long clientBatchItemRekeyKeyPair(int identifierKind,
            @Nullable byte[] identifierText, long identifierNumber, boolean hasOffset, long offset,
            long commonAttributes, long privateKeyAttributes, long publicKeyAttributes,
            long commonProtectionStorageMasks, long privateProtectionStorageMasks,
            long publicProtectionStorageMasks);
    public static native long clientBatchItemReprovision(long certificateRequest, long certificate);
    public static native long clientBatchItemRngRetrieve(int dataLength);
    public static native long clientBatchItemRngSeed(@Nullable byte[] data);
    public static native long clientBatchItemCheck(int identifierKind, @Nullable byte[] identifierText,
            long identifierNumber, boolean hasUsageLimitsCount, long usageLimitsCount,
            boolean hasCryptographicUsageMask, int cryptographicUsageMask,
            boolean hasLeaseTime, long leaseTime);
    public static native long clientBatchItemDelegatedLogin(long rights, int leaseTimeKind,
            long leaseTimeValue, boolean hasRequestCount, int requestCount, long usageLimits);
    public static native long clientBatchItemLogin(int leaseTimeKind, long leaseTimeValue,
            boolean hasRequestCount, int requestCount, long usageLimits);
    public static native long clientBatchItemLogout(long ticket);
    public static native long clientBatchItemGetUsageAllocation(int identifierKind,
            @Nullable byte[] identifierText, long identifierNumber, long usageLimitsCount);
    public static native long clientBatchItemActivate(int identifierKind,
            @Nullable byte[] identifierText, long identifierNumber);
    public static native long clientBatchItemArchive(int identifierKind,
            @Nullable byte[] identifierText, long identifierNumber);
    public static native long clientBatchItemDestroy(int identifierKind,
            @Nullable byte[] identifierText, long identifierNumber);
    public static native long clientBatchItemRecover(int identifierKind,
            @Nullable byte[] identifierText, long identifierNumber);
    public static native long clientBatchItemObtainLease(int identifierKind,
            @Nullable byte[] identifierText, long identifierNumber);
    public static native long clientBatchItemImport(int identifierKind, byte[] identifierText,
            long identifierNumber, long objectType, boolean hasReplaceExisting,
            boolean replaceExisting, boolean hasKeyWrapType, long keyWrapType,
            long attributes, long anyObject);
    public static native long splitKeyReferenceCreate(int identifierKind, byte[] identifierText,
            long identifierNumber, long splitKeyThreshold);
    public static native long clientBatchItemJoinSplitKey(long objectType, long[] references,
            boolean hasSecretDataType, long secretDataType, long attributes,
            long protectionStorageMasks);
    public static native long clientBatchItemGet(int identifierKind,
            @Nullable byte[] identifierText, long identifierNumber,
            boolean hasKeyFormatType, long keyFormatType,
            boolean hasKeyWrapType, long keyWrapType,
            boolean hasKeyCompressionType, long keyCompressionType,
            long keyWrappingSpecification);
    public static native long clientBatchItemLocate(
            boolean hasMaximumItems, int maximumItems,
            boolean hasOffsetItems, int offsetItems,
            boolean hasStorageStatusMask, long storageStatusMask,
            boolean hasObjectGroupMember, long objectGroupMember,
            long attributes);
    public static native long clientBatchItemGetConstraints();
    public static native long clientBatchItemGetAttributeList(
            int identifierKind, @Nullable byte[] identifierText, long identifierNumber);
    public static native long clientBatchItemGetAttributes(int identifierKind,
            @Nullable byte[] identifierText, long identifierNumber, int[] referenceKinds,
            long[] referenceTags, byte[][] vendorIdentifications, byte[][] attributeNames);
    public static native long clientBatchItemAddAttribute(
            int identifierKind, @Nullable byte[] identifierText, long identifierNumber,
            long newAttribute);
    public static native long clientBatchItemAdjustAttribute(
            int identifierKind, @Nullable byte[] identifierText, long identifierNumber,
            int referenceKind, long referenceTag, @Nullable byte[] vendorIdentification,
            @Nullable byte[] attributeName, long adjustmentType, long adjustmentValue);
    public static native long clientBatchItemDeleteAttribute(
            int identifierKind, @Nullable byte[] identifierText, long identifierNumber,
            long currentAttribute, int referenceKind, long referenceTag,
            @Nullable byte[] vendorIdentification, @Nullable byte[] attributeName);
    public static native long clientBatchItemModifyAttribute(
            int identifierKind, @Nullable byte[] identifierText, long identifierNumber,
            long currentAttribute, long newAttribute);
    public static native long clientBatchItemSetAttribute(
            int identifierKind, @Nullable byte[] identifierText, long identifierNumber,
            long newAttribute);
    public static native long clientBatchItemSetConstraints(long constraints);
    public static native long clientBatchItemSetDefaults(long defaultsInformation);
    public static native long clientBatchItemInterop(long interopFunction, byte[] interopIdentifier);
    public static native long clientBatchItemPkcs11(
            long function, long interfaceStructure, byte[] correlationValue, long inputParameters);
    public static native long clientBatchItemLog(byte[] logMessage);
    public static native long clientBatchItemValidate(long[] certificates,
            int[] identifierKinds, byte[][] identifierTexts, long[] identifierNumbers,
            long[] payloadExtensions,
            boolean hasValidityDate, long validityDate);
    public static native long clientBatchItemExport(int identifierKind,
            @Nullable byte[] identifierText, long identifierNumber,
            boolean hasKeyFormatType, long keyFormatType,
            boolean hasKeyWrapType, long keyWrapType,
            boolean hasKeyCompressionType, long keyCompressionType,
            long keyWrappingSpecification);
    public static native long clientBatchItemDeriveKey(long objectType, int[] identifierKinds,
            byte[][] identifierTexts, long[] identifierNumbers, long derivationMethod,
            long derivationParameters, long attributes);
    public static native long clientBatchItemEncrypt(
            int identifierKind, @Nullable byte[] identifierText, long identifierNumber,
            long parameters, long data, @Nullable byte[] ivCounterNonce,
            @Nullable byte[] correlationValue, int initIndicator, int finalIndicator,
            @Nullable byte[] authenticatedEncryptionAdditionalData);
    public static native long clientBatchItemDecrypt(
            int identifierKind, @Nullable byte[] identifierText, long identifierNumber,
            long parameters, long data, @Nullable byte[] ivCounterNonce,
            @Nullable byte[] authenticatedEncryptionTag, @Nullable byte[] correlationValue,
            int initIndicator, int finalIndicator,
            @Nullable byte[] authenticatedEncryptionAdditionalData);
    public static native long clientBatchItemHash(
            long parameters, long data, @Nullable byte[] correlationValue,
            int initIndicator, int finalIndicator);
    public static native long clientBatchItemMac(
            int identifierKind, @Nullable byte[] identifierText, long identifierNumber,
            long parameters, long data, @Nullable byte[] correlationValue,
            int initIndicator, int finalIndicator);
    public static native long clientBatchItemMacVerify(
            int identifierKind, @Nullable byte[] identifierText, long identifierNumber,
            long parameters, long data, @Nullable byte[] macData, @Nullable byte[] correlationValue,
            int initIndicator, int finalIndicator);
    public static native long clientBatchItemSign(
            int identifierKind, @Nullable byte[] identifierText, long identifierNumber,
            long parameters, long data, @Nullable byte[] digestedData, @Nullable byte[] correlationValue,
            int initIndicator, int finalIndicator);
    public static native long clientBatchItemSignatureVerify(
            int identifierKind, @Nullable byte[] identifierText, long identifierNumber,
            long parameters, long data, @Nullable byte[] digestedData, @Nullable byte[] signatureData,
            @Nullable byte[] correlationValue, int initIndicator, int finalIndicator);
    public static native long registerRequestCreate(int objectType, long attributes, long object);
    public static native long registerRequestWithOpaqueObjectContext(long request, int context);
    public static native long clientBatchItemRegister(long request);
    public static native long revocationReasonCreate(long rawCode, @Nullable byte[] message);
    public static native long revokeRequestCreate(int identifierKind,
            @Nullable byte[] identifierText, long identifierNumber, long reason,
            boolean hasCompromiseOccurrenceDate, long compromiseOccurrenceDate);
    public static native long clientBatchItemRevoke(long request);
    public static native long clientBatchItemWithUniqueBatchItemId(long item, byte[] id);
    public static native long clientBatchItemExtensionCount(long item);
    public static native long clientBatchItemExtensionIdentityAt(long item, long index);
    public static native boolean clientBatchItemExtensionCriticalityIndicatorAt(long item, long index);
    public static native long transportConfigCreate(int endpointKind, String endpoint, int endpointPort,
            int trustKind, int[] clientCertificateSources, int[] clientCertificateEncodings,
            byte[][] clientCertificates, int privateKeySource, int privateKeyEncoding,
            byte[] privateKey, int[] trustCertificateSources, int[] trustCertificateEncodings,
            byte[][] trustCertificates, int[] revocationListSources, int[] revocationListEncodings,
            byte[][] revocationLists, String tlsServerName, String targetUri, long[] timeouts,
            long maxRequestBytes);
    public static native long clientCreate(long extensionConfiguration, long transportConfiguration);
    public static native long clientBatchCreate();
    public static native void clientBatchAddItem(long batch, long item);
    public static native void clientBatchSetAsynchronousIndicator(long batch, long indicator);
    public static native void clientBatchSetRequestTimestamp(long batch, long timestamp);
    public static native void clientBatchSetOrderOption(long batch, boolean enabled);
    public static native void clientBatchAddErrorContinuationValue(long batch, long value);
    public static native void clientBatchSetClientCorrelationValue(long batch, byte[] value);
    public static native long clientExecute(long client, long batch, long[] codecLimits);
    public static native long clientExecuteWithOptions(
            long client, long batch, long[] codecLimits, long[] requestTimeouts);
    public static native long usageAllocationContextCreate(
            long itemResponse, long requestedUnits, long usageLimitsUnit);
    public static native long usageAllocationContextUsageLimitsUnit(long context);
    public static native long usageAllocationContextAllocatedUnits(long context);
    public static native long usageAllocationContextAvailableUnits(long context);
    public static native long usageAllocationContextReservedUnits(long context);
    public static native Object[] usageAllocationContextUniqueIdentifier(long context);
    public static native long clientExecuteGuardedProtectionItem(
            long client, long context, long item, long cost, long[] codecLimits);
    public static native long clientExecuteGuardedProtectionItemWithAsynchronousIndicator(
            long client, long context, long item, long cost, long asynchronousIndicator,
            long[] codecLimits);
    public static native long clientBatchResponseItemCount(long response);
    public static native long clientBatchResponseItemAt(long response, long index);
    public static native long[] clientBatchItemResponseMetadata(long itemResponse);
    public static native Object[] clientBatchItemResponseMetadataV2(long itemResponse);
    public static native @Nullable byte[] clientBatchItemResponseUniqueBatchItemId(long itemResponse);
    public static native @Nullable byte[] clientBatchItemResponsePendingCorrelationValue(long itemResponse);
    public static native @Nullable long[] clientBatchItemResponseDiscoverVersionsSupportedVersions(
            long itemResponse);
    public static native Object[] clientBatchItemResponseCreate(long itemResponse);
    public static native Object[] clientBatchItemResponseCertify(long itemResponse);
    public static native Object[] clientBatchItemResponseRecertify(long itemResponse);
    public static native Object[] clientBatchItemResponseRekey(long itemResponse);
    public static native Object[] clientBatchItemResponseRekeyKeyPair(long itemResponse);
    public static native Object[] clientBatchItemResponseReprovision(long itemResponse);
    public static native Object[] clientBatchItemResponseReprovisionTtlv(long itemResponse);
    public static native Object[] clientBatchItemResponseRegister(long itemResponse);
    public static native Object[] clientBatchItemResponseRevoke(long itemResponse);
    public static native Object[] clientBatchItemResponseRngRetrieve(long itemResponse);
    public static native Object[] clientBatchItemResponseRngSeed(long itemResponse);
    public static native Object[] clientBatchItemResponseActivate(long itemResponse);
    public static native Object[] clientBatchItemResponseArchive(long itemResponse);
    public static native Object[] clientBatchItemResponseDestroy(long itemResponse);
    public static native Object[] clientBatchItemResponseRecover(long itemResponse);
    public static native Object[] clientBatchItemResponseCheck(long itemResponse);
    public static native Object[] clientBatchItemResponseDelegatedLogin(long itemResponse);
    public static native Object[] clientBatchItemResponseLogin(long itemResponse);
    public static native long[] clientBatchItemResponseLogout(long itemResponse);
    public static native Object[] clientBatchItemResponseGetUsageAllocation(long itemResponse);
    public static native Object[] clientBatchItemResponseObtainLease(long itemResponse);
    public static native Object[] clientBatchItemResponseImport(long itemResponse);
    public static native Object[] clientBatchItemResponseJoinSplitKey(long itemResponse);
    public static native Object[] clientBatchItemResponseGet(long itemResponse);
    public static native Object[] clientBatchItemResponseLocate(long itemResponse);
    public static native Object[] clientBatchItemResponseGetConstraints(long itemResponse);
    public static native Object[] clientBatchItemResponseGetAttributeList(
            long itemResponse, long maxElements, long maxMessageBytes);
    public static native Object[] clientBatchItemResponseGetAttributes(long itemResponse);
    public static native Object[] clientBatchItemResponseAddAttribute(long itemResponse);
    public static native Object[] clientBatchItemResponseAdjustAttribute(long itemResponse);
    public static native Object[] clientBatchItemResponseDeleteAttribute(long itemResponse);
    public static native Object[] clientBatchItemResponseModifyAttribute(long itemResponse);
    public static native Object[] clientBatchItemResponseSetAttribute(long itemResponse);
    public static native Object[] clientBatchItemResponseSetConstraints(long itemResponse);
    public static native Object[] clientBatchItemResponseSetDefaults(long itemResponse);
    public static native Object[] clientBatchItemResponseValidate(long itemResponse);
    public static native long[] clientBatchItemResponsePing(long itemResponse);
    public static native Object[] clientBatchItemResponseQueryTtlv(long itemResponse);
    public static native Object[] clientBatchItemResponseInterop(long itemResponse);
    public static native Object[] clientBatchItemResponsePkcs11(long itemResponse);
    public static native Object[] clientBatchItemResponseLog(long itemResponse);
    public static native Object[] clientBatchItemResponseExport(long itemResponse);
    public static native Object[] clientBatchItemResponseDeriveKey(long itemResponse);
    public static native Object[] clientBatchItemResponseCreateKeyPair(long itemResponse);
    public static native Object[] clientBatchItemResponseCreateSplitKey(long itemResponse);
    public static native Object[] clientBatchItemResponseHash(long itemResponse);
    public static native Object[] clientBatchItemResponseMac(long itemResponse);
    public static native Object[] clientBatchItemResponseMacVerify(long itemResponse);
    public static native Object[] clientBatchItemResponseSign(long itemResponse);
    public static native Object[] clientBatchItemResponseSignatureVerify(long itemResponse);
    public static native Object[] clientBatchItemResponseDecrypt(long itemResponse);
    public static native Object[] clientBatchItemResponseEncrypt(long itemResponse);
    public static native long extensionInformationCreate(String extensionName);
    public static native long extensionInformationWithTag(long information, int tag);
    public static native long extensionInformationWithType(long information, int type);
    public static native long extensionInformationWithEnumeration(long information, int enumeration);
    public static native long extensionInformationWithAttribute(long information, boolean attribute);
    public static native long extensionInformationWithParentStructureTag(long information, int tag);
    public static native long extensionInformationWithDescription(long information, String description);
    public static native long extensionInformationToTtlv(long information);
    public static native long extensionDefinitionWithInformation(long definition, long information);
    public static native long extensionDefinitionInformation(long definition);
    public static native long clientExtensionRegistryDefinitionForIdentity(long registry, long identity);
    public static native boolean extensionRecognitionIsRecognized(long recognition);
    public static native long extensionRecognitionValidatedValue(long recognition);
    public static native long extensionRecognitionGenericValue(long recognition);
    public static native long validatedExtensionValueIdentity(long value);
    public static native long validatedExtensionValueGenericValue(long value);
    public static native long validatedExtensionValueValueAt(long value, long path);
    public static native String extensionIdentityVendorIdentifier(long identity);
    public static native String extensionIdentityName(long identity);
    public static native String extensionIdentityVersion(long identity);
    public static native long rawTagCreate(int raw);
    public static native int rawTagValue(long rawTag);
    public static native long rawTagChecked(long rawTag);
    public static native int tagValue(long tag);
    public static native long ttlvValueInteger(int value);
    public static native long ttlvValueLongInteger(long value);
    public static native long ttlvValueEnumeration(long value);
    public static native long ttlvValueBoolean(boolean value);
    public static native long ttlvValueDateTime(long value);
    public static native long ttlvValueInterval(long value);
    public static native long ttlvValueDateTimeExtended(long value);
    public static native long ttlvValueBigInteger(byte[] value, long[] codecLimits);
    public static native long ttlvValueTextString(byte[] value, long[] codecLimits);
    public static native long ttlvValueByteString(byte[] value, long[] codecLimits);
    public static native long ttlvValueStructure(long structure, long[] codecLimits);
    public static native long ttlvStructureCreate();
    public static native long ttlvItemCreate(long tag, long value, long[] codecLimits);
    public static native long ttlvStructureWithItem(long structure, long item, long[] codecLimits);
    public static native long ttlvStructureView(long structure);
    public static native long ttlvStructureViewItemAt(long structureView, long index);
    public static native long ttlvItemViewValue(long itemView);
    public static native long ttlvValueView(long value);
    public static native long ttlvValueViewStructure(long valueView);
    public static native int ttlvItemViewTag(long itemView);
    public static native int ttlvItemViewType(long itemView);
    public static native int ttlvValueViewType(long valueView);
    public static native int ttlvValueViewInteger(long valueView);
    public static native long ttlvValueViewLongInteger(long valueView);
    public static native long ttlvValueViewEnumeration(long valueView);
    public static native boolean ttlvValueViewBoolean(long valueView);
    public static native long ttlvValueViewDateTime(long valueView);
    public static native long ttlvValueViewInterval(long valueView);
    public static native long ttlvValueViewDateTimeExtended(long valueView);
    public static native long ttlvValueViewByteLength(long valueView);
    public static native short ttlvValueViewByteAt(long valueView, long index);
    public static native long ttlvStructureViewItemCount(long structureView);
    public static native long codecLimitsCreate(long maxMessageBytes, long maxStructureDepth, long maxElements);
}
