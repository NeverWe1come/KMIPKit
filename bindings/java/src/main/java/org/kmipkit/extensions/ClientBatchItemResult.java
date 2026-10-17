package org.kmipkit.extensions;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Optional;
import java.util.OptionalLong;
import java.util.function.Function;
import java.util.function.Supplier;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;
import org.jspecify.annotations.Nullable;

/** Common, operation-independent metadata for one response batch item. */
public final class ClientBatchItemResult implements AutoCloseable {
    private final NativeHandle handle;

    ClientBatchItemResult(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.CLIENT_BATCH_ITEM_RESPONSE);
    }

    /** Returns the forward-compatible completed or Pending outcome discriminator. */
    public BatchOutcomeKind outcomeKind() {
        return new BatchOutcomeKind(metadata()[0]);
    }

    /** Returns the raw unsigned KMIP Result Status value, preserving unknown values. */
    public long resultStatus() {
        return metadata()[1];
    }

    /** Returns the optional raw unsigned KMIP Result Reason value. */
    public OptionalLong resultReason() {
        long[] values = metadata();
        return values[2] == 0 ? OptionalLong.empty() : OptionalLong.of(values[3]);
    }

    /** Returns version 2 metadata, including the source operation and exact Result Message. */
    public ClientBatchItemMetadataV2 metadataV2() {
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(NativeExtensionRegistry::clientBatchItemResponseMetadataV2);
        return parseMetadataV2(values);
    }

    /** Returns the raw unsigned KMIP operation value, preserving unknown operations. */
    public long operation() {
        return metadataV2().operation();
    }

    /** Returns the exact optional server Result Message. */
    public Optional<String> resultMessage() {
        return metadataV2().resultMessage();
    }

    /** Returns the successful Create response, including the identifier's exact wire type. */
    public Optional<CreateResponse> createResponse() {
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(NativeExtensionRegistry::clientBatchItemResponseCreate);
        return parseCreateResponse(values);
    }

    /** Returns the generated certificate identifier only for a successful Certify response. */
    public Optional<CertifyResponse> certifyResponse() {
        if (resultStatus() != 0) return Optional.empty();
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(NativeExtensionRegistry::clientBatchItemResponseCertify);
        return parseCertifyResponse(values);
    }

    /** Returns the typed Check result, including policy fields and generic TTLV on failure. */
    public Optional<CheckResponse> checkResponse() {
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(NativeExtensionRegistry::clientBatchItemResponseCheck);
        return parseCheckResponse(values);
    }

    /** Returns the typed Delegated Login result and its raw-preserving Ticket when present. */
    public Optional<DelegatedLoginResponse> delegatedLoginResponse() {
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(
                NativeExtensionRegistry::clientBatchItemResponseDelegatedLogin);
        return parseDelegatedLoginResponseAfterNativeReturn(
                values, () -> ClientBatchItemResult::parseDelegatedLoginResponse);
    }

    /**
     * Returns the resolved identifier and complete generic TTLV from a successful Get Usage
     * Allocation response. The returned response owns an independent TTLV copy; callers must
     * close it when finished. Closing this batch item does not close the returned response.
     */
    public Optional<GetUsageAllocationResponse> getUsageAllocationResponse() {
        if (resultStatus() != 0) return Optional.empty();
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(
                NativeExtensionRegistry::clientBatchItemResponseGetUsageAllocation);
        return parseGetUsageAllocationResponse(values);
    }

    /** Returns distinct private and public identifiers for a successful Create Key Pair response. */
    public Optional<CreateKeyPairResponse> createKeyPairResponse() {
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(NativeExtensionRegistry::clientBatchItemResponseCreateKeyPair);
        return parseCreateKeyPairResponse(values);
    }

    /** Returns the typed Hash response only for a successful Hash operation. */
    public Optional<HashResponse> hashResponse() {
        if (resultStatus() != 0) return Optional.empty();
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(NativeExtensionRegistry::clientBatchItemResponseHash);
        return parseHashResponse(values);
    }

    /** Returns the typed MAC response only for a successful MAC operation. */
    public Optional<MacResponse> macResponse() {
        if (resultStatus() != 0) return Optional.empty();
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(NativeExtensionRegistry::clientBatchItemResponseMac);
        return parseMacResponse(values);
    }

    /** Returns the typed MAC Verify response only for a successful operation. */
    public Optional<MacVerifyResponse> macVerifyResponse() {
        if (resultStatus() != 0) return Optional.empty();
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(NativeExtensionRegistry::clientBatchItemResponseMacVerify);
        return parseMacVerifyResponse(values);
    }

    /** Returns the typed Sign response only for a successful Sign operation. */
    public Optional<SignResponse> signResponse() {
        if (resultStatus() != 0) return Optional.empty();
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(NativeExtensionRegistry::clientBatchItemResponseSign);
        return parseSignResponse(values);
    }

    /** Returns the typed Signature Verify response only for a successful operation. */
    public Optional<SignatureVerifyResponse> signatureVerifyResponse() {
        if (resultStatus() != 0) return Optional.empty();
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(
                NativeExtensionRegistry::clientBatchItemResponseSignatureVerify);
        return parseSignatureVerifyResponse(values);
    }

    /** Returns the one-or-more Create Split Key identifiers in response wire order. */
    public Optional<List<UniqueIdentifier>> createSplitKeyUniqueIdentifiers() {
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(NativeExtensionRegistry::clientBatchItemResponseCreateSplitKey);
        return parseCreateSplitKeyUniqueIdentifiers(values);
    }

    /** Successful Create response fields. */
    public static final class CreateResponse {
        private final long objectType;
        private final UniqueIdentifier uniqueIdentifier;

        private CreateResponse(long objectType, UniqueIdentifier uniqueIdentifier) {
            this.objectType = objectType;
            this.uniqueIdentifier = uniqueIdentifier;
        }

        public long objectType() { return objectType; }
        public UniqueIdentifier uniqueIdentifier() { return uniqueIdentifier; }

        @Override public String toString() { return "CreateResponse[objectType=" + objectType + ", uniqueIdentifier=redacted]"; }
    }

    /** Successful Certify response with its owned complete generic TTLV. */
    public static final class CertifyResponse implements AutoCloseable {
        private final UniqueIdentifier uniqueIdentifier;
        private final TtlvStructure responseTtlv;

        private CertifyResponse(UniqueIdentifier uniqueIdentifier, TtlvStructure responseTtlv) {
            this.uniqueIdentifier = uniqueIdentifier;
            this.responseTtlv = responseTtlv;
        }

        public UniqueIdentifier uniqueIdentifier() { return uniqueIdentifier; }

        /** Returns the complete ordered response structure owned by this result. */
        public TtlvStructure responseTtlv() { return responseTtlv; }

        @Override public void close() { responseTtlv.close(); }

        @Override public String toString() { return "CertifyResponse[uniqueIdentifier=redacted]"; }
    }

    /** Typed Check fields with the caller-owned complete ordered response TTLV. */
    public static final class CheckResponse implements AutoCloseable {
        private final Optional<UniqueIdentifier> uniqueIdentifier;
        private final OptionalLong usageLimitsCount;
        private final OptionalLong cryptographicUsageMask;
        private final OptionalLong leaseTime;
        private final TtlvStructure responseTtlv;

        private CheckResponse(Optional<UniqueIdentifier> uniqueIdentifier,
                OptionalLong usageLimitsCount, OptionalLong cryptographicUsageMask,
                OptionalLong leaseTime, TtlvStructure responseTtlv) {
            this.uniqueIdentifier = uniqueIdentifier;
            this.usageLimitsCount = usageLimitsCount;
            this.cryptographicUsageMask = cryptographicUsageMask;
            this.leaseTime = leaseTime;
            this.responseTtlv = responseTtlv;
        }

        public Optional<UniqueIdentifier> uniqueIdentifier() { return uniqueIdentifier; }
        public OptionalLong usageLimitsCount() { return usageLimitsCount; }
        public OptionalLong cryptographicUsageMask() { return cryptographicUsageMask; }
        public OptionalLong leaseTime() { return leaseTime; }

        /** Returns the complete ordered response structure owned by this result. */
        public TtlvStructure responseTtlv() { return responseTtlv; }

        @Override public void close() { responseTtlv.close(); }

        @Override public String toString() { return "CheckResponse[fields=redacted]"; }
    }

    /** Delegated Login Ticket fields and the independently owned generic Ticket TTLV copy. */
    public static final class DelegatedLoginResponse implements AutoCloseable {
        private final OptionalLong ticketTypeRaw;
        private @Nullable byte[] ticketValue;
        private @Nullable TtlvStructure ticketTtlv;

        private DelegatedLoginResponse(OptionalLong ticketTypeRaw,
                @Nullable byte[] ticketValue, @Nullable TtlvStructure ticketTtlv) {
            this.ticketTypeRaw = ticketTypeRaw;
            this.ticketValue = copyBytes(ticketValue);
            this.ticketTtlv = ticketTtlv;
        }

        /** Returns the raw unsigned Ticket Type, preserving unknown values. */
        public OptionalLong ticketTypeRaw() { return ticketTypeRaw; }

        /** Returns a caller-owned Ticket Value copy; callers must protect and clear it. */
        public Optional<byte[]> ticketValue() { return copiedBytes(ticketValue); }

        /** Returns the complete ordered Ticket structure, including unknown children. */
        public Optional<TtlvStructure> ticketTtlv() { return Optional.ofNullable(ticketTtlv); }

        @Override public void close() {
            if (ticketValue != null) Arrays.fill(ticketValue, (byte) 0);
            ticketValue = null;
            if (ticketTtlv != null) ticketTtlv.close();
            ticketTtlv = null;
        }

        @Override public String toString() {
            return "DelegatedLoginResponse[ticketTypeRaw=" + ticketTypeRaw
                    + ", ticketValue=redacted, ticketTtlv=preserved]";
        }
    }

    /** Successful Get Usage Allocation response with its owned complete generic TTLV. */
    public static final class GetUsageAllocationResponse implements AutoCloseable {
        private final UniqueIdentifier uniqueIdentifier;
        private final TtlvStructure responseTtlv;

        private GetUsageAllocationResponse(
                UniqueIdentifier uniqueIdentifier, TtlvStructure responseTtlv) {
            this.uniqueIdentifier = uniqueIdentifier;
            this.responseTtlv = responseTtlv;
        }

        public UniqueIdentifier uniqueIdentifier() { return uniqueIdentifier; }

        /** Returns the complete ordered response structure owned by this result. */
        public TtlvStructure responseTtlv() { return responseTtlv; }

        @Override public void close() { responseTtlv.close(); }

        @Override public String toString() {
            return "GetUsageAllocationResponse[uniqueIdentifier=redacted]";
        }
    }

    /** Successful Create Key Pair response with separate identifiers. */
    public static final class CreateKeyPairResponse {
        private final UniqueIdentifier privateKeyUniqueIdentifier;
        private final UniqueIdentifier publicKeyUniqueIdentifier;

        private CreateKeyPairResponse(UniqueIdentifier privateIdentifier, UniqueIdentifier publicIdentifier) {
            privateKeyUniqueIdentifier = privateIdentifier;
            publicKeyUniqueIdentifier = publicIdentifier;
        }

        public UniqueIdentifier privateKeyUniqueIdentifier() { return privateKeyUniqueIdentifier; }
        public UniqueIdentifier publicKeyUniqueIdentifier() { return publicKeyUniqueIdentifier; }

        @Override public String toString() { return "CreateKeyPairResponse[identifiers=redacted]"; }
    }

    /** Successful Hash response fields, copied from the native result. */
    public static final class HashResponse {
        private final @Nullable byte[] data;
        private final @Nullable byte[] correlationValue;

        private HashResponse(@Nullable byte[] data, @Nullable byte[] correlationValue) {
            this.data = copyBytes(data);
            this.correlationValue = copyBytes(correlationValue);
        }

        public Optional<byte[]> data() { return copiedBytes(data); }
        public Optional<byte[]> correlationValue() { return copiedBytes(correlationValue); }

        @Override public String toString() { return "HashResponse[fields=redacted]"; }
    }

    /** Successful MAC response fields, copied from the native result. */
    public static final class MacResponse {
        private final UniqueIdentifier uniqueIdentifier;
        private final @Nullable byte[] macData;
        private final @Nullable byte[] correlationValue;

        private MacResponse(UniqueIdentifier uniqueIdentifier,
                @Nullable byte[] macData, @Nullable byte[] correlationValue) {
            this.uniqueIdentifier = uniqueIdentifier;
            this.macData = copyBytes(macData);
            this.correlationValue = copyBytes(correlationValue);
        }

        public UniqueIdentifier uniqueIdentifier() { return uniqueIdentifier; }
        public Optional<byte[]> macData() { return copiedBytes(macData); }
        public Optional<byte[]> correlationValue() { return copiedBytes(correlationValue); }

        @Override public String toString() { return "MacResponse[fields=redacted]"; }
    }

    /** Successful MAC Verify response fields, including an unknown raw Validity Indicator. */
    public static final class MacVerifyResponse {
        private final UniqueIdentifier uniqueIdentifier;
        private final OptionalLong validityIndicatorRaw;
        private final @Nullable byte[] correlationValue;

        private MacVerifyResponse(UniqueIdentifier uniqueIdentifier,
                OptionalLong validityIndicatorRaw, @Nullable byte[] correlationValue) {
            this.uniqueIdentifier = uniqueIdentifier;
            this.validityIndicatorRaw = validityIndicatorRaw;
            this.correlationValue = copyBytes(correlationValue);
        }

        public UniqueIdentifier uniqueIdentifier() { return uniqueIdentifier; }
        public OptionalLong validityIndicatorRaw() { return validityIndicatorRaw; }
        public Optional<byte[]> correlationValue() { return copiedBytes(correlationValue); }

        @Override public String toString() { return "MacVerifyResponse[fields=redacted]"; }
    }

    /** Successful Sign response fields, copied from the native result. */
    public static final class SignResponse {
        private final UniqueIdentifier uniqueIdentifier;
        private final @Nullable byte[] signatureData;
        private final @Nullable byte[] correlationValue;

        private SignResponse(UniqueIdentifier uniqueIdentifier,
                @Nullable byte[] signatureData, @Nullable byte[] correlationValue) {
            this.uniqueIdentifier = uniqueIdentifier;
            this.signatureData = copyBytes(signatureData);
            this.correlationValue = copyBytes(correlationValue);
        }

        public UniqueIdentifier uniqueIdentifier() { return uniqueIdentifier; }
        public Optional<byte[]> signatureData() { return copiedBytes(signatureData); }
        public Optional<byte[]> correlationValue() { return copiedBytes(correlationValue); }

        @Override public String toString() { return "SignResponse[fields=redacted]"; }
    }

    /** Successful Signature Verify response fields, including an unknown raw Validity Indicator. */
    public static final class SignatureVerifyResponse {
        private final UniqueIdentifier uniqueIdentifier;
        private final OptionalLong validityIndicatorRaw;
        private final @Nullable byte[] recoveredData;
        private final @Nullable byte[] correlationValue;

        private SignatureVerifyResponse(UniqueIdentifier uniqueIdentifier,
                OptionalLong validityIndicatorRaw, @Nullable byte[] recoveredData,
                @Nullable byte[] correlationValue) {
            this.uniqueIdentifier = uniqueIdentifier;
            this.validityIndicatorRaw = validityIndicatorRaw;
            this.recoveredData = copyBytes(recoveredData);
            this.correlationValue = copyBytes(correlationValue);
        }

        public UniqueIdentifier uniqueIdentifier() { return uniqueIdentifier; }
        public OptionalLong validityIndicatorRaw() { return validityIndicatorRaw; }
        public Optional<byte[]> recoveredData() { return copiedBytes(recoveredData); }
        public Optional<byte[]> correlationValue() { return copiedBytes(correlationValue); }

        @Override public String toString() { return "SignatureVerifyResponse[fields=redacted]"; }
    }

    static Optional<HashResponse> parseHashResponse(Object[] values) {
        Object[] tuple = requireTuple(values, 3, "native Hash response is invalid");
        long[] fields = requireLongArray(tuple[0], 3, "native Hash response is invalid");
        if (!responsePresent(fields)) {
            requireNoResponseFields(fields, tuple);
            return Optional.empty();
        }
        return Optional.of(new HashResponse(
                responseBytes(fields[1], tuple[1]), responseBytes(fields[2], tuple[2])));
    }

    static Optional<MacResponse> parseMacResponse(Object[] values) {
        Object[] tuple = requireTuple(values, 4, "native MAC response is invalid");
        long[] fields = requireLongArray(tuple[0], 7, "native MAC response is invalid");
        if (!responsePresent(fields)) {
            requireNoResponseFields(fields, tuple);
            return Optional.empty();
        }
        return Optional.of(new MacResponse(
                requiredResponseIdentifier(fields, 1, tuple[1]),
                responseBytes(fields[5], tuple[2]), responseBytes(fields[6], tuple[3])));
    }

    static Optional<MacVerifyResponse> parseMacVerifyResponse(Object[] values) {
        Object[] tuple = requireTuple(values, 3, "native MAC Verify response is invalid");
        long[] fields = requireLongArray(tuple[0], 8, "native MAC Verify response is invalid");
        if (!responsePresent(fields)) {
            requireNoResponseFields(fields, tuple);
            return Optional.empty();
        }
        return Optional.of(new MacVerifyResponse(
                requiredResponseIdentifier(fields, 1, tuple[1]),
                optionalUnsigned(fields[5], fields[6]), responseBytes(fields[7], tuple[2])));
    }

    static Optional<SignResponse> parseSignResponse(Object[] values) {
        Object[] tuple = requireTuple(values, 4, "native Sign response is invalid");
        long[] fields = requireLongArray(tuple[0], 7, "native Sign response is invalid");
        if (!responsePresent(fields)) {
            requireNoResponseFields(fields, tuple);
            return Optional.empty();
        }
        return Optional.of(new SignResponse(
                requiredResponseIdentifier(fields, 1, tuple[1]),
                responseBytes(fields[5], tuple[2]), responseBytes(fields[6], tuple[3])));
    }

    static Optional<SignatureVerifyResponse> parseSignatureVerifyResponse(Object[] values) {
        Object[] tuple = requireTuple(values, 4, "native Signature Verify response is invalid");
        long[] fields = requireLongArray(tuple[0], 9, "native Signature Verify response is invalid");
        if (!responsePresent(fields)) {
            requireNoResponseFields(fields, tuple);
            return Optional.empty();
        }
        return Optional.of(new SignatureVerifyResponse(
                requiredResponseIdentifier(fields, 1, tuple[1]),
                optionalUnsigned(fields[5], fields[6]),
                responseBytes(fields[7], tuple[2]), responseBytes(fields[8], tuple[3])));
    }

    private static boolean responsePresent(long[] fields) {
        return responseFlag(fields[0]);
    }

    private static boolean responseFlag(long value) {
        if (value == 0) return false;
        if (value == 1) return true;
        throw new InvalidInputException("native response presence flag is invalid");
    }

    private static void requireNoResponseFields(long[] fields, Object[] tuple) {
        for (int index = 1; index < fields.length; index++) {
            if (fields[index] != 0) {
                throw new InvalidInputException("native absent response contains fields");
            }
        }
        for (int index = 1; index < tuple.length; index++) {
            if (tuple[index] != null) {
                throw new InvalidInputException("native absent response contains fields");
            }
        }
    }

    private static UniqueIdentifier requiredResponseIdentifier(
            long[] fields, int presenceIndex, Object value) {
        if (!responseFlag(fields[presenceIndex])) {
            if (fields[presenceIndex + 1] != 0 || fields[presenceIndex + 2] != 0
                    || fields[presenceIndex + 3] != 0 || value != null) {
                throw new InvalidInputException("native Unique Identifier presence is invalid");
            }
            throw new InvalidInputException("native successful response omitted its Unique Identifier");
        }
        long kind = fields[presenceIndex + 1];
        byte[] text = null;
        if (kind == 1) {
            text = requireBytes(value);
        } else if (value != null) {
            throw new InvalidInputException("native Unique Identifier text is invalid");
        }
        return UniqueIdentifier.fromNative(kind, text,
                fields[presenceIndex + 2], fields[presenceIndex + 3]);
    }

    private static OptionalLong optionalUnsigned(long present, long value) {
        if (!responseFlag(present)) {
            if (value != 0) {
                throw new InvalidInputException("native unsigned response value is invalid");
            }
            return OptionalLong.empty();
        }
        if (value < 0 || value > 0xffff_ffffL) {
            throw new InvalidInputException("native unsigned response value is invalid");
        }
        return OptionalLong.of(value);
    }

    private static OptionalLong optionalSigned(long present, long value) {
        if (!responseFlag(present)) {
            if (value != 0) {
                throw new InvalidInputException("native absent signed response value is invalid");
            }
            return OptionalLong.empty();
        }
        return OptionalLong.of(value);
    }

    private static OptionalLong optionalSignedInt(long present, long value) {
        OptionalLong result = optionalSigned(present, value);
        if (result.isPresent() && (value < Integer.MIN_VALUE || value > Integer.MAX_VALUE)) {
            throw new InvalidInputException("native Check usage mask is invalid");
        }
        return result;
    }

    private static Optional<UniqueIdentifier> optionalResponseIdentifier(long[] fields, Object value) {
        if (!responseFlag(fields[1])) {
            if (value != null) {
                throw new InvalidInputException("native absent Check identifier is invalid");
            }
            return Optional.empty();
        }
        return Optional.of(identifierRecord(value));
    }

    private static @Nullable byte[] responseBytes(long present, Object value) {
        if (!responseFlag(present)) {
            if (value != null) {
                throw new InvalidInputException("native absent response bytes are invalid");
            }
            return null;
        }
        return requireBytes(value);
    }

    private static @Nullable byte[] copyBytes(@Nullable byte[] value) {
        return value == null ? null : value.clone();
    }

    private static Optional<byte[]> copiedBytes(@Nullable byte[] value) {
        return value == null ? Optional.empty() : Optional.of(value.clone());
    }

    static ClientBatchItemMetadataV2 parseMetadataV2(Object[] values) {
        Object[] tuple = requireTuple(values, 2, "native batch result metadata v2 is invalid");
        if (!(tuple[0] instanceof long[] metadata)) {
            throw new InvalidInputException("native batch result metadata v2 is invalid");
        }
        byte[] message = tuple[1] == null ? null : requireBytes(tuple[1]);
        return new ClientBatchItemMetadataV2(metadata, message);
    }

    static Optional<CreateResponse> parseCreateResponse(Object[] values) {
        Object[] tuple = requireTuple(values, 2, "native Create response is invalid");
        long[] fields = requireLongArray(tuple[0], 5);
        if (fields[0] == 0) {
            return Optional.empty();
        }
        requirePresent(fields[0]);
        byte[] text = tuple[1] == null ? null : requireBytes(tuple[1]);
        return Optional.of(new CreateResponse(
                fields[1], identifier(fields[2], fields[3], fields[4], text)));
    }

    static Optional<CertifyResponse> parseCertifyResponse(Object[] values) {
        Object[] tuple = requireTuple(values, 3, "native Certify response is invalid");
        long[] fields = requireLongArray(tuple[0], 1, "native Certify response is invalid");
        long[] responseTtlvHandle = requireLongArray(
                tuple[2], 1, "native Certify response is invalid");
        if (!responsePresent(fields)) {
            if (tuple[1] != null || responseTtlvHandle[0] != 0) {
                if (responseTtlvHandle[0] > 0) {
                    TtlvStructure.fromNative(responseTtlvHandle[0], null).close();
                }
                throw new InvalidInputException("native absent Certify response contains fields");
            }
            return Optional.empty();
        }
        if (responseTtlvHandle[0] <= 0) {
            throw new InvalidInputException("native Certify response is invalid");
        }
        TtlvStructure responseTtlv = TtlvStructure.fromNative(responseTtlvHandle[0], null);
        try {
            return Optional.of(new CertifyResponse(identifierRecord(tuple[1]), responseTtlv));
        } catch (RuntimeException exception) {
            responseTtlv.close();
            throw exception;
        }
    }

    static Optional<CheckResponse> parseCheckResponse(Object[] values) {
        Object[] tuple = requireTuple(values, 3, "native Check response is invalid");
        long[] fields = requireLongArray(tuple[0], 8, "native Check response is invalid");
        long[] responseTtlvHandle = requireLongArray(
                tuple[2], 1, "native Check response is invalid");
        if (!responsePresent(fields)) {
            boolean hasUnexpectedFields = tuple[1] != null || responseTtlvHandle[0] != 0;
            for (int index = 1; index < fields.length; index++) {
                if (fields[index] != 0) {
                    hasUnexpectedFields = true;
                }
            }
            if (responseTtlvHandle[0] > 0) {
                TtlvStructure.fromNative(responseTtlvHandle[0], null).close();
            }
            if (hasUnexpectedFields) {
                throw new InvalidInputException("native absent Check response contains fields");
            }
            return Optional.empty();
        }
        if (responseTtlvHandle[0] <= 0) {
            throw new InvalidInputException("native Check response is invalid");
        }
        TtlvStructure responseTtlv = TtlvStructure.fromNative(responseTtlvHandle[0], null);
        try {
            Optional<UniqueIdentifier> identifier = optionalResponseIdentifier(fields, tuple[1]);
            OptionalLong count = optionalSigned(fields[2], fields[3]);
            OptionalLong usageMask = optionalSignedInt(fields[4], fields[5]);
            OptionalLong lease = optionalUnsigned(fields[6], fields[7]);
            return Optional.of(new CheckResponse(identifier, count, usageMask, lease, responseTtlv));
        } catch (RuntimeException exception) {
            responseTtlv.close();
            throw exception;
        }
    }

    static Optional<DelegatedLoginResponse> parseDelegatedLoginResponse(Object[] values) {
        return parseDelegatedLoginResponse(values, Optional::of);
    }

    static Optional<DelegatedLoginResponse> parseDelegatedLoginResponseAfterNativeReturn(
            Object[] values,
            Supplier<Function<Object[], Optional<DelegatedLoginResponse>>> parserFactory) {
        return parserFactory.get().apply(values);
    }

    private static void cleanupDelegatedLoginResponseValues(Object[] values) {
        if (values == null) return;
        if (values.length > 1 && values[1] instanceof byte[] ticketValue) {
            Arrays.fill(ticketValue, (byte) 0);
        }
        if (values.length > 2 && values[2] instanceof long[] ticketHandle
                && ticketHandle.length == 1 && ticketHandle[0] > 0) {
            long rawHandle = ticketHandle[0];
            ticketHandle[0] = 0;
            NativeExtensionRegistry.release(NativeExtensionRegistry.TTLV_STRUCTURE, rawHandle);
        }
    }

    static Optional<DelegatedLoginResponse> parseDelegatedLoginResponse(
            Object[] values,
            Function<DelegatedLoginResponse, Optional<DelegatedLoginResponse>> resultTransfer) {
        Object[] tuple = values;
        long rawHandle = 0;
        TtlvStructure ownedTtlv = null;
        DelegatedLoginResponse response = null;
        byte[] ownedValue = null;
        boolean transferred = false;
        try {
            tuple = requireTuple(values, 3, "native Delegated Login response is invalid");
            if (tuple[2] instanceof long[] handleValues && handleValues.length == 1) {
                rawHandle = handleValues[0];
            }
            long[] fields = requireLongArray(
                    tuple[0], 3, "native Delegated Login response is invalid");
            long[] ticketTtlvHandle = requireLongArray(
                    tuple[2], 1, "native Delegated Login response is invalid");
            rawHandle = ticketTtlvHandle[0];
            boolean responsePresent = responseFlag(fields[0]);
            boolean ticketPresent = responseFlag(fields[1]);
            if (!ticketPresent) {
                if (fields[2] != 0 || tuple[1] != null || rawHandle != 0) {
                    throw new InvalidInputException(
                            "native absent Delegated Login Ticket contains fields");
                }
                if (!responsePresent) return Optional.empty();
                response = new DelegatedLoginResponse(OptionalLong.empty(), null, null);
                Optional<DelegatedLoginResponse> result = resultTransfer.apply(response);
                transferred = true;
                response = null;
                return result;
            }
            if (!responsePresent || fields[2] < 0 || fields[2] > 0xffff_ffffL
                    || rawHandle <= 0) {
                throw new InvalidInputException("native Delegated Login Ticket is invalid");
            }
            ownedValue = requireBytes(tuple[1]);
            ownedTtlv = TtlvStructure.fromNative(rawHandle, null);
            clearDelegatedLoginTtlvHandle(tuple, rawHandle);
            rawHandle = 0;
            response = new DelegatedLoginResponse(
                    OptionalLong.of(fields[2]), ownedValue, ownedTtlv);
            Arrays.fill(ownedValue, (byte) 0);
            ownedTtlv = null;
            Optional<DelegatedLoginResponse> result = resultTransfer.apply(response);
            transferred = true;
            response = null;
            return result;
        } finally {
            if (response != null) {
                response.close();
            }
            if (ownedValue != null) {
                Arrays.fill(ownedValue, (byte) 0);
            }
            if (!transferred && tuple != null && tuple.length > 1 && tuple[1] instanceof byte[] value
                    && value != ownedValue) {
                Arrays.fill(value, (byte) 0);
            }
            if (ownedTtlv != null) {
                ownedTtlv.close();
            } else if (rawHandle > 0) {
                clearDelegatedLoginTtlvHandle(tuple, rawHandle);
                NativeExtensionRegistry.release(NativeExtensionRegistry.TTLV_STRUCTURE, rawHandle);
            }
        }
    }

    private static void clearDelegatedLoginTtlvHandle(Object[] tuple, long rawHandle) {
        if (tuple != null && tuple.length > 2 && tuple[2] instanceof long[] handles
                && handles.length == 1 && handles[0] == rawHandle) {
            handles[0] = 0;
        }
    }

    static Optional<GetUsageAllocationResponse> parseGetUsageAllocationResponse(Object[] values) {
        Object[] tuple = requireTuple(values, 3, "native Get Usage Allocation response is invalid");
        long[] fields = requireLongArray(
                tuple[0], 1, "native Get Usage Allocation response is invalid");
        long[] responseTtlvHandle = requireLongArray(
                tuple[2], 1, "native Get Usage Allocation response is invalid");
        if (!responsePresent(fields)) {
            if (tuple[1] != null || responseTtlvHandle[0] != 0) {
                if (responseTtlvHandle[0] > 0) {
                    TtlvStructure.fromNative(responseTtlvHandle[0], null).close();
                }
                throw new InvalidInputException("native absent response contains fields");
            }
            return Optional.empty();
        }
        if (responseTtlvHandle[0] <= 0) {
            throw new InvalidInputException("native Get Usage Allocation response is invalid");
        }
        TtlvStructure responseTtlv = TtlvStructure.fromNative(responseTtlvHandle[0], null);
        try {
            return Optional.of(new GetUsageAllocationResponse(
                    identifierRecord(tuple[1]), responseTtlv));
        } catch (RuntimeException exception) {
            responseTtlv.close();
            throw exception;
        }
    }

    static Optional<CreateKeyPairResponse> parseCreateKeyPairResponse(Object[] values) {
        Object[] tuple = requireTuple(values, 3, "native Create Key Pair response is invalid");
        long[] fields = requireLongArray(tuple[0], 1);
        if (fields[0] == 0) {
            return Optional.empty();
        }
        requirePresent(fields[0]);
        return Optional.of(new CreateKeyPairResponse(identifierRecord(tuple[1]), identifierRecord(tuple[2])));
    }

    static Optional<List<UniqueIdentifier>> parseCreateSplitKeyUniqueIdentifiers(Object[] values) {
        Object[] tuple = requireTuple(values, 2, "native Create Split Key response is invalid");
        long[] fields = requireLongArray(tuple[0], 2);
        if (fields[0] == 0) {
            return Optional.empty();
        }
        requirePresent(fields[0]);
        if (fields[1] < 1 || !(tuple[1] instanceof Object[] identifiers)
                || identifiers.length != fields[1]) {
            throw new InvalidInputException("native Create Split Key identifiers are invalid");
        }
        List<UniqueIdentifier> result = new ArrayList<>(identifiers.length);
        for (Object identifier : identifiers) {
            result.add(identifierRecord(identifier));
        }
        return Optional.of(List.copyOf(result));
    }

    private static UniqueIdentifier identifierRecord(Object value) {
        if (!(value instanceof Object[] values) || values.length != 2) {
            throw new InvalidInputException("native Unique Identifier is invalid");
        }
        long[] fields = requireLongArray(values[0], 4);
        requirePresent(fields[0]);
        byte[] text = values[1] == null ? null : requireBytes(values[1]);
        return identifier(fields[1], fields[2], fields[3], text);
    }

    private static UniqueIdentifier identifier(long kind, long enumeration, long integer, Object value) {
        byte[] text = value == null ? null : requireBytes(value);
        return UniqueIdentifier.fromNative(kind, text, enumeration, integer);
    }

    private static long[] requireLongArray(Object value, int length) {
        return requireLongArray(value, length, "native creation response fields are invalid");
    }

    private static long[] requireLongArray(Object value, int length, String message) {
        if (!(value instanceof long[] values) || values.length != length) {
            throw new InvalidInputException(message);
        }
        return values;
    }

    private static Object[] requireTuple(Object[] values, int length, String message) {
        if (values == null || values.length != length) {
            throw new InvalidInputException(message);
        }
        return values;
    }

    private static byte[] requireBytes(Object value) {
        if (!(value instanceof byte[] bytes)) {
            throw new InvalidInputException("native byte field is invalid");
        }
        return bytes;
    }

    private static void requirePresent(long value) {
        if (value != 1) {
            throw new InvalidInputException("native response presence flag is invalid");
        }
    }

    /** Returns the copied Unique Batch Item ID, when the server supplied one. */
    public Optional<byte[]> uniqueBatchItemId() {
        NativeExtensionRegistry.ensureLoaded();
        byte[] value = handle.withValue(NativeExtensionRegistry::clientBatchItemResponseUniqueBatchItemId);
        return value == null ? Optional.empty() : Optional.of(value.clone());
    }

    /** Returns the exact pending correlation bytes, when this result is Pending. */
    public Optional<byte[]> pendingCorrelationValue() {
        NativeExtensionRegistry.ensureLoaded();
        byte[] value = handle.withValue(
                NativeExtensionRegistry::clientBatchItemResponsePendingCorrelationValue);
        return value == null ? Optional.empty() : Optional.of(value.clone());
    }

    /**
     * Returns the ordered Protocol Version list for a successful Discover Versions result.
     * An empty present list is distinct from Pending, failure, or another operation.
     */
    public Optional<List<ProtocolVersion>> supportedVersions() {
        NativeExtensionRegistry.ensureLoaded();
        long[] values = handle.withValue(
                NativeExtensionRegistry::clientBatchItemResponseDiscoverVersionsSupportedVersions);
        return parseSupportedVersions(values);
    }

    static Optional<List<ProtocolVersion>> parseSupportedVersions(long[] values) {
        if (values == null) {
            return Optional.empty();
        }
        if ((values.length & 1) != 0) {
            throw new InvalidInputException("native Discover Versions list is invalid");
        }
        List<ProtocolVersion> versions = new ArrayList<>(values.length / 2);
        for (int index = 0; index < values.length; index += 2) {
            versions.add(new ProtocolVersion(
                    checkedVersionComponent(values[index]),
                    checkedVersionComponent(values[index + 1])));
        }
        return Optional.of(List.copyOf(versions));
    }

    private long[] metadata() {
        NativeExtensionRegistry.ensureLoaded();
        long[] values = handle.withValue(NativeExtensionRegistry::clientBatchItemResponseMetadata);
        return parseMetadata(values);
    }

    static long[] parseMetadata(long[] values) {
        return requireLongArray(values, 4, "native batch result metadata is invalid");
    }

    private static int checkedVersionComponent(long value) {
        if (value < Integer.MIN_VALUE || value > Integer.MAX_VALUE) {
            throw new InvalidInputException("native Discover Versions component is invalid");
        }
        return (int) value;
    }

    @Override
    public void close() {
        handle.close();
    }
}
