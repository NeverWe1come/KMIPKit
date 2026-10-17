package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.util.List;

import org.junit.jupiter.api.Test;
import org.kmipkit.extensions.Client;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.ClientConfiguration;
import org.kmipkit.extensions.ClientExtensionRegistry;
import org.kmipkit.extensions.ClientRequestMessageExtension;
import org.kmipkit.extensions.Compatibility;
import org.kmipkit.extensions.Discriminator;
import org.kmipkit.extensions.ExtensionChildRule;
import org.kmipkit.extensions.ExtensionDefinition;
import org.kmipkit.extensions.ExtensionIdentity;
import org.kmipkit.extensions.ExtensionInformation;
import org.kmipkit.extensions.ExtensionOrderConstraint;
import org.kmipkit.extensions.ExtensionRecognition;
import org.kmipkit.extensions.ExtensionSchema;
import org.kmipkit.extensions.RegisteredExtensionValue;
import org.kmipkit.extensions.TtlvPath;
import org.kmipkit.extensions.TransportConfiguration;
import org.kmipkit.extensions.ValidatedExtensionValue;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TtlvValueView;

final class JavaFacadeContractTest {
    private static final List<Class<?>> T021_NATIVE_RESOURCE_TYPES = List.of(
            Client.class,
            ClientBatch.class,
            ClientBatchItem.class,
            ClientBatchItemResult.class,
            ClientBatchResponse.class,
            ClientConfiguration.class,
            ClientExtensionRegistry.class,
            ClientRequestMessageExtension.class,
            Compatibility.class,
            Discriminator.class,
            ExtensionChildRule.class,
            ExtensionDefinition.class,
            ExtensionIdentity.class,
            ExtensionInformation.class,
            ExtensionOrderConstraint.class,
            ExtensionRecognition.class,
            ExtensionSchema.class,
            RegisteredExtensionValue.class,
            TransportConfiguration.class,
            TransportConfiguration.Builder.class,
            TransportConfiguration.TransportInput.class,
            TtlvPath.class,
            ValidatedExtensionValue.class,
            RawTag.class,
            Tag.class,
            TtlvItem.class,
            TtlvItemView.class,
            TtlvStructure.class,
            TtlvStructureView.class,
            TtlvValue.class,
            TtlvValueView.class);

    @Test
    void everyT021NativeResourceTypeIsAutoCloseable() {
        for (Class<?> type : T021_NATIVE_RESOURCE_TYPES) {
            assertTrue(AutoCloseable.class.isAssignableFrom(type),
                    () -> type.getName() + " must expose T021 AutoCloseable ownership");
        }
    }

    @Test
    void everyJavaPackageIsNullMarked() throws IOException {
        for (String packageInfo : List.of(
                "/org/kmipkit/package-info.class",
                "/org/kmipkit/extensions/package-info.class",
                "/org/kmipkit/ttlv/package-info.class",
                "/org/kmipkit/generated/package-info.class",
                "/org/kmipkit/internal/package-info.class")) {
            try (var resource = JavaFacadeContractTest.class.getResourceAsStream(packageInfo)) {
                assertTrue(resource != null, () -> packageInfo + " must declare JSpecify nullness");
                String classFile = new String(resource.readAllBytes(), StandardCharsets.ISO_8859_1);
                assertTrue(classFile.contains("org/jspecify/annotations/NullMarked"),
                        () -> packageInfo + " must be annotated with @NullMarked");
            }
        }
    }

    @Test
    void ttlvOwnedValueCloseReleasesItsHandleAndIsIdempotent() {
        TtlvValue value = TtlvValue.Integer(7);
        assertTrue(AutoCloseable.class.isAssignableFrom(TtlvValue.class),
                "owned TTLV values must expose deterministic close");
        AutoCloseable resource = (AutoCloseable) (Object) value;

        assertDoesNotThrow(resource::close);
        assertDoesNotThrow(resource::close);
        assertThrows(InvalidInputException.class, value::handle);
    }

    @Test
    void invalidByteStringArgumentsUseRedactedStableInputErrors() {
        byte[] secret = "private-payload-marker".getBytes(StandardCharsets.UTF_8);
        InvalidInputException failure = assertThrows(InvalidInputException.class,
                () -> TtlvValue.ByteString(secret, null));

        assertTrue(!failure.getMessage().contains("private-payload-marker"),
                "diagnostics must not include caller payload bytes");
    }
}
