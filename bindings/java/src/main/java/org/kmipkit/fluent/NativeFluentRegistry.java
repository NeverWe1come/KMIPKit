package org.kmipkit.fluent;

import org.kmipkit.NativeExtensionRegistry;
import org.jspecify.annotations.Nullable;

/** Package-private JNI bridge so fluent builder handles do not expand public exports. */
final class NativeFluentRegistry {
    static final int CRYPTO_OPERATION_BUILDER = 72;
    static final int CREATE_KEY_PAIR_BUILDER = 73;
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
}
