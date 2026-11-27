package org.kmipkit.fluent;

import static org.junit.jupiter.api.Assertions.*;

import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import org.junit.jupiter.api.Test;
import org.kmipkit.extensions.AttributeReference;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.TtlvCodec;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;

final class FluentParity64JavaTest {
    private static byte[] encodeItem(int tag, int type, byte[] value) {
        int pad = (8 - (value.length % 8)) % 8;
        ByteBuffer buf = ByteBuffer.allocate(8 + value.length + pad);
        buf.put((byte) ((tag >> 16) & 0xff));
        buf.put((byte) ((tag >> 8) & 0xff));
        buf.put((byte) (tag & 0xff));
        buf.put((byte) type);
        buf.putInt(value.length);
        buf.put(value);
        for (int i = 0; i < pad; i++) {
            buf.put((byte) 0);
        }
        return buf.array();
    }

    private static TtlvStructure structureFromChildren(byte[]... childWires) {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure s = TtlvStructure.create();
        for (byte[] wire : childWires) {
            TtlvItem child = TtlvCodec.decode(wire);
            s = TtlvStructure.withItem(s, child, limits);
        }
        return s;
    }

    private static TtlvStructure structureFromSingleEnum(int childTag, int enumValue) {
        return structureFromChildren(
                encodeItem(childTag, 0x05, ByteBuffer.allocate(4).putInt(enumValue).array()));
    }

    @Test
    void allSixteenNewJavaBuildersConstructValidBatchItems() throws Exception {
        UniqueIdentifier uid = UniqueIdentifier.textString("obj-1");

        try (TtlvValue adjVal = TtlvValue.Integer(10);
                AdjustAttributeBuilder b = new AdjustAttributeBuilder(null)
                        .uniqueIdentifier(uid)
                        .attributeReference(AttributeReference.tag(0x420008L))
                        .adjustmentType(1L)
                        .adjustmentValue(adjVal);
                ClientBatchItem item = b.build()) {
            assertEquals(0x30L, item.operation());
        }

        try (GetConstraintsBuilder b = new GetConstraintsBuilder(null);
                ClientBatchItem item = b.build()) {
            assertEquals(0x38L, item.operation());
        }

        try (CancelBuilder b = new CancelBuilder(null, new byte[] {1, 2});
                ClientBatchItem item = b.build()) {
            assertEquals(0x19L, item.operation());
        }

        try (PollBuilder b = new PollBuilder(null, new byte[] {1, 2});
                ClientBatchItem item = b.build()) {
            assertEquals(0x1aL, item.operation());
        }

        try (ProcessBuilder b = new ProcessBuilder(null, new byte[] {1, 2});
                ClientBatchItem item = b.build()) {
            assertEquals(0x3aL, item.operation());
        }

        try (QueryAsyncRequestsBuilder b = new QueryAsyncRequestsBuilder(null)
                        .correlationValue(new byte[] {1, 2})
                        .operation(1L);
                ClientBatchItem item = b.build()) {
            assertEquals(0x39L, item.operation());
        }

        try (LoginBuilder b = new LoginBuilder(null)
                        .leaseTimeInterval(120L)
                        .requestCount(4);
                ClientBatchItem item = b.build()) {
            assertEquals(0x2dL, item.operation());
        }

        try (TtlvStructure ticket = structureFromChildren(
                        encodeItem(0x42014A, 0x05, ByteBuffer.allocate(4).putInt(1).array()),
                        encodeItem(0x42014B, 0x08, "ticket".getBytes(StandardCharsets.UTF_8)));
                LogoutBuilder b = new LogoutBuilder(null, ticket);
                ClientBatchItem item = b.build()) {
            assertEquals(0x2eL, item.operation());
        }

        try (TtlvStructure rights = structureFromSingleEnum(0x42014E, 1);
                DelegatedLoginBuilder b = new DelegatedLoginBuilder(null, rights)
                        .leaseTimeDateTime(1_700_000_000L);
                ClientBatchItem item = b.build()) {
            assertEquals(0x2fL, item.operation());
        }

        try (SetEndpointRoleBuilder b = new SetEndpointRoleBuilder(null, 1L, "KMIPKit");
                ClientBatchItem item = b.build()) {
            assertEquals(0x32L, item.operation());
        }

        try (TtlvStructure fnStruct = structureFromSingleEnum(0x42015A, 1);
                Pkcs11Builder b = new Pkcs11Builder(null, fnStruct);
                ClientBatchItem item = b.build()) {
            assertEquals(0x33L, item.operation());
        }

        try (InteropBuilder b = new InteropBuilder(null, 1L, null).interopIdentifier("op-id");
                ClientBatchItem item = b.build()) {
            assertEquals(0x34L, item.operation());
        }

        try (ReProvisionBuilder b = new ReProvisionBuilder(null);
                ClientBatchItem item = b.build()) {
            assertEquals(0x35L, item.operation());
        }

        try (SetDefaultsBuilder b = new SetDefaultsBuilder(null, null);
                ClientBatchItem item = b.build()) {
            assertEquals(0x36L, item.operation());
        }

        try (TtlvStructure constraints = TtlvStructure.create();
                SetConstraintsBuilder b = new SetConstraintsBuilder(null, constraints);
                ClientBatchItem item = b.build()) {
            assertEquals(0x37L, item.operation());
        }

        try (LogBuilder b = new LogBuilder(null, null).logMessage("audit-msg");
                ClientBatchItem item = b.build()) {
            assertEquals(0x2cL, item.operation());
        }
    }
}
