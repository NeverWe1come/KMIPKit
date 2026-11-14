package org.kmipkit.fluent;

import java.nio.charset.StandardCharsets;
import java.util.Arrays;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.credentials.SecretBytes;
import org.kmipkit.credentials.SecretText;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.FluentBatchItemAdapter;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.protocol.CryptographicParameters;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;

/** Internal ownership bridge for the typed KMIP 3.0 operation builder ABI. */
final class Kmip30OperationHandle implements AutoCloseable {
    private static final int MAX_IDENTIFIER_BYTES = 16 * 1024 * 1024;
    private final NativeHandle handle;

    private Kmip30OperationHandle(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.KMIP30_OPERATION_BUILDER);
    }

    static Kmip30OperationHandle createGroup(AttributeSet attributes) {
        return createWithAttributes(0, attributes);
    }

    static Kmip30OperationHandle createUser(AttributeSet attributes) {
        return createWithAttributes(1, attributes);
    }

    private static Kmip30OperationHandle createWithAttributes(int operation, AttributeSet attributes) {
        if (attributes == null) throw new InvalidInputException("KMIP 3.0 Attributes are required");
        NativeExtensionRegistry.ensureLoaded();
        long nativeHandle = attributes.nativeHandleOwner().withValue(value ->
                NativeExtensionRegistry.kmip30OperationBuilderCreate(
                        operation, value, 0, null, 0, null));
        return new Kmip30OperationHandle(nativeHandle);
    }

    static Kmip30OperationHandle createCredential(
            AttributeSet attributes, CreateCredentialObject object) {
        if (attributes == null || object == null) {
            throw new InvalidInputException("Create Credential inputs are required");
        }
        NativeExtensionRegistry.ensureLoaded();
        long[] values = NativeHandle.withValues(
                new NativeHandle[] {attributes.nativeHandleOwner(), object.nativeHandleOwner()},
                handles -> new long[] {handles[0], handles[1]});
        return new Kmip30OperationHandle(
                NativeExtensionRegistry.kmip30CreateCredentialBuilderCreate(values[0], values[1]));
    }

    static Kmip30OperationHandle deactivate(UniqueIdentifier identifier) {
        return createIdentifierOperation(2, identifier, null);
    }

    static Kmip30OperationHandle decapsulate(UniqueIdentifier identifier, byte[] ciphertext) {
        if (ciphertext == null) throw new InvalidInputException("Decapsulate Data is required");
        byte[] staged = ciphertext.clone();
        try {
            return createIdentifierOperation(3, identifier, staged);
        } finally {
            Arrays.fill(staged, (byte) 0);
        }
    }

    static Kmip30OperationHandle encapsulate(UniqueIdentifier identifier) {
        return createIdentifierOperation(4, identifier, null);
    }

    static Kmip30OperationHandle obliterate(UniqueIdentifier identifier) {
        return createIdentifierOperation(5, identifier, null);
    }

    private static Kmip30OperationHandle createIdentifierOperation(
            int operation, UniqueIdentifier identifier, byte[] ciphertext) {
        if (identifier == null) throw new InvalidInputException("Unique Identifier is required");
        NativeExtensionRegistry.ensureLoaded();
        byte[] text = identifier.kind() == UniqueIdentifier.Kind.TEXT_STRING
                ? identifier.text().getBytes(StandardCharsets.UTF_8) : null;
        if (text != null && text.length > MAX_IDENTIFIER_BYTES) {
            Arrays.fill(text, (byte) 0);
            throw new InvalidInputException("Unique Identifier exceeds the configured limit");
        }
        int kind = switch (identifier.kind()) {
            case TEXT_STRING -> 1;
            case ENUMERATION -> 2;
            case INTEGER -> 3;
        };
        try {
            return new Kmip30OperationHandle(NativeExtensionRegistry.kmip30OperationBuilderCreate(
                    operation, 0, kind, text,
                    identifier.kind() == UniqueIdentifier.Kind.TEXT_STRING ? 0 : identifier.number(),
                    ciphertext));
        } finally {
            if (text != null) Arrays.fill(text, (byte) 0);
        }
    }

    void setAttributes(AttributeSet attributes) {
        if (attributes == null) throw new InvalidInputException("KMIP 3.0 Attributes are required");
        handle.withValue(builder -> attributes.nativeHandleOwner().withValue(value -> {
            NativeExtensionRegistry.kmip30OperationBuilderSetAttributes(builder, value);
            return null;
        }));
    }

    void setCryptographicParameters(CryptographicParameters parameters) {
        if (parameters == null) {
            throw new InvalidInputException("Cryptographic Parameters are required");
        }
        NativeHandle.withValues(new NativeHandle[] {handle, parameters.nativeHandleOwner()}, values -> {
            NativeExtensionRegistry.kmip30OperationBuilderSetCryptographicParameters(
                    values[0], values[1]);
            return null;
        });
    }

    void setDeactivationReason(TtlvStructure reason) {
        if (reason == null) throw new InvalidInputException("Deactivation Reason is required");
        NativeHandle.withValues(new NativeHandle[] {handle, reason.nativeHandleOwner()}, values -> {
            NativeExtensionRegistry.kmip30OperationBuilderSetDeactivationReason(
                    values[0], values[1]);
            return null;
        });
    }

    void setDeactivationDate(long unixSeconds) {
        handle.withValue(builder -> {
            NativeExtensionRegistry.kmip30OperationBuilderSetDeactivationDate(builder, unixSeconds);
            return null;
        });
    }

    void setAsynchronousIndicator(long value) {
        handle.withValue(builder -> {
            NativeExtensionRegistry.kmip30OperationBuilderSetAsynchronousIndicator(builder, value);
            return null;
        });
    }

    void validate() {
        handle.withValue(value -> {
            NativeExtensionRegistry.kmip30OperationBuilderValidate(value);
            return null;
        });
    }

    ClientBatchItem build() {
        return handle.withValue(value -> FluentBatchItemAdapter.takeOwnership(
                NativeExtensionRegistry.kmip30OperationBuilderBuildBatchItem(value)));
    }

    @Override
    public void close() {
        handle.close();
    }

    /** One closed alternative for the KMIP 3.0 Create Credential request. */
    static final class CreateCredentialObject implements AutoCloseable {
        private final NativeHandle object;

        private CreateCredentialObject(long nativeHandle) {
            object = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.CREATE_CREDENTIAL_OBJECT);
        }

        static CreateCredentialObject certificate(TtlvItem certificate) {
            if (certificate == null) throw new InvalidInputException("Certificate is required");
            NativeExtensionRegistry.ensureLoaded();
            return new CreateCredentialObject(certificate.nativeHandleOwner().withValue(
                    NativeExtensionRegistry::createCredentialObjectCertificate));
        }

        static CreateCredentialObject password(
                SecretText password, SecretBytes passwordSalt, Long saltAlgorithm,
                SecretBytes saltedPassword, Integer iterationCount) {
            return withSecretText(password, text -> withSecretBytes(passwordSalt, salt ->
                    withSecretBytes(saltedPassword, salted -> {
                        NativeExtensionRegistry.ensureLoaded();
                        return new CreateCredentialObject(
                                NativeExtensionRegistry.createCredentialObjectPassword(
                                        text, salt, saltAlgorithm != null,
                                        saltAlgorithm == null ? 0 : saltAlgorithm,
                                        salted, iterationCount != null,
                                        iterationCount == null ? 0 : iterationCount));
                    })));
        }

        static CreateCredentialObject device(
                String serial, String identifier, String network, String machine, String media) {
            byte[][] fields = {utf8(serial), utf8(identifier), utf8(network),
                    utf8(machine), utf8(media)};
            try {
                NativeExtensionRegistry.ensureLoaded();
                return new CreateCredentialObject(NativeExtensionRegistry.createCredentialObjectDevice(
                        fields[0], fields[1], fields[2], fields[3], fields[4]));
            } finally {
                wipe(fields);
            }
        }

        static CreateCredentialObject hashedPassword(
                SecretBytes hashedUsernamePassword, SecretBytes hashedPasswordUsername,
                Long hashingAlgorithm) {
            return withSecretBytes(hashedUsernamePassword, first ->
                    withSecretBytes(hashedPasswordUsername, second -> {
                        NativeExtensionRegistry.ensureLoaded();
                        return new CreateCredentialObject(
                                NativeExtensionRegistry.createCredentialObjectHashedPassword(
                                        hashingAlgorithm != null,
                                        hashingAlgorithm == null ? 0 : hashingAlgorithm,
                                        first, second));
                    }));
        }

        static CreateCredentialObject oneTimePassword(
                long algorithm, Long digest, String serial, SecretBytes seed,
                Long interval, Integer digits) {
            byte[] serialBytes = utf8(serial);
            try {
                return withSecretBytes(seed, seedBytes -> {
                    NativeExtensionRegistry.ensureLoaded();
                    return new CreateCredentialObject(
                            NativeExtensionRegistry.createCredentialObjectOneTimePassword(
                                    algorithm, digest != null, digest == null ? 0 : digest,
                                    serialBytes, seedBytes, interval != null,
                                    interval == null ? 0 : interval, digits != null,
                                    digits == null ? 0 : digits));
                });
            } finally {
                if (serialBytes != null) Arrays.fill(serialBytes, (byte) 0);
            }
        }

        public NativeHandle nativeHandleOwner() {
            return object;
        }

        @Override public void close() { object.close(); }
        @Override public String toString() { return "CreateCredentialObject[redacted]"; }

        private static byte[] utf8(String value) {
            if (value == null) return null;
            return value.getBytes(java.nio.charset.StandardCharsets.UTF_8);
        }

        private static void wipe(byte[][] arrays) {
            for (byte[] array : arrays) if (array != null) Arrays.fill(array, (byte) 0);
        }

        private static <T> T withSecretText(SecretText secret,
                java.util.function.Function<byte[], T> action) {
            if (secret == null) return action.apply(null);
            return secret.withStr(characters -> {
                java.nio.ByteBuffer encoded = java.nio.charset.StandardCharsets.UTF_8.encode(
                        java.nio.CharBuffer.wrap(characters));
                byte[] bytes = new byte[encoded.remaining()];
                encoded.get(bytes);
                try { return action.apply(bytes); }
                finally {
                    Arrays.fill(bytes, (byte) 0);
                    if (encoded.hasArray()) Arrays.fill(encoded.array(), (byte) 0);
                }
            });
        }

        private static <T> T withSecretBytes(SecretBytes secret,
                java.util.function.Function<byte[], T> action) {
            return secret == null ? action.apply(null) : secret.withBytes(action::apply);
        }
    }
}
