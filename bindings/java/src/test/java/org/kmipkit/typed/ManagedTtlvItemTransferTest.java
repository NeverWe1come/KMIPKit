package org.kmipkit.typed;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.util.List;
import org.junit.jupiter.api.Test;
import org.kmipkit.extensions.RegisterRequest;
import org.kmipkit.InvalidInputException;
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
        TtlvItem certificateType = item(0x0042_001d, TtlvValue.Enumeration(2), limits);
        TtlvItem certificateValue = item(
                0x0042_001e, TtlvValue.ByteString(new byte[] {0x43, 0x45, 0x52, 0x54}, limits), limits);
        try (certificateType;
                certificateValue;
                ManagedObject object = ManagedObject.certificate(List.of(certificateType, certificateValue));
                TtlvStructure attributes = TtlvStructure.create()) {
            TtlvItem item = object.intoItem();
            assertThrows(InvalidInputException.class, object::view);
            try (item; TtlvItemView view = item.view(); Tag tag = view.tag()) {
                assertEquals(0x0042_0013, tag.raw());
                try (RegisterRequest request = RegisterRequest.create(1, attributes, item)) {
                    assertNotNull(request);
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
