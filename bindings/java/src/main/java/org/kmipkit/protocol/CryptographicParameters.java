package org.kmipkit.protocol;

import java.util.Arrays;
import java.util.Optional;
import java.util.OptionalInt;
import java.util.OptionalLong;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;

/** Typed KMIP 2.1 / 3.0 Cryptographic Parameters with raw-preserving enumerations. */
public final class CryptographicParameters implements AutoCloseable {
    private static final int FIELD_COUNT = 23;
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

    /** Returns the raw KEM Algorithm Enumeration (KMIP 3.0), including future values, when present. */
    public OptionalLong kemAlgorithm() { return enumeration(18); }

    /** Returns Deterministic (KMIP 3.0) when present. */
    public Optional<Boolean> deterministic() {
        Fields fields = readFields();
        return fields.present()[19] ? Optional.of(fields.values()[19] != 0) : Optional.empty();
    }

    /** Returns a caller-owned Context String copy (KMIP 3.0) when present. */
    public Optional<byte[]> contextString() {
        Fields fields = readFields();
        return fields.present()[20] ? Optional.of(fields.contextString()) : Optional.empty();
    }

    /** Returns a caller-owned Seed copy (KMIP 3.0) when present. */
    public Optional<byte[]> seed() {
        Fields fields = readFields();
        return fields.present()[21] ? Optional.of(fields.seed()) : Optional.empty();
    }

    /** Returns a caller-owned Input Key Material copy (KMIP 3.0) when present. */
    public Optional<byte[]> inputKeyMaterial() {
        Fields fields = readFields();
        return fields.present()[22] ? Optional.of(fields.inputKeyMaterial()) : Optional.empty();
    }

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
            byte[][] byteArrays = NativeExtensionRegistry.cryptographicParametersReadFieldsV2(
                    raw, present, values);
            byte[] pSource = byteArrays[0] == null ? new byte[0] : byteArrays[0];
            byte[] contextString = byteArrays[1] == null ? new byte[0] : byteArrays[1];
            byte[] seed = byteArrays[2] == null ? new byte[0] : byteArrays[2];
            byte[] inputKeyMaterial = byteArrays[3] == null ? new byte[0] : byteArrays[3];
            return new Fields(present, values, pSource, contextString, seed, inputKeyMaterial);
        });
    }

    /** Internal lifecycle bridge for KMIPKit's typed request builders. */
    public NativeHandle nativeHandleOwner() {
        return handle;
    }

    private record Fields(
            boolean[] present,
            long[] values,
            byte[] pSource,
            byte[] contextString,
            byte[] seed,
            byte[] inputKeyMaterial) {}

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
        private byte[] contextString;
        private byte[] seed;
        private byte[] inputKeyMaterial;
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
        public Builder kemAlgorithm(long raw) { return enumeration(18, raw); }
        public Builder deterministic(boolean value) { return set(19, value ? 1 : 0); }

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

        public Builder contextString(byte[] value) {
            ensureOpen();
            if (value == null || value.length > 16 * 1024 * 1024) {
                throw new InvalidInputException("Context String is invalid");
            }
            wipe(contextString);
            contextString = value.clone();
            present[20] = true;
            return this;
        }

        public Builder seed(byte[] value) {
            ensureOpen();
            if (value == null || value.length > 16 * 1024 * 1024) {
                throw new InvalidInputException("Seed is invalid");
            }
            wipe(seed);
            seed = value.clone();
            present[21] = true;
            return this;
        }

        public Builder inputKeyMaterial(byte[] value) {
            ensureOpen();
            if (value == null || value.length > 16 * 1024 * 1024) {
                throw new InvalidInputException("Input Key Material is invalid");
            }
            wipe(inputKeyMaterial);
            inputKeyMaterial = value.clone();
            present[22] = true;
            return this;
        }

        /** Builds a typed model; Rust validates values and emits Table 59 order. */
        public CryptographicParameters build() {
            ensureOpen();
            NativeExtensionRegistry.ensureLoaded();
            try {
                return new CryptographicParameters(
                        NativeExtensionRegistry.cryptographicParametersFromFieldsV2(
                                present.clone(), values.clone(), pSource,
                                contextString, seed, inputKeyMaterial));
            } finally {
                built = true;
                wipe(pSource);
                pSource = null;
                wipe(contextString);
                contextString = null;
                wipe(seed);
                seed = null;
                wipe(inputKeyMaterial);
                inputKeyMaterial = null;
            }
        }

        @Override
        public void close() {
            built = true;
            wipe(pSource);
            pSource = null;
            wipe(contextString);
            contextString = null;
            wipe(seed);
            seed = null;
            wipe(inputKeyMaterial);
            inputKeyMaterial = null;
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
