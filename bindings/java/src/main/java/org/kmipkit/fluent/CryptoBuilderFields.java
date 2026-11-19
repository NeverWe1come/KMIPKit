package org.kmipkit.fluent;

import java.nio.charset.StandardCharsets;

import org.kmipkit.InvalidInputException;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.protocol.CryptographicParameters;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;

/** Package-private capability interfaces keep each operation's public setters precise. */
/** Package-private casts used by the mixins without adding helper methods to builder APIs. */
final class CryptoBuilderCalls {
    private CryptoBuilderCalls() {}

    @SuppressWarnings("unchecked")
    static <B extends CryptoOperationBuilder<B>> B self(Object builder) {
        return (B) builder;
    }

    @SuppressWarnings("unchecked")
    static <B extends CryptoOperationBuilder<B>> CryptoOperationBuilder<B> builder(Object value) {
        return (CryptoOperationBuilder<B>) value;
    }
}

interface CryptoKeyIdentifier<B extends CryptoOperationBuilder<B>> {
    default B uniqueIdentifier(UniqueIdentifier value) {
        if (value == null) throw new InvalidInputException("Unique Identifier is invalid");
        int kind;
        byte[] text = null;
        long number = 0;
        switch (value.kind()) {
            case TEXT_STRING -> {
                kind = 1;
                text = value.text().getBytes(StandardCharsets.UTF_8);
            }
            case ENUMERATION -> {
                kind = 2;
                number = value.number();
            }
            case INTEGER -> {
                kind = 3;
                number = value.number();
            }
            default -> throw new InvalidInputException("Unique Identifier kind is invalid");
        }
        CryptoBuilderCalls.<B>builder(this).setUniqueIdentifier(kind, text, number);
        return CryptoBuilderCalls.self(this);
    }
}

interface CryptoParametersInput<B extends CryptoOperationBuilder<B>> {
    default B cryptographicParameters(CryptographicParameters value) {
        CryptoBuilderCalls.<B>builder(this).setCryptographicParameters(value);
        return CryptoBuilderCalls.self(this);
    }

    default B cryptographicParametersStructure(TtlvStructure value) {
        CryptoBuilderCalls.<B>builder(this).setCryptographicParametersStructure(value);
        return CryptoBuilderCalls.self(this);
    }
}

interface CryptoDataInput<B extends CryptoOperationBuilder<B>> {
    default B data(byte[] value) {
        CryptoBuilderCalls.<B>builder(this).setBytes(CryptoOperationBuilder.DATA, value);
        return CryptoBuilderCalls.self(this);
    }

    default B dataValue(TtlvValue value) {
        CryptoBuilderCalls.<B>builder(this).setDataValue(value);
        return CryptoBuilderCalls.self(this);
    }
}

interface CryptoMultipartInput<B extends CryptoOperationBuilder<B>> {
    default B correlationValue(byte[] value) {
        CryptoBuilderCalls.<B>builder(this).setBytes(CryptoOperationBuilder.CORRELATION_VALUE, value);
        return CryptoBuilderCalls.self(this);
    }

    default B initIndicator(boolean value) {
        CryptoBuilderCalls.<B>builder(this).setIndicator(CryptoOperationBuilder.INITIAL_INDICATOR, value);
        return CryptoBuilderCalls.self(this);
    }

    default B finalIndicator(boolean value) {
        CryptoBuilderCalls.<B>builder(this).setIndicator(CryptoOperationBuilder.FINAL_INDICATOR, value);
        return CryptoBuilderCalls.self(this);
    }
}

interface CipherInput<B extends CryptoOperationBuilder<B>> extends CryptoKeyIdentifier<B>,
        CryptoParametersInput<B>, CryptoDataInput<B>, CryptoMultipartInput<B> {
    default B ivCounterNonce(byte[] value) {
        CryptoBuilderCalls.<B>builder(this).setBytes(CryptoOperationBuilder.IV_COUNTER_NONCE, value);
        return CryptoBuilderCalls.self(this);
    }

    default B authenticatedEncryptionAdditionalData(byte[] value) {
        CryptoBuilderCalls.<B>builder(this).setBytes(CryptoOperationBuilder.AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA, value);
        return CryptoBuilderCalls.self(this);
    }
}

interface DecryptInput<B extends CryptoOperationBuilder<B>> extends CipherInput<B> {
    default B authenticatedEncryptionTag(byte[] value) {
        CryptoBuilderCalls.<B>builder(this).setBytes(CryptoOperationBuilder.AUTHENTICATED_ENCRYPTION_TAG, value);
        return CryptoBuilderCalls.self(this);
    }
}

interface SignInput<B extends CryptoOperationBuilder<B>> extends CryptoKeyIdentifier<B>,
        CryptoParametersInput<B>, CryptoDataInput<B>, CryptoMultipartInput<B> {
    default B digestedData(byte[] value) {
        CryptoBuilderCalls.<B>builder(this).setBytes(CryptoOperationBuilder.DIGESTED_DATA, value);
        return CryptoBuilderCalls.self(this);
    }
}

interface SignatureVerifyInput<B extends CryptoOperationBuilder<B>> extends SignInput<B> {
    default B signatureData(byte[] value) {
        CryptoBuilderCalls.<B>builder(this).setBytes(CryptoOperationBuilder.SIGNATURE_DATA, value);
        return CryptoBuilderCalls.self(this);
    }
}

interface MacInput<B extends CryptoOperationBuilder<B>> extends CryptoKeyIdentifier<B>,
        CryptoParametersInput<B>, CryptoDataInput<B>, CryptoMultipartInput<B> {}

interface MacVerifyInput<B extends CryptoOperationBuilder<B>> extends MacInput<B> {
    default B macData(byte[] value) {
        CryptoBuilderCalls.<B>builder(this).setBytes(CryptoOperationBuilder.MAC_DATA, value);
        return CryptoBuilderCalls.self(this);
    }
}

interface HashInput<B extends CryptoOperationBuilder<B>> extends CryptoParametersInput<B>,
        CryptoDataInput<B>, CryptoMultipartInput<B> {}
