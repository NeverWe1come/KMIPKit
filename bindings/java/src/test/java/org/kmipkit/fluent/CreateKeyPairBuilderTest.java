package org.kmipkit.fluent;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.nio.charset.StandardCharsets;

import org.junit.jupiter.api.Test;
import org.kmipkit.ClientOperationException;
import org.kmipkit.InvalidInputException;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.protocol.KmipValues;
import org.kmipkit.protocol.KeyUsage;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.StandardAttribute;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;

final class CreateKeyPairBuilderTest {
    @Test
    void fluentRequestExposesCreateKeyPairBuilder() throws Exception {
        assertEquals(CreateKeyPairBuilder.class,
                FluentRequest.class.getMethod("createKeyPair").getReturnType());
        assertNotNull(CreateKeyPairBuilder.class.getMethod("privateUsageMask", int.class));
        assertNotNull(CreateKeyPairBuilder.class.getMethod("publicUsageMask", int.class));
        assertNotNull(CreateKeyPairBuilder.class.getMethod("privateUsageMask", KeyUsage[].class));
        assertNotNull(CreateKeyPairBuilder.class.getMethod("publicUsageMask", KeyUsage[].class));
    }

    @Test
    void namedKeyUsageFlagsAreMappedToTheirWireBits() {
        assertEquals((int) KmipValues.CryptographicUsageMask.ENCRYPT, KeyUsage.ENCRYPT.bit());
        assertEquals(
                (int) (KmipValues.CryptographicUsageMask.SIGN | KmipValues.CryptographicUsageMask.VERIFY),
                KeyUsage.mask(KeyUsage.SIGN, KeyUsage.VERIFY));
        assertEquals(0, KeyUsage.mask());
        assertThrows(NullPointerException.class, () -> KeyUsage.mask((KeyUsage[]) null));
        try (CreateKeyPairBuilder builder = new CreateKeyPairBuilder(null)) {
            assertSame(builder, builder.privateUsageMask(KeyUsage.SIGN, KeyUsage.UNWRAP_KEY));
            assertSame(builder, builder.publicUsageMask(KeyUsage.VERIFY));
        }
    }

    @Test
    void builderRejectsMissingCryptoChoicesAndBuildsTypedBatchItem() {
        try (CreateKeyPairBuilder builder = new CreateKeyPairBuilder(null)) {
            assertThrows(ClientOperationException.class, builder::build);
            builder.algorithm(KmipValues.CryptographicAlgorithm.RSA)
                    .length(2048)
                    .seed(new byte[] {1, 2, 3})
                    .privateUsageMask((int) KmipValues.CryptographicUsageMask.SIGN)
                    .publicUsageMask((int) KmipValues.CryptographicUsageMask.VERIFY);
            builder.validate();
            try (ClientBatchItem item = builder.build()) {
                assertEquals(KmipValues.Operation.CREATE_KEY_PAIR, item.operation());
            }
        }
    }

    @Test
    void builderExercisesNamesAttributesOptionsAndInputValidation() {
        CodecLimits limits = CodecLimits.defaults();
        try (RawTag expertRaw = RawTag.fromRaw(0x54F001);
                Tag expertTag = expertRaw.checked();
                TtlvValue expertValue = TtlvValue.Integer(17);
                TtlvItem expertItem = TtlvItem.create(expertTag, expertValue, limits);
                RawTag descriptionRaw = RawTag.fromRaw(StandardAttribute.DESCRIPTION.tag());
                Tag descriptionTag = descriptionRaw.checked();
                TtlvValue description = TtlvValue.TextString(
                        "key pair description".getBytes(StandardCharsets.UTF_8), limits);
                TtlvStructure nested = TtlvStructure.create();
                TtlvValue nestedValue = TtlvValue.structure(nested, limits);
                TtlvItem nestedItem = TtlvItem.create(expertTag, nestedValue, limits);
                CreateKeyPairBuilder builder = new CreateKeyPairBuilder(null)) {
            assertSame(builder, builder.privateName("signing key"));
            assertSame(builder, builder.publicName("verification key"));
            assertSame(builder, builder.commonAttributeItem(expertItem));
            assertSame(builder, builder.privateAttributeItem(expertItem));
            assertSame(builder, builder.publicAttributeItem(expertItem));
            assertSame(builder, builder.commonCataloguedAttribute(descriptionTag, description));
            assertSame(builder, builder.privateCataloguedAttribute(descriptionTag, description));
            assertSame(builder, builder.publicCataloguedAttribute(descriptionTag, description));
            assertSame(builder, builder.commonVendorAttribute("Vendor_Name.1", "profile", nestedItem));
            assertSame(builder, builder.privateVendorAttribute("Vendor_Name.1", "policy", expertItem));
            assertSame(builder, builder.publicVendorAttribute("Vendor_Name.1", "region", expertItem));
            assertSame(builder, builder.withOptions(RequestOptions.defaults()));
            assertSame(builder, builder.asynchronousIndicator(0xffff_ffffL));

            builder.algorithm(KmipValues.CryptographicAlgorithm.ECDSA)
                    .length(256)
                    .privateUsageMask((int) KmipValues.CryptographicUsageMask.SIGN)
                    .publicUsageMask((int) KmipValues.CryptographicUsageMask.VERIFY);
            builder.validate();
            try (ClientBatchItem item = builder.toBatchItem()) {
                assertEquals(KmipValues.Operation.CREATE_KEY_PAIR, item.operation());
            }

            assertThrows(InvalidInputException.class, () -> builder.privateName(null));
            assertThrows(InvalidInputException.class, () -> builder.publicName(null));
            assertThrows(InvalidInputException.class, () -> builder.commonAttributeItem(null));
            assertThrows(InvalidInputException.class,
                    () -> builder.privateCataloguedAttribute(null, description));
            assertThrows(InvalidInputException.class,
                    () -> builder.publicCataloguedAttribute(descriptionTag, null));
            assertThrows(InvalidInputException.class,
                    () -> builder.commonVendorAttribute(null, "region", expertItem));
            assertThrows(InvalidInputException.class,
                    () -> builder.privateVendorAttribute("Vendor_Name.1", null, expertItem));
            assertThrows(InvalidInputException.class,
                    () -> builder.publicVendorAttribute("Vendor_Name.1", "region", null));
            assertThrows(InvalidInputException.class, () -> builder.withOptions(null));
            assertThrows(InvalidInputException.class, () -> builder.asynchronousIndicator(-1));
            assertThrows(InvalidInputException.class,
                    () -> builder.asynchronousIndicator(0x1_0000_0000L));
            assertThrows(InvalidInputException.class, () -> builder.execute(null));
            assertThrows(InvalidInputException.class, () -> builder.execute(limits));
        }
    }
}
