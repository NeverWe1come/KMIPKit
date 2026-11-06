package org.kmipkit.typed;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.util.List;
import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.RegisterRequest;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.ManagedObject;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;

final class ManagedTtlvItemTransferTest {
    @Test
    void transfersTypedObjectIntoRegisterRequestAsItsDirectStructureItem() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvItem opaqueDataType = item(0x0042_0059, TtlvValue.Enumeration(2), limits);
        TtlvItem opaqueDataValue = item(
                0x0042_005a, TtlvValue.ByteString(new byte[] {0x4f, 0x50, 0x41, 0x51}, limits), limits);
        try (opaqueDataType;
                opaqueDataValue;
                ManagedObject object = ManagedObject.opaqueObject(List.of(opaqueDataType, opaqueDataValue));
                TtlvStructure attributes = TtlvStructure.create()) {
            TtlvItem item = object.intoItem();
            assertThrows(InvalidInputException.class, object::view);
            try (item) {
                try (TtlvItemView view = item.view(); Tag tag = view.tag()) {
                    assertEquals(0x0042_005b, tag.raw());
                }
                try (RegisterRequest request = RegisterRequest.create(8, attributes, item);
                        ClientBatchItem batchItem = ClientBatchItem.register(request)) {
                    assertEquals(3, batchItem.operation());
                    assertNotNull(batchItem);
                }
            }
        }
    }

    private static TtlvItem item(int rawTag, TtlvValue value, CodecLimits limits) {
        try (RawTag raw = RawTag.fromRaw(rawTag); Tag tag = raw.checked()) {
            return TtlvItem.create(tag, value, limits);
        }
    }
}
