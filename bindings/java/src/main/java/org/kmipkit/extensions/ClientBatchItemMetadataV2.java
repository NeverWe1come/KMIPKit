package org.kmipkit.extensions;

import java.nio.ByteBuffer;
import java.nio.charset.CharacterCodingException;
import java.nio.charset.CodingErrorAction;
import java.nio.charset.StandardCharsets;
import java.util.Optional;
import java.util.OptionalLong;

import org.kmipkit.InvalidInputException;

/**
 * Version 2 common metadata including the source operation and exact Result Message.
 *
 * <p>The native message is bounded by the response codec limit (16 MiB by default) and copied
 * into a Java {@link String}. The Java runtime does not guarantee deterministic zeroization of
 * that external copy when it becomes unreachable. The value is returned only through explicit
 * accessors and is omitted from this class's default formatting.
 */
public final class ClientBatchItemMetadataV2 {
    private final long operation;
    private final BatchOutcomeKind outcomeKind;
    private final long resultStatus;
    private final OptionalLong resultReason;
    private final Optional<String> resultMessage;

    ClientBatchItemMetadataV2(long[] values, byte[] messageBytes) {
        if (values == null || values.length != 6 || values[0] < 0 || values[0] > 0xffff_ffffL
                || values[1] < 0 || values[1] > 0xffff_ffffL
                || values[2] < 0 || values[2] > 0xffff_ffffL
                || (values[3] != 0 && values[3] != 1)
                || values[4] < 0 || values[4] > 0xffff_ffffL
                || (values[5] != 0 && values[5] != 1)) {
            throw new InvalidInputException("native batch result metadata v2 is invalid");
        }
        if ((values[5] == 0) != (messageBytes == null)) {
            throw new InvalidInputException("native Result Message presence is invalid");
        }
        operation = values[0];
        outcomeKind = new BatchOutcomeKind(values[1]);
        resultStatus = values[2];
        resultReason = values[3] == 0 ? OptionalLong.empty() : OptionalLong.of(values[4]);
        resultMessage = values[5] == 0 ? Optional.empty() : Optional.of(decodeUtf8(messageBytes));
    }

    /** Returns the raw unsigned KMIP operation value, preserving future values. */
    public long operation() {
        return operation;
    }

    public BatchOutcomeKind outcomeKind() {
        return outcomeKind;
    }

    public long resultStatus() {
        return resultStatus;
    }

    public OptionalLong resultReason() {
        return resultReason;
    }

    /** Returns the exact server Result Message without logging or redacting its value. */
    public Optional<String> resultMessage() {
        return resultMessage;
    }

    private static String decodeUtf8(byte[] bytes) {
        try {
            return StandardCharsets.UTF_8.newDecoder()
                    .onMalformedInput(CodingErrorAction.REPORT)
                    .onUnmappableCharacter(CodingErrorAction.REPORT)
                    .decode(ByteBuffer.wrap(bytes)).toString();
        } catch (CharacterCodingException exception) {
            throw new InvalidInputException("native Result Message is not valid UTF-8");
        }
    }

    @Override
    public String toString() {
        return "ClientBatchItemMetadataV2[operation=" + operation + ", outcomeKind=" + outcomeKind
                + ", resultStatus=" + resultStatus + ", resultReason=" + resultReason
                + ", resultMessage=redacted]";
    }
}
