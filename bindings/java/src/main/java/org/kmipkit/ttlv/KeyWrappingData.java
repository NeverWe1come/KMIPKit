package org.kmipkit.ttlv;

import java.util.List;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;

/** Owned, validated KMIP Key Wrapping Data with a lossless generic TTLV view. */
public final class KeyWrappingData extends ManagedTtlvValue {
    private KeyWrappingData(long nativeHandle) { super(nativeHandle, 5, 0); }

    /** Copies and validates Key Wrapping Data without consuming the source Item. */
    public static KeyWrappingData fromItem(TtlvItem item) {
        if (item == null) throw new InvalidInputException("Key Wrapping Data input is invalid");
        NativeExtensionRegistry.ensureLoaded();
        return item.nativeHandleOwner().withValue(
                value -> new KeyWrappingData(NativeExtensionRegistry.keyWrappingDataFromItem(value)));
    }

    /** Copies caller-owned Key Wrapping Data fields and applies table order. */
    public static KeyWrappingData fromFields(List<TtlvItem> fields) {
        NativeExtensionRegistry.ensureLoaded();
        return new KeyWrappingData(withFields(fields, NativeExtensionRegistry::keyWrappingDataFromFields));
    }

    /** Returns the source-preserving raw Wrapping Method. */
    public long wrappingMethod() { return readEnumerationField(0x0042009e); }

    /** Reports whether Encryption Key Information is present. */
    public boolean hasEncryptionKeyInformation() { return hasField(0x00420036); }

    /** Returns an owned raw Encryption Key Information Structure when present. */
    public TtlvItem encryptionKeyInformationItem() { return fieldItem(0x00420036); }

    /** Reports whether MAC/Signature Key Information is present. */
    public boolean hasMacSignatureKeyInformation() { return hasField(0x0042004e); }

    /** Returns an owned raw MAC/Signature Key Information Structure when present. */
    public TtlvItem macSignatureKeyInformationItem() { return fieldItem(0x0042004e); }

    /** Returns a caller-owned copy of optional MAC/Signature bytes. */
    public byte[] macSignature() { return readBytesField(0x0042004d); }

    /** Returns a caller-owned copy of optional IV/Counter/Nonce bytes. */
    public byte[] ivCounterNonce() { return readBytesField(0x0042003d); }

    /** Reads the raw optional Encoding Option. */
    public long encodingOption() { return readEnumerationField(0x004200a3); }

}
