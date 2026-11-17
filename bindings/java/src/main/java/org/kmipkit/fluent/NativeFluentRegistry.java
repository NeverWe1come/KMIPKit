package org.kmipkit.fluent;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.extensions.AttributeReference;
import org.kmipkit.extensions.SetAttributeRequest;
import org.kmipkit.extensions.UniqueIdentifier;
import org.jspecify.annotations.Nullable;

/** Package-private JNI bridge so fluent builder handles do not expand public exports. */
final class NativeFluentRegistry {
    static final int CRYPTO_OPERATION_BUILDER = 72;
    static final int CREATE_KEY_PAIR_BUILDER = 73;
    static final int SET_ATTRIBUTE_BUILDER = 74;
    static final int ADD_ATTRIBUTE_BUILDER = 75;
    static final int DELETE_ATTRIBUTE_BUILDER = 76;
    static final int CREATE_BUILDER = 77;
    static final int SYMMETRIC_KEY_BUILDER = 71;

    private NativeFluentRegistry() {}

    static void ensureLoaded() {
        NativeExtensionRegistry.ensureLoaded();
    }

    static native long cryptoOperationBuilderCreate(int operation);

    static native void cryptoOperationBuilderSetUniqueIdentifier(
            long builder, int kind, @Nullable byte[] text, long number);

    static native void cryptoOperationBuilderSetBytes(long builder, int field, byte[] value);

    static native void cryptoOperationBuilderSetDataValue(long builder, long value);

    static native void cryptoOperationBuilderSetCryptographicParameters(
            long builder, long parameters);

    static native void cryptoOperationBuilderSetCryptographicParametersStructure(
            long builder, long parameters);

    static native void cryptoOperationBuilderSetIndicator(long builder, int indicator, boolean value);

    static native long cryptoOperationBuilderBuildBatchItem(long builder);

    static native void symmetricKeyBuilderInsertAttributeItem(
            long builder, int index, long item);

    static native long createKeyPairBuilderCreate();

    static native void createKeyPairBuilderSetAlgorithm(long builder, long algorithm);

    static native void createKeyPairBuilderSetLength(long builder, int length);

    static native void createKeyPairBuilderSetPrivateUsageMask(long builder, int usageMask);

    static native void createKeyPairBuilderSetPublicUsageMask(long builder, int usageMask);

    static native void createKeyPairBuilderSetPrivateName(long builder, String name);

    static native void createKeyPairBuilderSetPublicName(long builder, String name);

    static native void createKeyPairBuilderAddCommonAttributeItem(long builder, long item);

    static native void createKeyPairBuilderAddPrivateAttributeItem(long builder, long item);

    static native void createKeyPairBuilderAddPublicAttributeItem(long builder, long item);

    static native void createKeyPairBuilderAddCommonCataloguedAttribute(
            long builder, int tag, long value);

    static native void createKeyPairBuilderAddPrivateCataloguedAttribute(
            long builder, int tag, long value);

    static native void createKeyPairBuilderAddPublicCataloguedAttribute(
            long builder, int tag, long value);

    static native void createKeyPairBuilderAddCommonVendorAttribute(
            long builder, String vendor, String name, long value);

    static native void createKeyPairBuilderAddPrivateVendorAttribute(
            long builder, String vendor, String name, long value);

    static native void createKeyPairBuilderAddPublicVendorAttribute(
            long builder, String vendor, String name, long value);

    static native void createKeyPairBuilderValidate(long builder);

    static native long createKeyPairBuilderBuildBatchItem(long builder);

    static native long setAttributeBuilderCreate();

    static native void setAttributeBuilderSetUniqueIdentifier(long builder, String value);

    static native void setAttributeBuilderAddAttributeItem(long builder, long item);

    static native void setAttributeBuilderAddCataloguedAttribute(
            long builder, int tag, long value);

    static native void setAttributeBuilderAddVendorAttribute(
            long builder, String vendor, String name, long value);

    static native void setAttributeBuilderSetAsynchronousIndicator(long builder, long value);

    static native void setAttributeBuilderValidate(long builder);

    static native SetAttributeRequest setAttributeBuilderBuildRequest(
            long builder, @Nullable UniqueIdentifier identifier);

    static native long setAttributeBuilderBuildBatchItem(long builder);

    static native long addAttributeBuilderCreate();

    static native void addAttributeBuilderSetUniqueIdentifier(long builder, String value);

    static native void addAttributeBuilderAddAttributeItem(long builder, long item);

    static native void addAttributeBuilderAddCataloguedAttribute(
            long builder, int tag, long value);

    static native void addAttributeBuilderAddVendorAttribute(
            long builder, String vendor, String name, long value);

    static native void addAttributeBuilderSetAsynchronousIndicator(long builder, long value);

    static native void addAttributeBuilderValidate(long builder);

    static native org.kmipkit.extensions.AddAttributeRequest addAttributeBuilderBuildRequest(
            long builder, @Nullable UniqueIdentifier identifier);

    static native long addAttributeBuilderBuildBatchItem(long builder);

    static native long deleteAttributeBuilderCreate();

    static native void deleteAttributeBuilderSetUniqueIdentifier(long builder, String value);

    static native void deleteAttributeBuilderSetCurrentAttributeItem(long builder, long item);

    static native void deleteAttributeBuilderSetCataloguedAttribute(long builder, int tag, long value);

    static native void deleteAttributeBuilderSetVendorAttribute(
            long builder, String vendor, String name, long value);

    static native void deleteAttributeBuilderSetAttributeReference(
            long builder, int kind, long tag, @Nullable byte[] vendor, @Nullable byte[] name);

    static native void deleteAttributeBuilderSetAsynchronousIndicator(long builder, long value);

    static native void deleteAttributeBuilderValidate(long builder);

    static native org.kmipkit.extensions.DeleteAttributeRequest deleteAttributeBuilderBuildRequest(
            long builder, @Nullable UniqueIdentifier identifier,
            @Nullable AttributeReference reference);

    static native long deleteAttributeBuilderBuildBatchItem(long builder);

    static native long createBuilderCreate();

    static native void createBuilderSetObjectType(long builder, long objectType);

    static native void createBuilderSetAttributes(long builder, long attributes);

    static native void createBuilderSetProtectionStorageMasks(long builder, long masks);

    static native void createBuilderSetAsynchronousIndicator(long builder, long value);

    static native void createBuilderValidate(long builder);

    static native long createBuilderBuildBatchItem(long builder);
}
