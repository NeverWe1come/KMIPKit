package org.kmipkit.extensions;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Optional;
import java.util.OptionalInt;
import java.util.OptionalLong;
import java.util.function.Function;
import java.util.function.Supplier;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;
import org.jspecify.annotations.Nullable;

/** Common, operation-independent metadata for one response batch item. */
public final class ClientBatchItemResult implements AutoCloseable {
    private static final int MAX_NATIVE_ARRAY_ELEMENTS = 100_000;

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
        try {
            return parseDelegatedLoginResponseAfterNativeReturn(
                    values, () -> ClientBatchItemResult::parseDelegatedLoginResponse);
        } finally {
            cleanupDelegatedLoginResponseValues(values);
        }
    }

    /** Returns the typed Login result and its raw-preserving Ticket when present. */
    public Optional<LoginResponse> loginResponse() {
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(NativeExtensionRegistry::clientBatchItemResponseLogin);
        try {
            return parseLoginResponseAfterNativeReturn(
                    values, () -> ClientBatchItemResult::parseLoginResponse);
        } finally {
            cleanupLoginResponseValues(values);
        }
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

    /** Returns the successful Obtain Lease fields and an independently owned ordered TTLV copy. */
    public Optional<ObtainLeaseResponse> obtainLeaseResponse() {
        if (resultStatus() != 0) return Optional.empty();
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(
                NativeExtensionRegistry::clientBatchItemResponseObtainLease);
        return parseObtainLeaseResponse(values);
    }

    /**
     * Returns the generated identifier and complete generic TTLV from a successful Import.
     * The returned response owns an independent TTLV copy; callers must close it when finished.
     */
    public Optional<ImportResponse> importResponse() {
        if (resultStatus() != 0) return Optional.empty();
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(NativeExtensionRegistry::clientBatchItemResponseImport);
        return parseImportResponse(values);
    }

    /** Returns the joined identifier and its full ordered response TTLV on Success. */
    public Optional<JoinSplitKeyResponse> joinSplitKeyResponse() {
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(
                NativeExtensionRegistry::clientBatchItemResponseJoinSplitKey);
        return parseJoinSplitKeyResponse(values);
    }

    /** Returns the typed Get result, with an independently owned secret-bearing Any Object copy. */
    public Optional<GetResponse> getResponse() {
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(NativeExtensionRegistry::clientBatchItemResponseGet);
        return parseGetResponse(values);
    }

    /** Returns the ordered Locate identifiers and optional Located Items value. */
    public Optional<LocateResponse> locateResponse() {
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(NativeExtensionRegistry::clientBatchItemResponseLocate);
        return parseLocateResponse(values);
    }

    /** Returns the required Constraints payload in an owned complete response copy on Success. */
    public Optional<GetConstraintsResponse> getConstraintsResponse() {
        if (resultStatus() != 0) return Optional.empty();
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(
                NativeExtensionRegistry::clientBatchItemResponseGetConstraints);
        return parseGetConstraintsResponse(values);
    }

    /** Returns the complete typed Interop result for Success, Failure, or Pending. */
    public Optional<InteropResponse> interopResponse() {
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(NativeExtensionRegistry::clientBatchItemResponseInterop);
        return parseInteropResponse(values);
    }

    /** Returns the complete typed Log result for Success, Failure, or Pending. */
    public Optional<LogResponse> logResponse() {
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(NativeExtensionRegistry::clientBatchItemResponseLog);
        return parseLogResponse(values);
    }

    /** Returns a typed Export response for both Success and KMIP Failure outcomes. */
    public Optional<ExportResponse> exportResponse() {
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(NativeExtensionRegistry::clientBatchItemResponseExport);
        return parseExportResponse(values);
    }

    /**
     * Returns the resolved identifier and complete generic TTLV from a successful Derive Key
     * response. The returned response owns an independent TTLV copy; callers must close it.
     */
    public Optional<DeriveKeyResponse> deriveKeyResponse() {
        if (resultStatus() != 0) return Optional.empty();
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(NativeExtensionRegistry::clientBatchItemResponseDeriveKey);
        return Optional.of(parseDeriveKeyResponse(values));
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

    /** Successful Get fields and the independently owned secret-bearing Any Object. */
    public static final class GetResponse implements AutoCloseable {
        private final OptionalLong objectTypeRaw;
        private final Optional<UniqueIdentifier> uniqueIdentifier;
        private final @Nullable TtlvStructure object;

        private GetResponse(OptionalLong objectTypeRaw,
                Optional<UniqueIdentifier> uniqueIdentifier, @Nullable TtlvStructure object) {
            this.objectTypeRaw = objectTypeRaw;
            this.uniqueIdentifier = uniqueIdentifier;
            this.object = object;
        }

        /** Returns the raw unsigned Object Type only when the server returned Success. */
        public OptionalLong objectTypeRaw() { return objectTypeRaw; }
        /** Returns the response Unique Identifier only when the server returned Success. */
        public Optional<UniqueIdentifier> uniqueIdentifier() { return uniqueIdentifier; }
        /** Returns the caller-owned opaque Any Object Structure on Success. */
        public Optional<TtlvStructure> object() { return Optional.ofNullable(object); }

        @Override public void close() {
            if (object != null) object.close();
        }

        @Override public String toString() {
            return "GetResponse[objectTypeRaw=" + objectTypeRaw
                    + ", uniqueIdentifier=" + (uniqueIdentifier.isPresent() ? "redacted" : "absent")
                    + ", object=" + (object == null ? "absent" : "redacted") + "]";
        }
    }

    /** Locate's optional count and response identifiers in their original wire order. */
    public static final class LocateResponse {
        private final OptionalInt locatedItems;
        private final List<UniqueIdentifier> uniqueIdentifiers;

        private LocateResponse(OptionalInt locatedItems, List<UniqueIdentifier> uniqueIdentifiers) {
            this.locatedItems = locatedItems;
            this.uniqueIdentifiers = List.copyOf(uniqueIdentifiers);
        }

        /** Returns the optional server-reported Located Items Integer. */
        public OptionalInt locatedItems() { return locatedItems; }
        /** Returns all response identifiers, retaining repetitions and server order. */
        public List<UniqueIdentifier> uniqueIdentifiers() { return uniqueIdentifiers; }
    }

    /** Typed KMIP Export result and an owned copy of its complete ordered response TTLV. */
    public static final class ExportResponse implements AutoCloseable {
        private final OptionalLong objectTypeRaw;
        private final Optional<UniqueIdentifier> uniqueIdentifier;
        private final @Nullable TtlvStructure attributes;
        private final @Nullable TtlvStructure anyObject;
        private final TtlvStructure responseTtlv;

        private ExportResponse(OptionalLong objectTypeRaw,
                Optional<UniqueIdentifier> uniqueIdentifier,
                @Nullable TtlvStructure attributes,
                @Nullable TtlvStructure anyObject,
                TtlvStructure responseTtlv) {
            this.objectTypeRaw = objectTypeRaw;
            this.uniqueIdentifier = uniqueIdentifier;
            this.attributes = attributes;
            this.anyObject = anyObject;
            this.responseTtlv = responseTtlv;
        }

        /** Returns the raw unsigned Object Type only when the server returned Success. */
        public OptionalLong objectTypeRaw() { return objectTypeRaw; }
        /** Returns the exact Unique Identifier wire value only on Success. */
        public Optional<UniqueIdentifier> uniqueIdentifier() { return uniqueIdentifier; }
        /** Returns the required Attributes on Success, or empty for a KMIP Failure. */
        public Optional<TtlvStructure> attributes() { return Optional.ofNullable(attributes); }
        /** Returns the original Any Object item, including its outer tag and ordered contents. */
        public Optional<TtlvStructure> anyObject() { return Optional.ofNullable(anyObject); }
        /** Returns the complete ordered response TTLV, including KMIP Failure responses. */
        public TtlvStructure responseTtlv() { return responseTtlv; }

        @Override
        public void close() {
            if (attributes != null) attributes.close();
            if (anyObject != null) anyObject.close();
            responseTtlv.close();
        }

        @Override
        public String toString() {
            return "ExportResponse[objectTypeRaw=" + objectTypeRaw
                    + ", uniqueIdentifier=" + (uniqueIdentifier.isPresent() ? "redacted" : "absent")
                    + ", attributes=" + (attributes == null ? "absent" : "present")
                    + ", anyObject=" + (anyObject == null ? "absent" : "present")
                    + ", responseTtlv=redacted]";
        }
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

    /** Login Ticket fields and the independently owned generic Ticket TTLV copy. */
    public static final class LoginResponse implements AutoCloseable {
        private final OptionalLong ticketTypeRaw;
        private @Nullable byte[] ticketValue;
        private @Nullable TtlvStructure ticketTtlv;

        private LoginResponse(OptionalLong ticketTypeRaw,
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
            return "LoginResponse[ticketTypeRaw=" + ticketTypeRaw
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

    /** Successful Obtain Lease response with exact Table 269 values and its ordered TTLV copy. */
    public static final class ObtainLeaseResponse implements AutoCloseable {
        private final UniqueIdentifier uniqueIdentifier;
        private final long leaseTime;
        private final long lastChangeDate;
        private final TtlvStructure responseTtlv;

        private ObtainLeaseResponse(UniqueIdentifier uniqueIdentifier, long leaseTime,
                long lastChangeDate, TtlvStructure responseTtlv) {
            this.uniqueIdentifier = uniqueIdentifier;
            this.leaseTime = leaseTime;
            this.lastChangeDate = lastChangeDate;
            this.responseTtlv = responseTtlv;
        }

        public UniqueIdentifier uniqueIdentifier() { return uniqueIdentifier; }
        /** Returns the exact unsigned Lease Time Interval, including zero. */
        public long leaseTime() { return leaseTime; }
        /** Returns Last Change Date as signed KMIP Date-Time seconds since the epoch. */
        public long lastChangeDate() { return lastChangeDate; }
        /** Returns the complete ordered response structure owned by this result. */
        public TtlvStructure responseTtlv() { return responseTtlv; }
        @Override public void close() { responseTtlv.close(); }
        @Override public String toString() {
            return "ObtainLeaseResponse[uniqueIdentifier=redacted, leaseTime=" + leaseTime
                    + ", lastChangeDate=" + lastChangeDate + ", responseTtlv=preserved]";
        }
    }

    /** Successful Import response with its independently owned complete generic TTLV. */
    public static final class ImportResponse implements AutoCloseable {
        private final UniqueIdentifier uniqueIdentifier;
        private final TtlvStructure responseTtlv;

        private ImportResponse(UniqueIdentifier uniqueIdentifier, TtlvStructure responseTtlv) {
            this.uniqueIdentifier = uniqueIdentifier;
            this.responseTtlv = responseTtlv;
        }

        public UniqueIdentifier uniqueIdentifier() { return uniqueIdentifier; }

        /** Returns the full ordered response structure, including unknown fields. */
        public TtlvStructure responseTtlv() { return responseTtlv; }

        @Override public void close() { responseTtlv.close(); }

        @Override public String toString() {
            return "ImportResponse[uniqueIdentifier=redacted, responseTtlv=preserved]";
        }
    }

    /** Successful Join Split Key response with its complete ordered generic TTLV copy. */
    public static final class JoinSplitKeyResponse implements AutoCloseable {
        private final UniqueIdentifier uniqueIdentifier;
        private final TtlvStructure responseTtlv;

        private JoinSplitKeyResponse(UniqueIdentifier uniqueIdentifier, TtlvStructure responseTtlv) {
            this.uniqueIdentifier = uniqueIdentifier;
            this.responseTtlv = responseTtlv;
        }

        public UniqueIdentifier uniqueIdentifier() { return uniqueIdentifier; }
        /** Returns the full ordered response structure, including unknown fields. */
        public TtlvStructure responseTtlv() { return responseTtlv; }
        @Override public void close() { responseTtlv.close(); }
        @Override public String toString() {
            return "JoinSplitKeyResponse[uniqueIdentifier=redacted, responseTtlv=preserved]";
        }
    }

    /** Successful Get Constraints response with its complete ordered generic TTLV copy. */
    public static final class GetConstraintsResponse implements AutoCloseable {
        private final TtlvStructure responseTtlv;

        private GetConstraintsResponse(TtlvStructure responseTtlv) {
            this.responseTtlv = responseTtlv;
        }

        /** Returns the full ordered response, including the required Constraints structure. */
        public TtlvStructure responseTtlv() { return responseTtlv; }

        @Override public void close() { responseTtlv.close(); }

        @Override public String toString() { return "GetConstraintsResponse[fields=redacted]"; }
    }

    /** Interop response with its complete ordered generic TTLV for every valid result status. */
    public static final class InteropResponse implements AutoCloseable {
        private final TtlvStructure responseTtlv;

        private InteropResponse(TtlvStructure responseTtlv) {
            this.responseTtlv = responseTtlv;
        }

        /** Returns the full ordered response structure, including unknown fields. */
        public TtlvStructure responseTtlv() { return responseTtlv; }

        @Override public void close() { responseTtlv.close(); }

        @Override public String toString() { return "InteropResponse[responseTtlv=redacted]"; }
    }

    /** Log response with its complete ordered generic TTLV for every valid result status. */
    public static final class LogResponse implements AutoCloseable {
        private final TtlvStructure responseTtlv;

        private LogResponse(TtlvStructure responseTtlv) {
            this.responseTtlv = responseTtlv;
        }

        /** Returns the full ordered response structure, including unknown fields. */
        public TtlvStructure responseTtlv() { return responseTtlv; }

        @Override public void close() { responseTtlv.close(); }

        @Override public String toString() { return "LogResponse[responseTtlv=redacted]"; }
    }

    /** Successful Derive Key response with its owned complete generic TTLV. */
    public static final class DeriveKeyResponse implements AutoCloseable {
        private final UniqueIdentifier uniqueIdentifier;
        private final TtlvStructure responseTtlv;

        private DeriveKeyResponse(UniqueIdentifier uniqueIdentifier, TtlvStructure responseTtlv) {
            this.uniqueIdentifier = uniqueIdentifier;
            this.responseTtlv = responseTtlv;
        }

        public UniqueIdentifier uniqueIdentifier() { return uniqueIdentifier; }

        /** Returns the complete ordered response structure owned by this result. */
        public TtlvStructure responseTtlv() { return responseTtlv; }

        @Override public void close() { responseTtlv.close(); }

        @Override public String toString() {
            return "DeriveKeyResponse[uniqueIdentifier=redacted]";
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
        try {
            return parserFactory.get().apply(values);
        } finally {
            cleanupDelegatedLoginResponseValues(values);
        }
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
                ownedTtlv.close(); // COVERAGE-EXCLUDE: Only allocation failure before response construction leaves this wrapper owned here.
            } else if (rawHandle > 0) {
                clearDelegatedLoginTtlvHandle(tuple, rawHandle);
                NativeExtensionRegistry.release(NativeExtensionRegistry.TTLV_STRUCTURE, rawHandle);
            }
        }
    }

    static Optional<LoginResponse> parseLoginResponse(Object[] values) {
        return parseLoginResponse(values, Optional::of);
    }

    static Optional<LoginResponse> parseLoginResponseAfterNativeReturn(
            Object[] values,
            Supplier<Function<Object[], Optional<LoginResponse>>> parserFactory) {
        try {
            return parserFactory.get().apply(values);
        } finally {
            cleanupLoginResponseValues(values);
        }
    }

    private static void cleanupLoginResponseValues(Object[] values) {
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

    static Optional<LoginResponse> parseLoginResponse(
            Object[] values,
            Function<LoginResponse, Optional<LoginResponse>> resultTransfer) {
        Object[] tuple = values;
        long rawHandle = 0;
        TtlvStructure ownedTtlv = null;
        LoginResponse response = null;
        byte[] ownedValue = null;
        boolean transferred = false;
        try {
            tuple = requireTuple(values, 3, "native Login response is invalid");
            if (tuple[2] instanceof long[] handleValues && handleValues.length == 1) {
                rawHandle = handleValues[0];
            }
            long[] fields = requireLongArray(tuple[0], 3, "native Login response is invalid");
            long[] ticketTtlvHandle = requireLongArray(
                    tuple[2], 1, "native Login response is invalid");
            rawHandle = ticketTtlvHandle[0];
            boolean responsePresent = responseFlag(fields[0]);
            boolean ticketPresent = responseFlag(fields[1]);
            if (!ticketPresent) {
                if (fields[2] != 0 || tuple[1] != null || rawHandle != 0) {
                    throw new InvalidInputException("native absent Login Ticket contains fields");
                }
                if (!responsePresent) return Optional.empty();
                response = new LoginResponse(OptionalLong.empty(), null, null);
                Optional<LoginResponse> result = resultTransfer.apply(response);
                transferred = true;
                response = null;
                return result;
            }
            if (!responsePresent || fields[2] < 0 || fields[2] > 0xffff_ffffL
                    || rawHandle <= 0) {
                throw new InvalidInputException("native Login Ticket is invalid");
            }
            ownedValue = requireBytes(tuple[1]);
            ownedTtlv = TtlvStructure.fromNative(rawHandle, null);
            clearLoginTtlvHandle(tuple, rawHandle);
            rawHandle = 0;
            response = new LoginResponse(OptionalLong.of(fields[2]), ownedValue, ownedTtlv);
            Arrays.fill(ownedValue, (byte) 0);
            ownedTtlv = null;
            Optional<LoginResponse> result = resultTransfer.apply(response);
            transferred = true;
            response = null;
            return result;
        } finally {
            if (response != null) response.close();
            if (ownedValue != null) Arrays.fill(ownedValue, (byte) 0);
            if (!transferred && tuple != null && tuple.length > 1 && tuple[1] instanceof byte[] value
                    && value != ownedValue) {
                Arrays.fill(value, (byte) 0);
            }
            if (ownedTtlv != null) {
                ownedTtlv.close(); // COVERAGE-EXCLUDE: Only allocation failure before response construction leaves this wrapper owned here.
            } else if (rawHandle > 0) {
                clearLoginTtlvHandle(tuple, rawHandle);
                NativeExtensionRegistry.release(NativeExtensionRegistry.TTLV_STRUCTURE, rawHandle);
            }
        }
    }

    private static void clearLoginTtlvHandle(Object[] tuple, long rawHandle) {
        if (tuple != null && tuple.length > 2 && tuple[2] instanceof long[] handles
                && handles.length == 1 && handles[0] == rawHandle) {
            handles[0] = 0;
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

    static Optional<ObtainLeaseResponse> parseObtainLeaseResponse(Object[] values) {
        Object[] tuple = requireTuple(values, 3, "native Obtain Lease response is invalid");
        long[] fields = requireLongArray(
                tuple[0], 3, "native Obtain Lease response is invalid");
        long[] responseTtlvHandle = requireLongArray(
                tuple[2], 1, "native Obtain Lease response is invalid");
        if (!responsePresent(fields)) {
            if (tuple[1] != null || fields[1] != 0 || fields[2] != 0
                    || responseTtlvHandle[0] != 0) {
                if (responseTtlvHandle[0] > 0) {
                    TtlvStructure.fromNative(responseTtlvHandle[0], null).close();
                }
                throw new InvalidInputException("native absent Obtain Lease response contains fields");
            }
            return Optional.empty();
        }
        if (tuple[1] == null || responseTtlvHandle[0] <= 0
                || fields[1] < 0 || fields[1] > 0xFFFF_FFFFL) {
            if (responseTtlvHandle[0] > 0) {
                TtlvStructure.fromNative(responseTtlvHandle[0], null).close();
            }
            throw new InvalidInputException("native successful Obtain Lease response is incomplete");
        }
        TtlvStructure responseTtlv = TtlvStructure.fromNative(responseTtlvHandle[0], null);
        try {
            return Optional.of(new ObtainLeaseResponse(
                    identifierRecord(tuple[1]), fields[1], fields[2], responseTtlv));
        } catch (RuntimeException exception) {
            responseTtlv.close();
            throw exception;
        }
    }

    static Optional<ImportResponse> parseImportResponse(Object[] values) {
        Object[] tuple = requireTuple(values, 3, "native Import response is invalid");
        long[] fields = requireLongArray(tuple[0], 1, "native Import response is invalid");
        long[] responseTtlvHandle = requireLongArray(
                tuple[2], 1, "native Import response is invalid");
        if (!responsePresent(fields)) {
            if (tuple[1] != null || responseTtlvHandle[0] != 0) {
                if (responseTtlvHandle[0] > 0) {
                    TtlvStructure.fromNative(responseTtlvHandle[0], null).close();
                }
                throw new InvalidInputException("native absent Import response contains fields");
            }
            return Optional.empty();
        }
        if (tuple[1] == null || responseTtlvHandle[0] <= 0) {
            if (responseTtlvHandle[0] > 0) {
                TtlvStructure.fromNative(responseTtlvHandle[0], null).close();
            }
            throw new InvalidInputException("native successful Import response is incomplete");
        }
        TtlvStructure responseTtlv = TtlvStructure.fromNative(responseTtlvHandle[0], null);
        try {
            return Optional.of(new ImportResponse(identifierRecord(tuple[1]), responseTtlv));
        } catch (RuntimeException exception) {
            responseTtlv.close();
            throw exception;
        }
    }

    static Optional<JoinSplitKeyResponse> parseJoinSplitKeyResponse(Object[] values) {
        Object[] tuple = requireTuple(values, 3, "native Join Split Key response is invalid");
        long[] fields = requireLongArray(tuple[0], 1, "native Join Split Key response is invalid");
        long[] responseTtlvHandle = requireLongArray(
                tuple[2], 1, "native Join Split Key response is invalid");
        if (!responsePresent(fields)) {
            if (tuple[1] != null || responseTtlvHandle[0] != 0) {
                if (responseTtlvHandle[0] > 0) {
                    TtlvStructure.fromNative(responseTtlvHandle[0], null).close();
                }
                throw new InvalidInputException(
                        "native absent Join Split Key response contains fields");
            }
            return Optional.empty();
        }
        if (tuple[1] == null || responseTtlvHandle[0] <= 0) {
            if (responseTtlvHandle[0] > 0) {
                TtlvStructure.fromNative(responseTtlvHandle[0], null).close();
            }
            throw new InvalidInputException("native successful Join Split Key response is incomplete");
        }
        TtlvStructure responseTtlv = TtlvStructure.fromNative(responseTtlvHandle[0], null);
        try {
            return Optional.of(new JoinSplitKeyResponse(identifierRecord(tuple[1]), responseTtlv));
        } catch (RuntimeException exception) {
            responseTtlv.close();
            throw exception;
        }
    }

    static Optional<GetResponse> parseGetResponse(Object[] values) {
        Object[] tuple = requireTuple(values, 3, "native Get response is invalid");
        long[] fields = requireLongArray(tuple[0], 3, "native Get response is invalid");
        long[] objectHandle = requireLongArray(tuple[2], 1, "native Get response is invalid");
        long ownedHandle = objectHandle[0];
        TtlvStructure object = null;
        try {
            if ((fields[0] != 0 && fields[0] != 1)
                    || (fields[1] != 0 && fields[1] != 1)
                    || (fields[0] == 0 && (fields[1] != 0 || fields[2] != 0
                            || tuple[1] != null || ownedHandle != 0))) {
                throw new InvalidInputException("native Get response presence is invalid");
            }
            if (fields[0] == 0) return Optional.empty();
            if (fields[1] == 0) {
                if (fields[2] != 0 || tuple[1] != null || ownedHandle != 0) {
                    throw new InvalidInputException("native failed Get response contains payload fields");
                }
                return Optional.of(new GetResponse(OptionalLong.empty(), Optional.empty(), null));
            }
            if (tuple[1] == null || ownedHandle <= 0) {
                throw new InvalidInputException("native successful Get response is incomplete");
            }
            object = TtlvStructure.fromNative(ownedHandle, null);
            UniqueIdentifier identifier = identifierRecord(tuple[1]);
            return Optional.of(new GetResponse(
                    OptionalLong.of(fields[2]), Optional.of(identifier), object));
        } catch (RuntimeException exception) {
            if (object != null) object.close();
            else if (ownedHandle > 0) TtlvStructure.fromNative(ownedHandle, null).close();
            throw exception;
        }
    }

    static Optional<LocateResponse> parseLocateResponse(Object[] values) {
        Object[] tuple = requireTuple(values, 2, "native Locate response is invalid");
        long[] fields = requireLongArray(tuple[0], 5, "native Locate response is invalid");
        if (!(tuple[1] instanceof Object[] identifiers)) {
            throw new InvalidInputException("native Locate identifiers are invalid");
        }
        if ((fields[0] != 0 && fields[0] != 1)
                || (fields[1] != 0 && fields[1] != 1)
                || (fields[2] != 0 && fields[2] != 1)
                || fields[3] < Integer.MIN_VALUE || fields[3] > Integer.MAX_VALUE
                || fields[4] < 0 || fields[4] > Integer.MAX_VALUE) {
            throw new InvalidInputException("native Locate response fields are invalid");
        }
        if (fields[0] == 0) {
            if (fields[1] != 0 || fields[2] != 0 || fields[3] != 0 || fields[4] != 0
                    || identifiers.length != 0) {
                throw new InvalidInputException("native absent Locate response contains fields");
            }
            return Optional.empty();
        }
        if (fields[1] == 0) {
            if (fields[2] != 0 || fields[3] != 0 || fields[4] != 0 || identifiers.length != 0) {
                throw new InvalidInputException("native failed Locate response contains payload fields");
            }
            return Optional.of(new LocateResponse(OptionalInt.empty(), List.of()));
        }
        if (fields[4] != identifiers.length) {
            throw new InvalidInputException("native Locate identifier count is inconsistent");
        }
        if (identifiers.length > MAX_NATIVE_ARRAY_ELEMENTS) {
            throw new ResourceLimitException("Locate response identifier count exceeds the configured limit");
        }
        OptionalInt locatedItems = fields[2] == 1
                ? OptionalInt.of((int) fields[3]) : OptionalInt.empty();
        if (fields[2] == 0 && fields[3] != 0) {
            throw new InvalidInputException("native omitted Located Items contains a value");
        }
        List<UniqueIdentifier> copiedIdentifiers = new ArrayList<>(identifiers.length);
        for (Object identifier : identifiers) copiedIdentifiers.add(identifierRecord(identifier));
        return Optional.of(new LocateResponse(locatedItems, copiedIdentifiers));
    }

    static Optional<GetConstraintsResponse> parseGetConstraintsResponse(Object[] values) {
        Object[] tuple = requireTuple(values, 2, "native Get Constraints response is invalid");
        long[] fields = requireLongArray(
                tuple[0], 1, "native Get Constraints response is invalid");
        long[] handles = requireLongArray(
                tuple[1], 1, "native Get Constraints response is invalid");
        if (!responsePresent(fields)) {
            if (handles[0] != 0) {
                TtlvStructure.fromNative(handles[0], null).close();
                throw new InvalidInputException("native absent response contains fields");
            }
            return Optional.empty();
        }
        if (handles[0] <= 0) {
            throw new InvalidInputException("native Get Constraints response is invalid");
        }
        return Optional.of(new GetConstraintsResponse(TtlvStructure.fromNative(handles[0], null)));
    }

    static Optional<InteropResponse> parseInteropResponse(Object[] values) {
        Object[] tuple = requireTuple(values, 2, "native Interop response is invalid");
        long[] fields = requireLongArray(tuple[0], 1, "native Interop response is invalid");
        long[] handles = requireLongArray(tuple[1], 1, "native Interop response is invalid");
        if (!responsePresent(fields)) {
            if (handles[0] != 0) {
                TtlvStructure.fromNative(handles[0], null).close();
                throw new InvalidInputException("native absent Interop response contains fields");
            }
            return Optional.empty();
        }
        if (handles[0] <= 0) {
            throw new InvalidInputException("native Interop response is invalid");
        }
        return Optional.of(new InteropResponse(TtlvStructure.fromNative(handles[0], null)));
    }

    static Optional<LogResponse> parseLogResponse(Object[] values) {
        Object[] tuple = requireTuple(values, 2, "native Log response is invalid");
        long[] fields = requireLongArray(tuple[0], 1, "native Log response is invalid");
        long[] handles = requireLongArray(tuple[1], 1, "native Log response is invalid");
        if (!responsePresent(fields)) {
            if (handles[0] != 0) {
                TtlvStructure.fromNative(handles[0], null).close();
                throw new InvalidInputException("native absent Log response contains fields");
            }
            return Optional.empty();
        }
        if (handles[0] <= 0) {
            throw new InvalidInputException("native Log response is invalid");
        }
        return Optional.of(new LogResponse(TtlvStructure.fromNative(handles[0], null)));
    }

    static Optional<ExportResponse> parseExportResponse(Object[] values) {
        Object[] tuple = requireTuple(values, 3, "native Export response is invalid");
        long[] fields = requireLongArray(tuple[0], 3, "native Export response is invalid");
        long[] handles = requireLongArray(tuple[2], 3, "native Export response is invalid");
        long attributesHandle = handles[0];
        long anyObjectHandle = handles[1];
        long responseTtlvHandle = handles[2];
        TtlvStructure attributes = null;
        TtlvStructure anyObject = null;
        TtlvStructure responseTtlv = null;
        try {
            if (fields[0] == 0) {
                if (fields[1] != 0 || fields[2] != 0 || tuple[1] != null
                        || attributesHandle != 0 || anyObjectHandle != 0 || responseTtlvHandle != 0) {
                    throw new InvalidInputException("native absent Export response contains fields");
                }
                return Optional.empty();
            }
            requirePresent(fields[0]);
            if (fields[1] != 0 && fields[1] != 1) {
                throw new InvalidInputException("native Export payload presence is invalid");
            }
            if (responseTtlvHandle <= 0) {
                throw new InvalidInputException("native Export response TTLV is missing");
            }
            responseTtlv = TtlvStructure.fromNative(responseTtlvHandle, null);
            if (fields[1] == 0) {
                if (tuple[1] != null || attributesHandle != 0 || anyObjectHandle != 0) {
                    throw new InvalidInputException("native failed Export response contains payload fields");
                }
                return Optional.of(new ExportResponse(OptionalLong.empty(), Optional.empty(),
                        null, null, responseTtlv));
            }
            if (tuple[1] == null || attributesHandle <= 0 || anyObjectHandle <= 0) {
                throw new InvalidInputException("native successful Export response is incomplete");
            }
            UniqueIdentifier identifier = identifierRecord(tuple[1]);
            attributes = TtlvStructure.fromNative(attributesHandle, null);
            anyObject = TtlvStructure.fromNative(anyObjectHandle, null);
            return Optional.of(new ExportResponse(OptionalLong.of(fields[2]), Optional.of(identifier),
                    attributes, anyObject, responseTtlv));
        } catch (RuntimeException exception) {
            closeExportHandle(attributes, attributesHandle);
            closeExportHandle(anyObject, anyObjectHandle);
            closeExportHandle(responseTtlv, responseTtlvHandle);
            throw exception;
        }
    }

    private static void closeExportHandle(@Nullable TtlvStructure owned, long handle) {
        if (owned != null) owned.close();
        else if (handle > 0) TtlvStructure.fromNative(handle, null).close();
    }

    static DeriveKeyResponse parseDeriveKeyResponse(Object[] values) {
        Object[] tuple = requireTuple(values, 3, "native Derive Key response is invalid");
        long[] fields = requireLongArray(
                tuple[0], 1, "native Derive Key response is invalid");
        long[] responseTtlvHandle = requireLongArray(
                tuple[2], 1, "native Derive Key response is invalid");
        if (!responsePresent(fields)) {
            if (tuple[1] != null || responseTtlvHandle[0] != 0) {
                if (responseTtlvHandle[0] > 0) {
                    TtlvStructure.fromNative(responseTtlvHandle[0], null).close();
                }
                throw new InvalidInputException("native absent response contains fields");
            }
            throw new InvalidInputException(
                    "native Derive Key success response is missing its Unique Identifier");
        }
        if (responseTtlvHandle[0] <= 0) {
            throw new InvalidInputException("native Derive Key response is invalid");
        }
        TtlvStructure responseTtlv = TtlvStructure.fromNative(responseTtlvHandle[0], null);
        try {
            return new DeriveKeyResponse(identifierRecord(tuple[1]), responseTtlv);
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
