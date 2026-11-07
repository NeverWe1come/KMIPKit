import java.util.List;
import org.kmipkit.credentials.Authentication;
import org.kmipkit.credentials.Credential;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.message.ResponseMessage;
import org.kmipkit.protocol.CryptographicParameters;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.KeyMaterial;
import org.kmipkit.ttlv.ManagedObject;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.StandardAttribute;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TypedAttribute;

public final class Consumer {
    private Consumer() {}

    private static TtlvItem item(int rawTag, TtlvValue value, CodecLimits limits) {
        try (RawTag raw = RawTag.fromRaw(rawTag); Tag tag = raw.checked()) {
            return TtlvItem.create(tag, value, limits);
        }
    }

    private static TtlvStructure append(
            TtlvStructure structure, int rawTag, TtlvValue value, CodecLimits limits) {
        return TtlvStructure.withItem(structure, item(rawTag, value, limits), limits);
    }

    private static TtlvStructure syntheticPingResponse(CodecLimits limits) {
        TtlvStructure version = TtlvStructure.create();
        version = append(version, 0x42006a, TtlvValue.Integer(2), limits);
        version = append(version, 0x42006b, TtlvValue.Integer(1), limits);
        TtlvStructure header = TtlvStructure.create();
        header = append(header, 0x420069, TtlvValue.structure(version, limits), limits);
        header = append(header, 0x420092, TtlvValue.DateTime(1_751_100_123L), limits);
        header = append(header, 0x42000d, TtlvValue.Integer(1), limits);
        TtlvStructure responseItem = TtlvStructure.create();
        responseItem = append(responseItem, 0x42005c, TtlvValue.Enumeration(0x3b), limits);
        responseItem = append(responseItem, 0x42007f, TtlvValue.Enumeration(0), limits);
        responseItem = append(
                responseItem, 0x42007c,
                TtlvValue.structure(TtlvStructure.create(), limits), limits);
        TtlvStructure root = TtlvStructure.create();
        root = append(root, 0x42007a, TtlvValue.structure(header, limits), limits);
        return append(root, 0x42000f, TtlvValue.structure(responseItem, limits), limits);
    }

    private static Authentication demoAuthentication(CodecLimits limits) {
        TtlvStructure credentialValue = TtlvStructure.create();
        credentialValue = append(
                credentialValue, 0x4200bb,
                TtlvValue.TextString("demo-user".getBytes(), limits), limits);
        TtlvStructure credential = TtlvStructure.create();
        credential = append(credential, 0x420024, TtlvValue.Enumeration(1), limits);
        credential = append(
                credential, 0x420025,
                TtlvValue.structure(credentialValue, limits), limits);
        return Authentication.create(List.of(Credential.fromTtlv(credential)));
    }

    public static void main(String[] args) {
        CodecLimits limits = CodecLimits.defaults();
        try (TtlvValue commentValue = TtlvValue.TextString("local example".getBytes(), limits);
                TypedAttribute attribute = TypedAttribute.create(
                        StandardAttribute.CONTACT_INFORMATION, commentValue);
                CryptographicParameters.Builder parameterBuilder = CryptographicParameters.builder();
                CryptographicParameters parameters = parameterBuilder
                        .cryptographicAlgorithm(0x8000_0001L).build()) {
            if (attribute.attribute() != StandardAttribute.CONTACT_INFORMATION
                    || parameters.cryptographicAlgorithm().orElseThrow() != 0x8000_0001L) {
                throw new IllegalStateException("typed package example failed");
            }

            try (TtlvItem dataType = item(0x420059, TtlvValue.Enumeration(1), limits);
                    TtlvItem dataValue = item(0x42005a, TtlvValue.ByteString("demo bytes".getBytes(), limits), limits);
                    ManagedObject managedObject = ManagedObject.opaqueObject(List.of(dataType, dataValue))) {
                if (managedObject.objectType() != 8) throw new IllegalStateException("object type mismatch");
            }

            try (TtlvItem material = item(0x420043, TtlvValue.ByteString("demo key bytes".getBytes(), limits), limits);
                    KeyMaterial keyMaterial = KeyMaterial.fromItem(1, material)) {
                if (keyMaterial.formatRaw() != 1) throw new IllegalStateException("key format mismatch");
            }

            ClientBatchItem ping = ClientBatchItem.ping();
            try (var report = ping.validationReport()) {
                if (!report.isValid()) throw new IllegalStateException("Ping did not validate");
            }
            try (ClientBatch batch = ClientBatch.create()) {
                batch.add(ping).setAuthentication(demoAuthentication(limits));
            }

            try (ResponseMessage response = ResponseMessage.fromTtlv(syntheticPingResponse(limits));
                    var item = response.batchItemAt(0)) {
                if (response.batchItemCount() != 1 || item.resultStatus().orElseThrow() != 0) {
                    throw new IllegalStateException("synthetic response did not parse");
                }
                try (TtlvStructure responseTtlv = item.ttlv(); var view = responseTtlv.view()) {
                    if (view.itemCount() != 3) throw new IllegalStateException("response TTLV mismatch");
                }
            }
        }
    }
}
