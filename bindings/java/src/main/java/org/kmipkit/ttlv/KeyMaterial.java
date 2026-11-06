package org.kmipkit.ttlv;

import java.util.List;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;

/** Owned, validated KMIP Key Material with explicit format and a TTLV view. */
public final class KeyMaterial extends ManagedTtlvValue {
    private final long formatRaw;

    private KeyMaterial(long nativeHandle, long formatRaw) {
        super(nativeHandle, 4, (int) formatRaw);
        this.formatRaw = formatRaw;
    }

    /** Copies and validates Key Material for the supplied open Key Format Type. */
    public static KeyMaterial fromItem(long formatRaw, TtlvItem item) {
        if (formatRaw < 0 || formatRaw > 0xffff_ffffL || item == null) {
            throw new InvalidInputException("Key Material input is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        return item.nativeHandleOwner().withValue(value -> new KeyMaterial(
                NativeExtensionRegistry.keyMaterialFromItem((int) formatRaw, value), formatRaw));
    }

    /** Copies caller-owned fields for the supplied open Key Format Type. */
    public static KeyMaterial fromFields(long formatRaw, List<TtlvItem> fields) {
        if (formatRaw < 0 || formatRaw > 0xffff_ffffL) {
            throw new InvalidInputException("Key Material format is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        return new KeyMaterial(withFields(fields, values ->
                NativeExtensionRegistry.keyMaterialFromFields((int) formatRaw, values)), formatRaw);
    }

    /** Returns the source-preserving Key Format Type raw value. */
    public long formatRaw() { return formatRaw; }

    /** Returns a caller-owned copy of a transparent RSA public modulus. */
    public byte[] modulus() { return readBytesField(0x00420052); }

    /** Returns a caller-owned copy of a transparent RSA public exponent. */
    public byte[] publicExponent() { return readBytesField(0x0042006c); }

    /** Reads a caller-owned transparent byte-string or BigInteger field by raw tag. */
    public byte[] fieldBytes(int rawTag) { return readBytesField(rawTag); }

    /** Reads a transparent enumeration field by raw tag. */
    public long fieldEnumeration(int rawTag) { return readEnumerationField(rawTag); }

    /** Reads a transparent integer field by raw tag. */
    public int fieldInteger(int rawTag) { return readIntegerField(rawTag); }

    /** Reads the Transparent Symmetric Key's Key Byte String. */
    public byte[] keyByteString() { return readBytesField(0x0042003f); }

    /** Reads the transparent P component used by DSA and DH formats. */
    public byte[] p() { return readBytesField(0x0042005e); }

    /** Reads the transparent Q component used by DSA and DH formats. */
    public byte[] q() { return readBytesField(0x00420071); }

    /** Reads the transparent G component used by DSA and DH formats. */
    public byte[] g() { return readBytesField(0x00420037); }

    /** Reads the transparent X component used by private DSA and DH formats. */
    public byte[] x() { return readBytesField(0x0042009f); }

    /** Reads the transparent Y component used by public DSA and DH formats. */
    public byte[] y() { return readBytesField(0x004200a0); }

    /** Reads the transparent J component used by DH formats. */
    public byte[] j() { return readBytesField(0x0042003e); }

    /** Reads the transparent EC private D component. */
    public byte[] d() { return readBytesField(0x0042002e); }

    /** Reads the transparent EC public Q String. */
    public byte[] qString() { return readBytesField(0x00420072); }

    /** Reads the raw Recommended Curve value. */
    public long recommendedCurve() { return readEnumerationField(0x00420075); }

    /** Reads the RSA Private Key's optional private exponent. */
    public byte[] privateExponent() { return readBytesField(0x00420063); }

    /** Reads the RSA Private Key's optional prime exponent P. */
    public byte[] primeExponentP() { return readBytesField(0x00420060); }

    /** Reads the RSA Private Key's optional prime exponent Q. */
    public byte[] primeExponentQ() { return readBytesField(0x00420061); }

    /** Reads the RSA Private Key's optional CRT coefficient. */
    public byte[] crtCoefficient() { return readBytesField(0x00420027); }

}
