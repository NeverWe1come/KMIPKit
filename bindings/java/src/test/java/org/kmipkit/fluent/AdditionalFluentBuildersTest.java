package org.kmipkit.fluent;

import static org.junit.jupiter.api.Assertions.*;
import java.util.List;
import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.extensions.*;
import org.kmipkit.protocol.KmipValues;
import org.kmipkit.ttlv.*;

class AdditionalFluentBuildersTest {
    private static final UniqueIdentifier ID = UniqueIdentifier.textString("existing-key");

    @Test
    void lifecycleAndReplacementBuildersBuildValidatedRoutes() {
        try (ActivateBuilder activate = new ActivateBuilder(null).uniqueIdentifier(ID);
             ArchiveBuilder archive = new ArchiveBuilder(null).uniqueIdentifier(ID);
             RecoverBuilder recover = new RecoverBuilder(null).uniqueIdentifier(ID);
             RekeyBuilder rekey = new RekeyBuilder(null).uniqueIdentifier(ID).offset(7);
             RekeyKeyPairBuilder pair = new RekeyKeyPairBuilder(null).privateKeyUniqueIdentifier(ID).offset(9);
             RecertifyBuilder recertify = new RecertifyBuilder(null).uniqueIdentifier(ID).offset(11)) {
            List<TypedOperationBuilder<?>> builders = List.of(activate, archive, recover, rekey, pair, recertify);
            long[] operations = {0x12, 0x15, 0x16, 4, 0x1d, 7};
            for (int index = 0; index < builders.size(); index++) {
                TypedOperationBuilder<?> builder = builders.get(index);
                builder.withOptions(RequestOptions.defaults()).asynchronousIndicator(1);
                builder.validate();
                try (ClientBatchItem item = builder.toBatchItem()) { assertEquals(operations[index], item.operation()); }
            }
            assertThrows(InvalidInputException.class, () -> rekey.offset(-1));
            assertThrows(InvalidInputException.class, () -> rekey.asynchronousIndicator(0x1_0000_0000L));
        }
    }

    @Test
    void splitKeyBuildsRepeatedlyAfterTheAttributeSourceCloses() {
        try (CreateSplitKeyBuilder split = new CreateSplitKeyBuilder(null)
                    .objectType(2).method(1).parts(3).threshold(2).uniqueIdentifier(ID)) {
            try (TtlvStructure empty = TtlvStructure.create(); AttributeSet attributes = AttributeSet.tryNew(empty)) {
                split.attributes(attributes);
            }
            split.validate();
            try (ClientBatchItem item = split.build()) { assertEquals(0x28, item.operation()); }
            split.method(KmipValues.SplitKeyMethod.POLYNOMIAL_SHARING_PRIME_FIELD);
            assertThrows(InvalidInputException.class, split::validate);
        }
    }

    @Test
    void registerCopiesTheObjectAndPreservesVendorAttributes() {
        try (RegisterBuilder builder = new RegisterBuilder(null)) {
            try (Tag typeTag = RawTag.fromRaw(0x420059).checked();
                 Tag valueTag = RawTag.fromRaw(0x42005a).checked();
                 TtlvValue type = TtlvValue.Enumeration(0x80000001L);
                 TtlvValue bytes = TtlvValue.ByteString(new byte[]{1, 2, 3}, CodecLimits.defaults());
                 TtlvItem typeItem = TtlvItem.create(typeTag, type, CodecLimits.defaults());
                 TtlvItem dataItem = TtlvItem.create(valueTag, bytes, CodecLimits.defaults());
                 ManagedObject object = ManagedObject.opaqueObject(List.of(typeItem, dataItem))) {
                builder.object(object).vendorAttribute("vendor", "opaque-metadata", dataItem);
            }
            builder.opaqueObjectContext(OpaqueObjectContext.GENERIC);
            builder.validate();
            try (ClientBatchItem item = builder.build()) { assertEquals(3, item.operation()); }
        }
    }

    @Test
    void closedBuildersRejectFurtherCalls() {
        ActivateBuilder builder = new ActivateBuilder(null);
        builder.close();
        assertThrows(InvalidInputException.class, builder::build);
        assertThrows(InvalidInputException.class, () -> builder.uniqueIdentifier(ID));
    }
}
