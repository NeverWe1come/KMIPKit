package org.kmipkit.protocol;

import java.util.Arrays;
import java.util.Optional;
import java.util.OptionalInt;
import java.util.OptionalLong;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;

/** Typed KMIP 2.1 Cryptographic Parameters with raw-preserving enumerations. */
public final class CryptographicParameters implements AutoCloseable {
    private static final int FIELD_COUNT = 18;
    private final NativeHandle handle;

    private CryptographicParameters(long nativeHandle) {
        handle = NativeHandle.owned(
                nativeHandle, NativeExtensionRegistry.CRYPTOGRAPHIC_PARAMETERS);
    }

    /** Creates a builder whose setters are serialized by the Rust Table 59 model. */
    public static Builder builder() {
        return new Builder();
    }

    /** Copies and validates a generic TTLV Structure using the Rust schema. */
    public static CryptographicParameters fromTtlv(TtlvStructure structure) {
        if (structure == null) {
            throw new InvalidInputException("Cryptographic Parameters structure is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        return structure.nativeHandleOwner().withValue(rawStructure ->
                new CryptographicParameters(
                        NativeExtensionRegistry.cryptographicParametersFromTtlv(rawStructure)));
    }

    /** Consumes this typed value into its lossless generic TTLV Structure. */
    public TtlvStructure intoTtlv() {
        NativeExtensionRegistry.ensureLoaded();
        long raw = handle.transfer();
        return TtlvStructure.fromNative(
                NativeExtensionRegistry.cryptographicParametersIntoTtlv(raw), this);
    }

    /** Returns the raw Block Cipher Mode Enumeration, including future values, when present. */
    public OptionalLong blockCipherMode() { return enumeration(0); }
    /** Returns the raw Padding Method Enumeration, including future values, when present. */
    public OptionalLong paddingMethod() { return enumeration(1); }
    /** Returns the raw Hashing Algorithm Enumeration, including future values, when present. */
    public OptionalLong hashingAlgorithm() { return enumeration(2); }
    /** Returns the raw Key Role Type Enumeration, including future values, when present. */
    public OptionalLong keyRoleType() { return enumeration(3); }
    /** Returns the raw Digital Signature Algorithm Enumeration, including future values, when present. */
    public OptionalLong digitalSignatureAlgorithm() { return enumeration(4); }
    /** Returns the raw Cryptographic Algorithm Enumeration, including future values, when present. */
    public OptionalLong cryptographicAlgorithm() { return enumeration(5); }
    /** Returns Random IV when present. */
    public Optional<Boolean> randomIv() {
        Fields fields = readFields();
        return fields.present()[6] ? Optional.of(fields.values()[6] != 0) : Optional.empty();
    }
    /** Returns IV Length when present. */
    public OptionalInt ivLength() { return integer(7); }
    /** Returns Tag Length when present. */
    public OptionalInt tagLength() { return integer(8); }
    /** Returns Fixed Field Length when present. */
    public OptionalInt fixedFieldLength() { return integer(9); }
    /** Returns Invocation Field Length when present. */
    public OptionalInt invocationFieldLength() { return integer(10); }
    /** Returns Counter Length when present. */
    public OptionalInt counterLength() { return integer(11); }
    /** Returns Initial Counter Value when present. */
    public OptionalInt initialCounterValue() { return integer(12); }
    /** Returns Salt Length when present. */
    public OptionalInt saltLength() { return integer(13); }
    /** Returns the raw Mask Generator Enumeration, including future values, when present. */
    public OptionalLong maskGenerator() { return enumeration(14); }
    /** Returns the raw Mask Generator Hashing Algorithm Enumeration when present. */
    public OptionalLong maskGeneratorHashingAlgorithm() { return enumeration(15); }
    /**
     * Returns a caller-owned P Source copy when present.
     *
     * <p>The Java byte array is outside KMIPKit's deterministic zeroization guarantee.
     */
    public Optional<byte[]> pSource() {
        Fields fields = readFields();
        return fields.present()[16] ? Optional.of(fields.pSource()) : Optional.empty();
    }
    /** Returns Trailer Field when present. */
    public OptionalInt trailerField() { return integer(17); }

    private OptionalLong enumeration(int index) {
        Fields fields = readFields();
        return fields.present()[index]
                ? OptionalLong.of(fields.values()[index])
                : OptionalLong.empty();
    }

    private OptionalInt integer(int index) {
        Fields fields = readFields();
        return fields.present()[index]
                ? OptionalInt.of(Math.toIntExact(fields.values()[index]))
                : OptionalInt.empty();
    }

    private Fields readFields() {
        return handle.withValue(raw -> {
            boolean[] present = new boolean[FIELD_COUNT];
            long[] values = new long[FIELD_COUNT];
            byte[] pSource = NativeExtensionRegistry.cryptographicParametersReadFields(
                    raw, present, values);
            return new Fields(present, values, pSource == null ? new byte[0] : pSource);
        });
    }

    private record Fields(boolean[] present, long[] values, byte[] pSource) {}

    @Override
    public void close() {
        handle.close();
    }

    @Override
    public String toString() {
        return "CryptographicParameters([REDACTED])";
    }

    /** Pure-Java fluent input record; the Rust builder performs validation and ordering. */
    public static final class Builder implements AutoCloseable {
        private final boolean[] present = new boolean[FIELD_COUNT];
        private final long[] values = new long[FIELD_COUNT];
        private byte[] pSource;
        private boolean built;

        private Builder() {}

        public Builder blockCipherMode(long raw) { return enumeration(0, raw); }
        public Builder paddingMethod(long raw) { return enumeration(1, raw); }
        public Builder hashingAlgorithm(long raw) { return enumeration(2, raw); }
        public Builder keyRoleType(long raw) { return enumeration(3, raw); }
        public Builder digitalSignatureAlgorithm(long raw) { return enumeration(4, raw); }
        public Builder cryptographicAlgorithm(long raw) { return enumeration(5, raw); }
        public Builder randomIv(boolean value) { return set(6, value ? 1 : 0); }
        public Builder ivLength(int value) { return set(7, value); }
        public Builder tagLength(int value) { return set(8, value); }
        public Builder fixedFieldLength(int value) { return set(9, value); }
        public Builder invocationFieldLength(int value) { return set(10, value); }
        public Builder counterLength(int value) { return set(11, value); }
        public Builder initialCounterValue(int value) { return set(12, value); }
        public Builder saltLength(int value) { return set(13, value); }
        public Builder maskGenerator(long raw) { return enumeration(14, raw); }
        public Builder maskGeneratorHashingAlgorithm(long raw) { return enumeration(15, raw); }
        public Builder trailerField(int value) { return set(17, value); }

        /** Copies P Source bytes into builder-owned storage, cleared after build or close. */
        public Builder pSource(byte[] value) {
            ensureOpen();
            if (value == null || value.length > 16 * 1024 * 1024) {
                throw new InvalidInputException("P Source is invalid");
            }
            wipe(pSource);
            pSource = value.clone();
            present[16] = true;
            return this;
        }

        /** Builds a typed model; Rust validates values and emits Table 59 order. */
        public CryptographicParameters build() {
            ensureOpen();
            NativeExtensionRegistry.ensureLoaded();
            try {
                return new CryptographicParameters(
                        NativeExtensionRegistry.cryptographicParametersFromFields(
                                present.clone(), values.clone(), pSource));
            } finally {
                built = true;
                wipe(pSource);
                pSource = null;
            }
        }

        @Override
        public void close() {
            built = true;
            wipe(pSource);
            pSource = null;
        }

        private Builder enumeration(int index, long raw) {
            if (raw < 0 || raw > 0xffff_ffffL) {
                throw new InvalidInputException("Enumeration value is outside uint32");
            }
            return set(index, raw);
        }

        private Builder set(int index, long raw) {
            ensureOpen();
            present[index] = true;
            values[index] = raw;
            return this;
        }

        private void ensureOpen() {
            if (built) throw new InvalidInputException("Cryptographic Parameters builder is closed");
        }

        private static void wipe(byte[] value) {
            if (value != null) Arrays.fill(value, (byte) 0);
        }
    }
}
