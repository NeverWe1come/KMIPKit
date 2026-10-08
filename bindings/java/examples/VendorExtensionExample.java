import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.Optional;

import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientConfiguration;
import org.kmipkit.extensions.ClientExtensionRegistry;
import org.kmipkit.extensions.ClientRequestMessageExtension;
import org.kmipkit.extensions.Compatibility;
import org.kmipkit.extensions.Discriminator;
import org.kmipkit.extensions.ExtensionChildRule;
import org.kmipkit.extensions.ExtensionDefinition;
import org.kmipkit.extensions.ExtensionIdentity;
import org.kmipkit.extensions.ExtensionRecognition;
import org.kmipkit.extensions.ExtensionRegistryLimits;
import org.kmipkit.extensions.ExtensionSchema;
import org.kmipkit.extensions.InvalidExtensionSchemaException;
import org.kmipkit.extensions.RegisteredExtensionValue;
import org.kmipkit.extensions.TtlvPath;
import org.kmipkit.extensions.ValidatedExtensionValue;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;

/** Local-only example of registering, inspecting, validating, and attaching an extension.
 *
 * <p>Per KMIPKIT-0007, unknown critical response extensions are rejected, while unknown
 * non-critical response extensions remain available as generic TTLV. Java-managed
 * {@code String} and {@code byte[]} copies cannot be reliably zeroized by KMIPKit. The
 * example values are non-secret placeholders.
 */
public final class VendorExtensionExample {
    private static final String VENDOR_IDENTIFIER = "com.example";
    private static final String EXTENSION_NAME = "key-policy";
    private static final String EXTENSION_VERSION = "1";

    private static final int NESTED_TAG = 0x0054_0010;
    private static final int DISCRIMINATOR_TAG = 0x0054_0011;
    private static final int POLICY_TAG = 0x0054_0012;

    private VendorExtensionExample() {}

    public static void main(String[] args) {
        CodecLimits codecLimits = CodecLimits.defaults();
        ExtensionIdentity identity = ExtensionIdentity.create(
                VENDOR_IDENTIFIER, EXTENSION_NAME, EXTENSION_VERSION);
        ExtensionDefinition definition = createDefinition(identity);
        ClientExtensionRegistry declaredRegistry = ClientExtensionRegistry.create(
                List.of(definition), ExtensionRegistryLimits.defaults());

        try (ClientConfiguration configuration = ClientConfiguration.create(declaredRegistry);
                ClientExtensionRegistry registry = configuration.extensionRegistry()) {
            TtlvStructure validPayload = payload("key-policy-v1", 7, codecLimits);
            ExtensionRecognition recognized = registry.inspect(
                    VENDOR_IDENTIFIER, validPayload, codecLimits);
            require(ExtensionRecognition.isRecognized(recognized),
                    "the registered vendor and discriminator should be recognized");

            Optional<ValidatedExtensionValue> recognizedValue =
                    ExtensionRecognition.validatedValue(recognized);
            require(recognizedValue.isPresent(), "recognized content should expose a validated view");
            require(genericPayloadIsPreserved(ExtensionRecognition.genericValue(recognized)),
                    "recognition should retain the original generic TTLV subtree");
            require(genericPayloadIsPreserved(
                    ValidatedExtensionValue.genericValue(recognizedValue.orElseThrow())),
                    "the typed view should retain access to the original generic TTLV subtree");

            TtlvStructure invalidPayload = payload("key-policy-v1", -1, codecLimits);
            ExtensionRecognition unrecognized = registry.inspect(
                    VENDOR_IDENTIFIER, invalidPayload, codecLimits);
            require(!ExtensionRecognition.isRecognized(unrecognized),
                    "a payload outside the declared schema should remain unrecognized");
            require(genericPayloadIsPreserved(ExtensionRecognition.genericValue(unrecognized)),
                    "unrecognized content should remain available as generic TTLV");
            expectSchemaRejection(registry, identity, invalidPayload, codecLimits);

            RegisteredExtensionValue firstValue = registry.validateExtensionValue(
                    identity, validPayload, codecLimits);
            RegisteredExtensionValue secondValue = registry.validateExtensionValue(
                    identity, payload("key-policy-v1", 9, codecLimits), codecLimits);
            ClientRequestMessageExtension requiredUse = ClientRequestMessageExtension.create(firstValue, true);
            ClientRequestMessageExtension advisoryUse = ClientRequestMessageExtension.create(secondValue, false);

            ClientBatchItem discoverVersions = ClientBatchItem.discoverVersions()
                    .withExtension(requiredUse)
                    .withExtension(advisoryUse);
            require(discoverVersions.extensionCount() == 2,
                    "both validated extensions should be attached");
            require(discoverVersions.extensionIdentityAt(0).equals(identity)
                            && discoverVersions.extensionIdentityAt(1).equals(identity),
                    "extensions should remain in caller attachment order");
            require(discoverVersions.extensionCriticalityIndicatorAt(0),
                    "the first caller-selected Criticality Indicator should be true");
            require(!discoverVersions.extensionCriticalityIndicatorAt(1),
                    "the second caller-selected Criticality Indicator should be false");

            System.out.println("Registered, recognized, and validated "
                    + VENDOR_IDENTIFIER + ":" + EXTENSION_NAME + ":" + EXTENSION_VERSION
                    + "; preserved unrecognized TTLV; attached two ordered extensions to Discover Versions.");
            System.out.println("This example makes no network calls.");
        }
    }

