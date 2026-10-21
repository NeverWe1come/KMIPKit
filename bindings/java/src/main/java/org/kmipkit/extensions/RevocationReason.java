package org.kmipkit.extensions;

import java.nio.ByteBuffer;
import java.nio.CharBuffer;
import java.nio.charset.CharacterCodingException;
import java.nio.charset.CodingErrorAction;
import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.Optional;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.internal.NativeHandle;
import org.jspecify.annotations.Nullable;

/** Raw-preserving KMIP 2.1 Revocation Reason Code and optional audit message. */
public final class RevocationReason implements AutoCloseable {
    private static final int MAX_MESSAGE_BYTES = 16 * 1024 * 1024;

    private final NativeHandle handle;
    private final long rawCode;
    private final Optional<String> message;

    private RevocationReason(long nativeHandle, long rawCode, @Nullable String message) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.REVOCATION_REASON);
        this.rawCode = rawCode;
        this.message = Optional.ofNullable(message);
    }

    /** Creates a reason while preserving its unsigned Enumeration value exactly. */
    public static RevocationReason create(long rawCode, @Nullable String message) {
        if (rawCode < 0 || rawCode > 0xffff_ffffL) {
            throw new InvalidInputException("Revocation Reason Code is outside uint32");
        }
        byte[] encodedMessage = message == null ? null : encodeMessage(message);
        try {
            NativeExtensionRegistry.ensureLoaded();
            long nativeHandle = NativeExtensionRegistry.revocationReasonCreate(
                    rawCode, encodedMessage);
            return new RevocationReason(nativeHandle, rawCode, message);
        } finally {
            if (encodedMessage != null) Arrays.fill(encodedMessage, (byte) 0);
        }
    }

    /** Returns the exact unsigned raw Enumeration value. */
    public long rawCode() {
        return rawCode;
    }

    /** Returns the optional audit message. */
    public Optional<String> message() {
        return message;
    }

    NativeHandle nativeHandleOwner() {
        return handle;
    }

    private static byte[] encodeMessage(String value) {
        try {
            ByteBuffer encoded = StandardCharsets.UTF_8.newEncoder()
                    .onMalformedInput(CodingErrorAction.REPORT)
                    .onUnmappableCharacter(CodingErrorAction.REPORT)
                    .encode(CharBuffer.wrap(value));
            if (encoded.remaining() > MAX_MESSAGE_BYTES) {
                throw new ResourceLimitException("Revocation Message exceeds the configured limit");
            }
            byte[] bytes = new byte[encoded.remaining()];
            encoded.get(bytes);
            return bytes;
        } catch (CharacterCodingException exception) {
            throw new InvalidInputException("Revocation Message is not valid Unicode");
        }
    }

    @Override
    public void close() {
        handle.close();
    }

    @Override
    public String toString() {
        return "RevocationReason[rawCode=" + rawCode + ", message="
                + (message.isPresent() ? "redacted" : "absent") + "]";
    }
}
