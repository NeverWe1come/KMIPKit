package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;

import org.junit.jupiter.api.Test;
import org.kmipkit.protocol.KmipValues;
import org.kmipkit.ttlv.TtlvStructure;

final class Kmip30SemanticBatchItemTest {
    @Test
    void buildsSemanticSystemObjectOperationsFromTypedInputs() {
        try (TtlvStructure empty = TtlvStructure.create();
             AttributeSet attributes = AttributeSet.tryNew(empty);
             ClientBatchItem createGroup = ClientBatchItem.createGroup(attributes);
             ClientBatchItem createUser = ClientBatchItem.createUser(attributes)) {
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
}
