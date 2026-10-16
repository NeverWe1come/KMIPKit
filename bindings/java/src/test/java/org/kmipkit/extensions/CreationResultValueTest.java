package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Optional;
import java.util.OptionalLong;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;

final class CreationResultValueTest {
    @Test
    void failureResultMessageMatchesTheSharedFixture() throws Exception {
        Path fixture = Path.of("..").resolve("..").resolve("tests/fixtures/api/shared-capability-parity.json");
        String json = Files.readString(fixture.normalize());
        assertTrue(json.contains("\"id\": \"creation.result-message-on-failure.v1\""));
        assertTrue(json.contains("\"resultMessage\": \"création refusée ✓\""));
        byte[] message = "création refusée ✓".getBytes(StandardCharsets.UTF_8);
        ClientBatchItemMetadataV2 metadata = new ClientBatchItemMetadataV2(
                new long[] {1, 1, 1, 1, 5, 1}, message);
        assertEquals(1, metadata.operation());
        assertEquals(1, metadata.resultStatus());
        assertEquals(OptionalLong.of(5), metadata.resultReason());
        assertEquals(Optional.of("création refusée ✓"), metadata.resultMessage());
        assertFalse(metadata.toString().contains("création refusée"));
    }

    @Test
    void metadataRetainsUnknownValuesAndDistinguishesEmptyFromAbsentMessage() {
        byte[] message = "metadata marker: création ✓".getBytes(StandardCharsets.UTF_8);
        ClientBatchItemMetadataV2 metadata = new ClientBatchItemMetadataV2(
                new long[] {0xffff_ffffL, 99, 0xffff_fffeL, 1, 0xffff_fffdL, 1}, message);

        assertEquals(0xffff_ffffL, metadata.operation());
        assertEquals(99, metadata.outcomeKind().raw());
        assertEquals(0xffff_fffeL, metadata.resultStatus());
        assertEquals(OptionalLong.of(0xffff_fffdL), metadata.resultReason());
        assertEquals(Optional.of("metadata marker: création ✓"), metadata.resultMessage());
        assertFalse(metadata.toString().contains("metadata marker"));

        ClientBatchItemMetadataV2 empty = new ClientBatchItemMetadataV2(
                new long[] {40, 1, 1, 1, 5, 1}, new byte[0]);
        ClientBatchItemMetadataV2 absent = new ClientBatchItemMetadataV2(
                new long[] {40, 1, 1, 1, 5, 0}, null);
        assertEquals(Optional.of(""), empty.resultMessage());
        assertEquals(Optional.empty(), absent.resultMessage());
    }

    @Test
    void metadataRejectsMalformedShapePresenceAndUtf8() {
        assertThrows(InvalidInputException.class, () -> new ClientBatchItemMetadataV2(
                new long[] {-1, 1, 0, 0, 0, 0}, null));
        assertThrows(InvalidInputException.class, () -> new ClientBatchItemMetadataV2(
                new long[] {40, 1, 1, 1, 5, 1}, null));
        assertThrows(InvalidInputException.class, () -> new ClientBatchItemMetadataV2(
                new long[] {40, 1, 1, 1, 5, 1}, new byte[] {(byte) 0xff}));
    }

    @Test
    void uniqueIdentifierPreservesEachWireTypeAndRedactsItsText() {
        UniqueIdentifier text = UniqueIdentifier.fromNative(
                1, "public-key-α".getBytes(StandardCharsets.UTF_8), 0, 0);
        UniqueIdentifier enumeration = UniqueIdentifier.fromNative(2, null, 0xffff_ffffL, 0);
        UniqueIdentifier integer = UniqueIdentifier.fromNative(3, null, 0, -7);

        assertEquals(UniqueIdentifier.Kind.TEXT_STRING, text.kind());
        assertEquals("public-key-α", text.text());
        assertFalse(text.toString().contains("public-key"));
        assertEquals(0xffff_ffffL, enumeration.number());
        assertEquals(-7, integer.number());
    }

    @Test
    void uniqueIdentifierFactoriesAndAccessorsRejectInvalidValuesAndKinds() {
        assertThrows(InvalidInputException.class, () -> UniqueIdentifier.textString("\uD800"));
        assertThrows(InvalidInputException.class, () -> UniqueIdentifier.enumeration(-1));
        assertThrows(InvalidInputException.class, () -> UniqueIdentifier.enumeration(0x1_0000_0000L));

        UniqueIdentifier text = UniqueIdentifier.textString("key-id");
        UniqueIdentifier enumeration = UniqueIdentifier.enumeration(7);
        UniqueIdentifier integer = UniqueIdentifier.integer(-7);
        assertThrows(InvalidInputException.class, text::number);
        assertThrows(InvalidInputException.class, enumeration::text);
        assertThrows(InvalidInputException.class, integer::text);
    }

    @Test
    void uniqueIdentifierRejectsMalformedNativeUtf8WithoutExposingBytes() {
        InvalidInputException failure = assertThrows(InvalidInputException.class,
                () -> UniqueIdentifier.fromNative(1, new byte[] {(byte) 0xff}, 0, 0));

        assertEquals("native Unique Identifier is not valid UTF-8 text", failure.getMessage());
        assertThrows(InvalidInputException.class,
                () -> UniqueIdentifier.fromNative(4, null, 0, 0));
    }
}
