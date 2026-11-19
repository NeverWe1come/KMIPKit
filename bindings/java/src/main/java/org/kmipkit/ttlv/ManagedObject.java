package org.kmipkit.ttlv;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import java.util.List;

/** Owned, validated managed object with a lossless generic TTLV view. */
public final class ManagedObject extends ManagedTtlvValue {
    private final long objectType;

    private ManagedObject(long nativeHandle, long objectType) {
        super(nativeHandle, 1, (int) objectType);
        this.objectType = objectType;
    }

    /** Copies and validates a managed object without consuming its source Item. */
    public static ManagedObject fromItem(long objectType, TtlvItem item) {
        if (objectType < 0 || objectType > 0xffff_ffffL || item == null) {
            throw new InvalidInputException("managed object input is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        return item.nativeHandleOwner().withValue(value -> new ManagedObject(
                NativeExtensionRegistry.managedObjectFromItem((int) objectType, value), objectType));
    }

    /** Copies caller-owned field Items and orders recognized fields by the KMIP table. */
    public static ManagedObject fromFields(long objectType, List<TtlvItem> fields) {
        if (objectType < 0 || objectType > 0xffff_ffffL || fields == null) {
            throw new InvalidInputException("managed object fields are invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        return new ManagedObject(withFields(fields, values ->
                NativeExtensionRegistry.managedObjectFromFields((int) objectType, values)), objectType);
    }

    /** Builds a Certificate from its typed table fields. */
    public static ManagedObject certificate(List<TtlvItem> fields) { return fromFields(1, fields); }
    /** Builds a Symmetric Key from its typed table fields. */
    public static ManagedObject symmetricKey(List<TtlvItem> fields) { return fromFields(2, fields); }
    /** Builds a Public Key from its typed table fields. */
    public static ManagedObject publicKey(List<TtlvItem> fields) { return fromFields(3, fields); }
    /** Builds a Private Key from its typed table fields. */
    public static ManagedObject privateKey(List<TtlvItem> fields) { return fromFields(4, fields); }
    /** Builds a Split Key from its typed table fields. */
    public static ManagedObject splitKey(List<TtlvItem> fields) { return fromFields(5, fields); }
    /** Builds Secret Data from its typed table fields. */
    public static ManagedObject secretData(List<TtlvItem> fields) { return fromFields(7, fields); }
    /** Builds an Opaque Object from its typed table fields. */
    public static ManagedObject opaqueObject(List<TtlvItem> fields) { return fromFields(8, fields); }
    /** Builds a PGP Key from its typed table fields. */
    public static ManagedObject pgpKey(List<TtlvItem> fields) { return fromFields(9, fields); }
    /** Builds a Certificate Request from its typed table fields. */
    public static ManagedObject certificateRequest(List<TtlvItem> fields) { return fromFields(10, fields); }

    /** Returns the source-preserving KMIP Object Type raw value. */
    public long objectType() {
        return objectType;
    }

    /** Reads the raw Certificate Type enumeration while retaining open values. */
    public long certificateType() { return readEnumerationField(0x0042001d); }

    /** Returns a caller-owned copy of the Certificate Value bytes. */
    public byte[] certificateValue() { return readBytesField(0x0042001e); }

    /** Reads the raw Certificate Request Type enumeration. */
    public long certificateRequestType() { return readEnumerationField(0x00420019); }

    /** Returns a caller-owned copy of the Certificate Request Value bytes. */
    public byte[] certificateRequestValue() { return readBytesField(0x00420140); }

    /** Reads the raw Opaque Data Type enumeration. */
    public long opaqueDataType() { return readEnumerationField(0x00420059); }

    /** Returns a caller-owned copy of the Opaque Data Value bytes. */
    public byte[] opaqueDataValue() { return readBytesField(0x0042005a); }

    /** Reads the PGP Key Version integer. */
    public int pgpKeyVersion() { return readIntegerField(0x004200be); }

    /** Reads the raw Secret Data Type enumeration. */
    public long secretDataType() { return readEnumerationField(0x00420086); }

    /** Reads the Split Key Part Identifier. */
    public int keyPartIdentifier() { return readIntegerField(0x00420044); }

    /** Reads the raw Split Key Method enumeration. */
    public long splitKeyMethod() { return readEnumerationField(0x0042008a); }

    /** Reads the Split Key Parts count. */
    public int splitKeyParts() { return readIntegerField(0x0042008b); }

    /** Reads the Split Key Threshold. */
    public int splitKeyThreshold() { return readIntegerField(0x0042008c); }

    /** Reports whether this object has its required Key Block field. */
    public boolean hasKeyBlock() { return hasField(0x00420040); }

    /** Returns a caller-owned copy of the optional Split Key Prime Field Size. */
    public byte[] primeFieldSize() { return readBytesField(0x00420062); }

}
