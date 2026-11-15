package org.kmipkit.extensions;

import java.nio.charset.StandardCharsets;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ResourceLimitException;

/** Bounded UTF-8 checks shared by extension adapter entry points. */
final class ExtensionText {
    static final int MAX_BYTES = 4_096;

    private ExtensionText() {}

    static void requireWithinLimit(String value, String description) {
        if (value == null) {
            return;
        }
        checkedUtf8Length(value, MAX_BYTES, description);
    }

    static byte[] encodeUtf8(String value, int maximumBytes, String description) {
        checkedUtf8Length(value, maximumBytes, description);
        return value.getBytes(StandardCharsets.UTF_8);
    }

    static int checkedUtf8Length(String value, int maximumBytes, String description) {
        if (value.length() > maximumBytes) {
            throw new ResourceLimitException(description + " exceeds its configured limit");
        }
        long byteCount = 0;
        for (int index = 0; index < value.length(); index++) {
            char character = value.charAt(index);
            if (character <= 0x7F) {
                byteCount++;
            } else if (character <= 0x7FF) {
                byteCount += 2;
            } else if (Character.isHighSurrogate(character)) {
                if (index + 1 >= value.length() || !Character.isLowSurrogate(value.charAt(index + 1))) {
                    throw new InvalidInputException(description + " contains invalid UTF-16");
                }
                byteCount += 4;
                index++;
            } else if (Character.isLowSurrogate(character)) {
                throw new InvalidInputException(description + " contains invalid UTF-16");
            } else {
                byteCount += 3;
            }
            if (byteCount > maximumBytes) {
                throw new ResourceLimitException(description + " exceeds its configured limit");
            }
        }
        return (int) byteCount;
    }
}
