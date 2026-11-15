package org.kmipkit.extensions;

import java.nio.ByteBuffer;
import java.nio.charset.CharacterCodingException;
import java.nio.charset.CodingErrorAction;
import java.nio.charset.StandardCharsets;
import java.util.Optional;
import java.util.OptionalLong;

import org.kmipkit.InvalidInputException;

/** Common, forward-compatible metadata for one asynchronous client operation result. */
public final class AsyncOperationMetadataV1 {
    private final long operation;
    private final BatchOutcomeKind outcomeKind;
    private final long resultStatus;
    private final OptionalLong resultReason;
    private final OptionalLong cancellationResult;
    private final Optional<String> resultMessage;

    AsyncOperationMetadataV1(long[] values, byte[] messageBytes) {
        if (values == null || values.length != 8) {
            throw new InvalidInputException("native asynchronous result metadata is invalid");
        }
        for (int index : new int[] {0, 1, 2, 4, 6}) {
            if (values[index] < 0 || values[index] > 0xffff_ffffL) {
                throw new InvalidInputException("native asynchronous result metadata is invalid");
            }
        }
        if ((values[3] != 0 && values[3] != 1) || (values[5] != 0 && values[5] != 1)
                || (values[7] != 0 && values[7] != 1)
                || (values[3] == 0 && values[4] != 0)
                || (values[5] == 0 && values[6] != 0)
                || ((values[7] == 0) != (messageBytes == null))) {
            throw new InvalidInputException("native asynchronous result metadata is inconsistent");
        }
        operation = values[0];
        outcomeKind = new BatchOutcomeKind(values[1]);
        resultStatus = values[2];
        resultReason = values[3] == 0 ? OptionalLong.empty() : OptionalLong.of(values[4]);
        cancellationResult = values[5] == 0 ? OptionalLong.empty() : OptionalLong.of(values[6]);
        resultMessage = values[7] == 0 ? Optional.empty() : Optional.of(decodeUtf8(messageBytes));
    }

    /** Returns the raw unsigned KMIP operation value, preserving future values. */
    public long operation() { return operation; }

    /** Returns the forward-compatible completed or Pending outcome discriminator. */
    public BatchOutcomeKind outcomeKind() { return outcomeKind; }

    /** Returns the raw unsigned KMIP Result Status value. */
    public long resultStatus() { return resultStatus; }

    /** Returns the optional raw unsigned KMIP Result Reason value. */
    public OptionalLong resultReason() { return resultReason; }

    /** Returns the optional raw unsigned KMIP Cancellation Result value. */
    public OptionalLong cancellationResult() { return cancellationResult; }

    /** Returns the exact server Result Message; Java runtime copies are outside zeroization control. */
    public Optional<String> resultMessage() { return resultMessage; }

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
        return "AsyncOperationMetadataV1[operation=" + operation + ", outcomeKind=" + outcomeKind
                + ", resultStatus=" + resultStatus + ", resultReason=" + resultReason
                + ", cancellationResult=" + cancellationResult + ", resultMessage=redacted]";
    }
}
