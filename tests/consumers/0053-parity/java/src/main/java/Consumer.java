import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;

public final class Consumer {
    private Consumer() {}

    public static void main(String[] args) {
        try (ClientBatchItem ping = ClientBatchItem.ping();
                TtlvStructure payload = TtlvStructure.create();
                TtlvStructureView view = payload.view()) {
            if (ping.operation() != 0x3b || view.itemCount() != 0) {
                throw new IllegalStateException("KMIPKit consumer contract failed");
            }
        }
    }
}
