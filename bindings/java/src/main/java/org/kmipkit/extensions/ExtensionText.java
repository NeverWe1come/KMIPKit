package org.kmipkit.extensions;

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
        if (value.length() > MAX_BYTES || exceedsUtf8ByteLimit(value, description)) {
            throw new ResourceLimitException(description + " exceeds its configured limit");
        }
    }

    private static boolean exceedsUtf8ByteLimit(String value, String description) {
        int byteCount = 0;
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
            if (byteCount > MAX_BYTES) {
                return true;
            }
        }
        return false;
    }
}
