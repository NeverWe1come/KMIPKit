package org.kmipkit.credentials;

import java.nio.ByteBuffer;
import java.nio.CharBuffer;
import java.nio.charset.CharacterCodingException;
import java.nio.charset.CodingErrorAction;
import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.function.Function;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvValue;

/** Owns caller-provided UTF-8 text in native zeroizing memory. */
public final class SecretText implements AutoCloseable {
    private final NativeHandle handle;

    private SecretText(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.SECRET_TEXT);
    }

    /** Strictly encode text and copy it into a native zeroizing owner. */
    public static SecretText fromString(String value) {
        if (value == null) {
            throw new InvalidInputException("secret text input is invalid");
        }
        ByteBuffer encoded = null;
        byte[] bytes = null;
        try {
            encoded = StandardCharsets.UTF_8.newEncoder()
                    .onMalformedInput(CodingErrorAction.REPORT)
                    .onUnmappableCharacter(CodingErrorAction.REPORT)
                    .encode(CharBuffer.wrap(value));
            bytes = new byte[encoded.remaining()];
            encoded.get(bytes);
            NativeExtensionRegistry.ensureLoaded();
            long[] output = new long[1];
            CredentialInterop.throwIfFailed(
                    NativeExtensionRegistry.secretTextCreate(bytes, output), "secret text");
            return new SecretText(output[0]);
        } catch (CharacterCodingException exception) {
            throw new InvalidInputException("secret text is not valid Unicode");
        } finally {
            if (bytes != null) {
                Arrays.fill(bytes, (byte) 0);
            }
            if (encoded != null && encoded.hasArray()) {
                Arrays.fill(encoded.array(), (byte) 0);
            }
        }
    }

    /** Lend temporary decoded characters to a synchronous callback, then clear them. */
    @SuppressWarnings("unchecked")
    public <T> T withStr(Function<char[], T> callback) {
        if (callback == null) {
            throw new InvalidInputException("secret text callback is invalid");
        }
        return handle.withValue(value -> {
            byte[][] scopedBytes = new byte[1][];
            try {
                return (T) NativeExtensionRegistry.secretTextWithStr(value, borrowed -> {
                    scopedBytes[0] = borrowed;
                    char[] scratch = new char[borrowed.length];
                    char[] scopedCharacters = null;
                    try {
                        CharBuffer output = CharBuffer.wrap(scratch);
                        var decoder = StandardCharsets.UTF_8.newDecoder()
                                .onMalformedInput(CodingErrorAction.REPORT)
                                .onUnmappableCharacter(CodingErrorAction.REPORT);
                        var decoded = decoder.decode(ByteBuffer.wrap(borrowed), output, true);
                        if (decoded.isError()) {
                            decoded.throwException();
                        }
                        var flushed = decoder.flush(output);
                        if (flushed.isError()) {
                            flushed.throwException();
                        }
                        scopedCharacters = Arrays.copyOf(scratch, output.position());
                        return callback.apply(scopedCharacters);
                    } catch (CharacterCodingException exception) {
                        throw new InvalidInputException("native secret text is not valid UTF-8");
                    } finally {
                        if (scopedCharacters != null) {
                            Arrays.fill(scopedCharacters, '\0');
                        }
                        Arrays.fill(scratch, '\0');
                        Arrays.fill(borrowed, (byte) 0);
                        scopedBytes[0] = null;
                    }
                });
            } finally {
                if (scopedBytes[0] != null) {
                    Arrays.fill(scopedBytes[0], (byte) 0);
                }
            }
        });
    }

    /** Move the native secret allocation into a zeroizing TTLV Text String value. */
    public TtlvValue intoTtlvValue() {
        return NativeHandle.withNativeHandles(() -> {
            try {
                TtlvValue value = NativeExtensionRegistry.secretTextIntoTtlvValue(handle.get());
                handle.transfer();
                return value;
            } catch (InvalidInputException exception) {
                throw exception;
            } catch (RuntimeException | Error exception) {
                handle.transfer();
                throw exception;
            }
        });
    }

    @Override
    public void close() {
        handle.close();
    }

    @Override
    public String toString() {
        return "SecretText[redacted]";
    }
}