    private static ExtensionDefinition createDefinition(ExtensionIdentity identity) {
        Tag nestedTag = tag(NESTED_TAG);
        Tag discriminatorTag = tag(DISCRIMINATOR_TAG);
        Tag policyTag = tag(POLICY_TAG);

        TtlvPath discriminatorPath = TtlvPath.withChildTag(
                TtlvPath.create(nestedTag), discriminatorTag);
        Discriminator discriminator = Discriminator.create(
                discriminatorPath,
                TtlvValue.TextString("key-policy-v1".getBytes(StandardCharsets.UTF_8), CodecLimits.defaults()));

        ExtensionSchema labelSchema = ExtensionSchema.scalar(TtlvItemType.TextString);
        ExtensionSchema policySchema = ExtensionSchema.with_signed_range(
                ExtensionSchema.scalar(TtlvItemType.Integer), 0, Integer.MAX_VALUE);
        ExtensionSchema nestedSchema = ExtensionSchema.structure(
                List.of(
                        ExtensionChildRule.required(discriminatorTag, labelSchema),
                        ExtensionChildRule.optional(policyTag, policySchema)),
                List.of(), false);
        ExtensionSchema rootSchema = ExtensionSchema.structure(
                List.of(ExtensionChildRule.required(nestedTag, nestedSchema)), List.of(), false);

        Compatibility compatibility = Compatibility.create(2, 1, 2, 1, "0.1.0", "0.1.0");
        return ExtensionDefinition.create(identity, compatibility, discriminator, rootSchema);
    }

    private static TtlvStructure payload(String discriminator, int policy, CodecLimits limits) {
        TtlvStructure nested = TtlvStructure.withItem(
                TtlvStructure.create(),
                TtlvItem.create(tag(DISCRIMINATOR_TAG), text(discriminator, limits), limits),
                limits);
        nested = TtlvStructure.withItem(
                nested,
                TtlvItem.create(tag(POLICY_TAG), TtlvValue.Integer(policy), limits),
                limits);
        TtlvValue nestedValue = TtlvValue.structure(nested, limits);
        return TtlvStructure.withItem(
                TtlvStructure.create(),
                TtlvItem.create(tag(NESTED_TAG), nestedValue, limits),
                limits);
    }

    private static TtlvValue text(String value, CodecLimits limits) {
        return TtlvValue.TextString(value.getBytes(StandardCharsets.UTF_8), limits);
    }

    private static Tag tag(int raw) {
        return RawTag.fromRaw(raw).checked();
    }

    private static boolean genericPayloadIsPreserved(TtlvStructureView view) {
        return view.itemCount() == 1
                && view.itemAt(0).value().structure().itemCount() == 2;
    }

    private static void expectSchemaRejection(ClientExtensionRegistry registry,
            ExtensionIdentity identity, TtlvStructure payload, CodecLimits limits) {
        try {
            registry.validateExtensionValue(identity, payload, limits);
        } catch (InvalidExtensionSchemaException expected) {
            return;
        }
        throw new IllegalStateException("schema-invalid payload was accepted for outbound use");
    }

    private static void require(boolean condition, String message) {
        if (!condition) {
            throw new IllegalStateException(message);
        }
    }
}
