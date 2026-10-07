package org.kmipkit;

import org.kmipkit.ttlv.CodecLimits;

/**
 * Internal JNI bridge for the KMIPKit registry API.
 *
 * <p>Applications should use the typed classes in {@code org.kmipkit.extensions}
 * and {@code org.kmipkit.ttlv}. The native library is loaded lazily on the first
 * bridge call. Set {@code -Dkmipkit.nativeLibrary=/absolute/path/to/library}
 * to load a specific JNI library; otherwise {@code kmipkit_jni} is used.</p>
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
                String explicitLibrary = System.getProperty("kmipkit.nativeLibrary");
                if (explicitLibrary == null || explicitLibrary.isBlank()) {
                    System.loadLibrary("kmipkit_jni");
                } else {
                    System.load(explicitLibrary);
                }
                loaded = true;
            }
        }
    }

    public static long[] codecLimitValues(CodecLimits limits) {
        return new long[] {
                limits.maxMessageBytes(), limits.maxStructureDepth(), limits.maxElements()
        };
    }

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
    public static native long clientBatchItemExtensionCount(long item);
    public static native long clientBatchItemExtensionIdentityAt(long item, long index);
    public static native boolean clientBatchItemExtensionCriticalityIndicatorAt(long item, long index);
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
