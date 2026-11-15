package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.nio.charset.StandardCharsets;
import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
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
             ClientBatchItem createGroup = ClientBatchItem.createGroupFromAttributes(attributes);
             ClientBatchItem createUser = ClientBatchItem.createUserFromAttributes(attributes)) {
            attributes.close();
            assertEquals(KmipValues.V3_0_OPERATION_CREATE_GROUP, createGroup.operation());
            assertEquals(KmipValues.V3_0_OPERATION_CREATE_USER, createUser.operation());
        }

        UniqueIdentifier identifier = UniqueIdentifier.textString("system-object");
        try (ClientBatchItem obliterate = ClientBatchItem.obliterateFromIdentifier(identifier);
             ClientBatchItem deactivate = ClientBatchItem.deactivateFromFields(identifier, null, null)) {
            assertEquals(KmipValues.V3_0_OPERATION_OBLITERATE, obliterate.operation());
            assertEquals(KmipValues.V3_0_OPERATION_DEACTIVATE, deactivate.operation());
        }
    }

    @Test
    void buildsCredentialAndKemOperationsFromTypedInputs() {
        CodecLimits limits = CodecLimits.defaults();
        try (RawTag identifierRawTag = RawTag.fromRaw(0x004200A2);
             Tag identifierTag = identifierRawTag.checked();
             TtlvStructure emptyFields = TtlvStructure.create();
             TtlvValue identifierValue = TtlvValue.TextString(
                     "device-id".getBytes(StandardCharsets.UTF_8), limits);
             TtlvItem identifierItem = TtlvItem.create(identifierTag, identifierValue, limits);
             TtlvStructure fields = TtlvStructure.withItem(emptyFields, identifierItem, limits);
             RawTag rawTag = RawTag.fromRaw(0x004201A6);
             Tag tag = rawTag.checked();
             TtlvValue value = TtlvValue.structure(fields, limits);
             TtlvItem object = TtlvItem.create(tag, value, limits);
             TtlvStructure empty = TtlvStructure.create();
             AttributeSet attributes = AttributeSet.tryNew(empty);
             ClientBatchItem createCredential = ClientBatchItem.createCredentialFromFields(
                     CredentialType.fromRaw(CredentialType.DEVICE),
                     KmipValues.V3_0_OBJECT_TYPE_DEVICE_CREDENTIAL, attributes, object);
             ClientBatchItem encapsulate = ClientBatchItem.encapsulateFromFields(
                     UniqueIdentifier.textString("kem-key"), null, attributes);
             ClientBatchItem decapsulate = ClientBatchItem.decapsulateFromFields(
                     UniqueIdentifier.textString("kem-key"), new byte[] {0x31, 0x32}, null, attributes)) {
            assertEquals(KmipValues.V3_0_OPERATION_CREATE_CREDENTIAL, createCredential.operation());
            assertThrows(InvalidInputException.class, () ->
                    ClientBatchItem.createCredentialFromFields(
                            CredentialType.fromRaw(CredentialType.DEVICE),
                            KmipValues.V3_0_OBJECT_TYPE_PASSWORD_CREDENTIAL,
                            attributes, object));
            assertEquals(KmipValues.V3_0_OPERATION_ENCAPSULATE, encapsulate.operation());
            assertEquals(KmipValues.V3_0_OPERATION_DECAPSULATE, decapsulate.operation());
        }
    }
}
