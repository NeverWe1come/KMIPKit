package org.kmipkit.extensions;

import java.nio.charset.StandardCharsets;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.TtlvValue;

final class WrapperTransferValidationTest {
    private static final int PRIVATE_TAG = 0x420001;

    @Test
    void nullFirstTagIsRejectedBeforeNativePathCreation() {
        assertThrows(InvalidInputException.class, () -> TtlvPath.create(null));
    }

    @Test
    void nullTagDoesNotConsumeTtlvPath() {
        try (RawTag rawTag = RawTag.fromRaw(PRIVATE_TAG);
                Tag tag = rawTag.checked();
                TtlvPath path = TtlvPath.create(tag)) {
            assertThrows(InvalidInputException.class, () -> TtlvPath.withChildTag(path, null));

            try (TtlvPath child = TtlvPath.withChildTag(path, tag)) {
                assertNotEquals(0, child.handle());
            }
        }
    }

    @Test
    void closedTagDoesNotConsumeTtlvPath() {
        try (RawTag rawTag = RawTag.fromRaw(PRIVATE_TAG);
                Tag tag = rawTag.checked();
                RawTag rawClosedTag = RawTag.fromRaw(PRIVATE_TAG + 1);
                Tag closedTag = rawClosedTag.checked();
                TtlvPath path = TtlvPath.create(tag)) {
            closedTag.close();

            assertThrows(InvalidInputException.class, () -> TtlvPath.withChildTag(path, closedTag));

            try (TtlvPath child = TtlvPath.withChildTag(path, tag)) {
                assertNotEquals(0, child.handle());
            }
        }
    }

    @Test
    void nullInformationDoesNotConsumeExtensionDefinition() {
        try (ExtensionDefinition definition = createDefinition()) {
            assertThrows(InvalidInputException.class,
                    () -> ExtensionDefinition.withInformation(definition, null));

            assertNotEquals(0, definition.handle());
        }
    }

    @Test
    void closedInformationDoesNotConsumeExtensionDefinition() {
        try (ExtensionDefinition definition = createDefinition();
                ExtensionInformation information = ExtensionInformation.create("policy metadata")) {
            information.close();

            assertThrows(InvalidInputException.class,
                    () -> ExtensionDefinition.withInformation(definition, information));

            assertNotEquals(0, definition.handle());
        }
    }

    @Test
    void nullItemTypeDoesNotConsumeExtensionInformation() {
        try (ExtensionInformation information = ExtensionInformation.create("policy metadata")) {
            assertThrows(InvalidInputException.class,
                    () -> ExtensionInformation.with_type(information, null));

            try (var encoded = ExtensionInformation.toTtlv(information)) {
                assertNotEquals(0, encoded.handle());
            }
        }
    }

    private static ExtensionDefinition createDefinition() {
        try (RawTag rawTag = RawTag.fromRaw(PRIVATE_TAG);
                Tag tag = rawTag.checked();
                ExtensionIdentity identity = ExtensionIdentity.create("example.vendor", "policy", "1.0");
                Compatibility compatibility = Compatibility.create(2, 1, 2, 1, "1.0.0", "1.0.0");
                TtlvPath path = TtlvPath.create(tag);
                TtlvValue scalar = TtlvValue.TextString(
                        "marker".getBytes(StandardCharsets.UTF_8), CodecLimits.defaults());
                ExtensionSchema scalarSchema = ExtensionSchema.scalar(TtlvItemType.TextString);
                ExtensionChildRule childRule = ExtensionChildRule.required(tag, scalarSchema);
                ExtensionSchema schema = ExtensionSchema.structure(List.of(childRule), List.of(), true)) {
            try (Discriminator discriminator = Discriminator.create(path, scalar)) {
                return ExtensionDefinition.create(identity, compatibility, discriminator, schema);
            }
        }
    }
}
