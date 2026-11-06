package org.kmipkit.ttlv;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import java.util.List;

/** Owned, validated plaintext KMIP Key Value with a generic TTLV view. */
public final class KeyValue extends ManagedTtlvValue {
    private final long formatRaw;

    private KeyValue(long nativeHandle, long formatRaw) {
        super(nativeHandle, 3, (int) formatRaw);
        this.formatRaw = formatRaw;
    }

    /** Copies and validates a plaintext Key Value for the supplied open format. */
    public static KeyValue fromItem(long formatRaw, TtlvItem item) {
        if (formatRaw < 0 || formatRaw > 0xffff_ffffL || item == null) {
            throw new InvalidInputException("Key Value input is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        return item.nativeHandleOwner().withValue(value -> new KeyValue(
                NativeExtensionRegistry.keyValueFromItem((int) formatRaw, value), formatRaw));
    }

    /** Copies caller-owned Key Value fields for the supplied open key format. */
    public static KeyValue fromFields(long formatRaw, List<TtlvItem> fields) {
        if (formatRaw < 0 || formatRaw > 0xffff_ffffL) {
            throw new InvalidInputException("Key Value format is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        return new KeyValue(withFields(fields, values ->
                NativeExtensionRegistry.keyValueFromFields((int) formatRaw, values)), formatRaw);
    }

    /** Returns the source-preserving Key Format Type raw value. */
    public long formatRaw() { return formatRaw; }

    /** Returns the TTLV type used by Key Material for the explicit format. */
    public TtlvItemType keyMaterialType() { return fieldType(0x00420043); }

    /** Reports whether this Key Value contains optional Attributes. */
    public boolean hasAttributes() { return hasField(0x00420125); }

}
