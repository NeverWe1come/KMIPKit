package org.kmipkit.ttlv;

import java.util.List;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;

/** Owned, validated KMIP Key Block with a lossless generic TTLV view. */
public final class KeyBlock extends ManagedTtlvValue {
    private KeyBlock(long nativeHandle) { super(nativeHandle, 2, 0); }

    /** Copies and validates a Key Block without consuming its source Item. */
    public static KeyBlock fromItem(TtlvItem item) {
        if (item == null) throw new InvalidInputException("Key Block input is invalid");
        NativeExtensionRegistry.ensureLoaded();
        return item.nativeHandleOwner().withValue(
                value -> new KeyBlock(NativeExtensionRegistry.keyBlockFromItem(value)));
    }

    /** Copies caller-owned Key Block fields and applies the KMIP table order. */
    public static KeyBlock fromFields(List<TtlvItem> fields) {
        NativeExtensionRegistry.ensureLoaded();
        return new KeyBlock(withFields(fields, NativeExtensionRegistry::keyBlockFromFields));
    }

    /** Returns the source-preserving raw Key Format Type. */
    public long keyFormatType() { return readEnumerationField(0x00420042); }

    /** Returns the source-preserving Key Compression Type when present. */
    public long keyCompressionType() { return readEnumerationField(0x00420041); }

    /** Returns the source-preserving Cryptographic Algorithm when present. */
    public long cryptographicAlgorithm() { return readEnumerationField(0x00420028); }

    /** Returns the Cryptographic Length when present. */
    public int cryptographicLength() { return readIntegerField(0x0042002a); }

    /** Returns the plaintext Structure or wrapped ByteString Key Value type. */
    public TtlvItemType keyValueType() { return fieldType(0x00420045); }

    /** Reports whether this Key Block contains Key Wrapping Data. */
    public boolean hasKeyWrappingData() { return hasField(0x00420046); }

}
