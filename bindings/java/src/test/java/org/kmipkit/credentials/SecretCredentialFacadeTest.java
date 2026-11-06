package org.kmipkit.credentials;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.util.concurrent.atomic.AtomicReference;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TtlvValueView;
import org.junit.jupiter.api.Test;

final class SecretCredentialFacadeTest {
    @Test
    void secretBytesCallbackIsScopedAndTtlvConversionPreservesBinaryBytes() {
        byte[] expected = {0x00, 0x7f, (byte) 0xff, 0x00};
        byte[] callerInput = expected.clone();
        AtomicReference<byte[]> callbackArray = new AtomicReference<>();
        SecretBytes secret = SecretBytes.fromBytes(callerInput);
        assertEquals("SecretBytes[redacted]", secret.toString());

        byte[] copied = secret.withBytes(value -> {
            callbackArray.set(value);
            return value.clone();
        });
        assertArrayEquals(expected, copied);
        assertArrayEquals(expected, callerInput);
        assertArrayEquals(new byte[expected.length], callbackArray.get());

        byte[] exceptionalArray = new byte[expected.length];
        assertThrows(IllegalStateException.class, () -> secret.withBytes(value -> {
            System.arraycopy(value, 0, exceptionalArray, 0, value.length);
            callbackArray.set(value);
            throw new IllegalStateException("callback failed");
        }));
        assertArrayEquals(new byte[expected.length], callbackArray.get());
        assertArrayEquals(expected, exceptionalArray);

        try (TtlvValue value = secret.intoTtlvValue(); TtlvValueView view = value.view()) {
            assertEquals(TtlvItemType.ByteString, view.itemType());
            assertEquals(expected.length, view.byteLength());
            for (int index = 0; index < expected.length; index++) {
                assertEquals(expected[index] & 0xff, view.byteAt(index));
            }
        }
        assertThrows(InvalidInputException.class, () -> secret.withBytes(ignored -> null));
        secret.close();
    }

    @Test
    void secretTextRejectsUnpairedSurrogatesAndClearsScopedCharacters() {
        String expected = "snowman \u2603\u0000 caf\u00e9";
        AtomicReference<char[]> callbackArray = new AtomicReference<>();
        SecretText secret = SecretText.fromString(expected);
        assertEquals("SecretText[redacted]", secret.toString());

        String copied = secret.withStr(value -> {
            callbackArray.set(value);
            return new String(value);
        });
        assertEquals(expected, copied);
        assertArrayEquals(new char[callbackArray.get().length], callbackArray.get());

        char[] exceptionalArray = new char[expected.length()];
        assertThrows(IllegalStateException.class, () -> secret.withStr(value -> {
            System.arraycopy(value, 0, exceptionalArray, 0, value.length);
            callbackArray.set(value);
            throw new IllegalStateException("callback failed");
        }));
        assertArrayEquals(new char[callbackArray.get().length], callbackArray.get());
        assertArrayEquals(expected.toCharArray(), exceptionalArray);

        assertThrows(InvalidInputException.class, () -> SecretText.fromString("\ud800"));
        secret.close();
    }

    @Test
    void secretTextMovesIntoATextStringTtlvValue() {
        SecretText secret = SecretText.fromString("native-text-sentinel");

        try (TtlvValue value = secret.intoTtlvValue(); TtlvValueView view = value.view()) {
            assertEquals(TtlvItemType.TextString, view.itemType());
            assertEquals("SecretText[redacted]", secret.toString());
        }
        secret.close();
    }
}
