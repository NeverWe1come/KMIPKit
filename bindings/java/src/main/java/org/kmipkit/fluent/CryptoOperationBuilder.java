package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.FluentBatchItemAdapter;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.protocol.CryptographicParameters;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;

/** Shared native ownership and one-exchange lifecycle for typed crypto builders. */
abstract class CryptoOperationBuilder<B extends CryptoOperationBuilder<B>> implements AutoCloseable {
    static final int NATIVE_OPERATION_ENCRYPT = 0;
    static final int NATIVE_OPERATION_DECRYPT = 1;
    static final int NATIVE_OPERATION_SIGN = 2;
    static final int NATIVE_OPERATION_SIGNATURE_VERIFY = 3;
    static final int NATIVE_OPERATION_MAC = 4;
    static final int NATIVE_OPERATION_MAC_VERIFY = 5;
    static final int NATIVE_OPERATION_HASH = 6;

    static final int DATA = 0;
    static final int IV_COUNTER_NONCE = 1;
    static final int AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA = 2;
    static final int AUTHENTICATED_ENCRYPTION_TAG = 3;
    static final int DIGESTED_DATA = 4;
    static final int SIGNATURE_DATA = 5;
    static final int MAC_DATA = 6;
    static final int CORRELATION_VALUE = 7;
    static final int INITIAL_INDICATOR = 0;
    static final int FINAL_INDICATOR = 1;

    private final NativeHandle handle;
    private final KmipClient client;
    private RequestOptions options;
    private Long asynchronousIndicator;

    CryptoOperationBuilder(KmipClient client, int operation) {
        this.client = client;
        NativeFluentRegistry.ensureLoaded();
        handle = NativeHandle.owned(
                NativeFluentRegistry.cryptoOperationBuilderCreate(operation),
                NativeFluentRegistry.CRYPTO_OPERATION_BUILDER);
    }

    final void setUniqueIdentifier(int kind, byte[] text, long number) {
        handle.withValue(builder -> {
            NativeFluentRegistry.cryptoOperationBuilderSetUniqueIdentifier(
                    builder, kind, text, number);
            return null;
        });
    }

    final void setBytes(int field, byte[] value) {
        if (value == null) throw new InvalidInputException("cryptographic input bytes are invalid");
        handle.withValue(builder -> {
            NativeFluentRegistry.cryptoOperationBuilderSetBytes(builder, field, value);
            return null;
        });
    }

    final void setDataValue(TtlvValue value) {
        if (value == null) throw new InvalidInputException("cryptographic Data value is invalid");
        NativeHandle.withValues(new NativeHandle[] {handle, value.nativeHandleOwner()}, values -> {
            NativeFluentRegistry.cryptoOperationBuilderSetDataValue(values[0], values[1]);
            return null;
        });
    }

    final void setCryptographicParameters(CryptographicParameters parameters) {
        if (parameters == null) {
            throw new InvalidInputException("Cryptographic Parameters are invalid");
        }
        NativeHandle.withValues(
                new NativeHandle[] {handle, parameters.nativeHandleOwner()}, values -> {
                    NativeFluentRegistry.cryptoOperationBuilderSetCryptographicParameters(
                            values[0], values[1]);
                    return null;
                });
    }

    final void setCryptographicParametersStructure(TtlvStructure parameters) {
        if (parameters == null) {
            throw new InvalidInputException("Cryptographic Parameters structure is invalid");
        }
        NativeHandle.withValues(new NativeHandle[] {handle, parameters.nativeHandleOwner()}, values -> {
            NativeFluentRegistry.cryptoOperationBuilderSetCryptographicParametersStructure(
                    values[0], values[1]);
            return null;
        });
    }

    final void setIndicator(int indicator, boolean value) {
        handle.withValue(builder -> {
            NativeFluentRegistry.cryptoOperationBuilderSetIndicator(builder, indicator, value);
            return null;
        });
    }

    /** Sets exchange-local transport options without changing the Request Header. */
    public final B withOptions(RequestOptions value) {
        if (value == null) throw new InvalidInputException("request options are invalid");
        options = value;
        return fluentSelf();
    }

    /** Sets the explicit Request Header Asynchronous Indicator for this exchange. */
    public final B asynchronousIndicator(long value) {
        if (value < 0 || value > 0xffff_ffffL) {
            throw new InvalidInputException("asynchronous indicator is outside uint32");
        }
        asynchronousIndicator = value;
        return fluentSelf();
    }

    /** Builds the existing typed batch item without consuming this builder. */
    public final ClientBatchItem build() {
        long item = handle.withValue(NativeFluentRegistry::cryptoOperationBuilderBuildBatchItem);
        return FluentBatchItemAdapter.takeOwnership(item);
    }

    /** Alias for {@link #build()}. */
    public final ClientBatchItem toBatchItem() {
        return build();
    }

    /** Executes exactly one typed exchange and never polls or retries it. */
    public final ClientBatchResponse execute(CodecLimits limits)
            throws org.kmipkit.ClientOperationException {
        if (limits == null) throw new InvalidInputException("codec limits are invalid");
        if (client == null) {
            throw new InvalidInputException("client is required to execute this builder");
        }
        try (ClientBatch batch = ClientBatch.create()) {
            batch.add(build());
            if (asynchronousIndicator != null) {
                batch.asynchronousIndicator(asynchronousIndicator);
            }
            return options == null ? client.execute(batch, limits)
                    : client.executeWithOptions(batch, limits, options);
        }
    }

    @Override
    public final void close() {
        handle.close();
    }

    @SuppressWarnings("unchecked")
    private B fluentSelf() {
        return (B) this;
    }
}
