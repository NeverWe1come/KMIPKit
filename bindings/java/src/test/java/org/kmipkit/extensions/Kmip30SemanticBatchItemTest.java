package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;

import org.junit.jupiter.api.Test;
import org.kmipkit.credentials.CredentialType;
import org.kmipkit.protocol.KmipValues;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;

final class Kmip30SemanticBatchItemTest {
    @Test
    void buildsSemanticSystemObjectOperationsFromTypedInputs() {
        try (TtlvStructure empty = TtlvStructure.create();
             AttributeSet attributes = AttributeSet.tryNew(empty);
             ClientBatchItem createGroup = ClientBatchItem.createGroup(attributes);
             ClientBatchItem createUser = ClientBatchItem.createUser(attributes)) {
            attributes.close();
            assertEquals(KmipValues.V3_0_OPERATION_CREATE_GROUP, createGroup.operation());
            assertEquals(KmipValues.V3_0_OPERATION_CREATE_USER, createUser.operation());
        }

        UniqueIdentifier identifier = UniqueIdentifier.textString("system-object");
        try (ClientBatchItem obliterate = ClientBatchItem.obliterate(identifier);
             ClientBatchItem deactivate = ClientBatchItem.deactivate(identifier, null, null)) {
            assertEquals(KmipValues.V3_0_OPERATION_OBLITERATE, obliterate.operation());
            assertEquals(KmipValues.V3_0_OPERATION_DEACTIVATE, deactivate.operation());
        }
    }

    @Test
    void buildsCredentialAndKemOperationsFromTypedInputs() {
        CodecLimits limits = CodecLimits.defaults();
        try (RawTag rawTag = RawTag.fromRaw(0x004201A6);
             Tag tag = rawTag.checked();
             TtlvStructure fields = TtlvStructure.create();
             TtlvValue value = TtlvValue.structure(fields, limits);
             TtlvItem object = TtlvItem.create(tag, value, limits);
             TtlvStructure empty = TtlvStructure.create();
             AttributeSet attributes = AttributeSet.tryNew(empty);
             ClientBatchItem createCredential = ClientBatchItem.createCredential(
                     CredentialType.fromRaw(CredentialType.DEVICE),
                     KmipValues.V3_0_OBJECT_TYPE_DEVICE_CREDENTIAL, attributes, object);
             ClientBatchItem encapsulate = ClientBatchItem.encapsulate(
                     UniqueIdentifier.textString("kem-key"), null, null);
             ClientBatchItem decapsulate = ClientBatchItem.decapsulate(
                     UniqueIdentifier.textString("kem-key"), new byte[] {0x31, 0x32}, null, null)) {
            assertEquals(KmipValues.V3_0_OPERATION_CREATE_CREDENTIAL, createCredential.operation());
            assertEquals(KmipValues.V3_0_OPERATION_ENCAPSULATE, encapsulate.operation());
            assertEquals(KmipValues.V3_0_OPERATION_DECAPSULATE, decapsulate.operation());
        }
    }
}
