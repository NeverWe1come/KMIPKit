package org.kmipkit.fluent;

import java.util.List;
import java.util.Optional;
import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.UniqueIdentifier;

/** Page-by-page paginator for KMIP Locate using Maximum Items and Offset Items. */
public final class LocatePaginator {
    private final KmipClient client;
    private final AttributeSet attributes;
    private final int pageSize;
    private int offset;
    private boolean exhausted;

    public LocatePaginator(KmipClient client, AttributeSet attributes, int pageSize) {
        if (client == null || attributes == null) {
            throw new InvalidInputException("client and attributes are required");
        }
        this.client = client;
        this.attributes = attributes;
        this.pageSize = Math.max(1, pageSize);
    }

    public Optional<List<UniqueIdentifier>> nextPage() throws org.kmipkit.ClientOperationException {
        if (exhausted) return Optional.empty();
        try (LocateBuilder b = new LocateBuilder(client, attributes)
                .maximumItems(pageSize)
                .offsetItems(offset);
             ClientBatchResponse resp = b.execute(client.codecLimits());
             ClientBatchItemResult item = resp.itemAt(0)) {
            List<UniqueIdentifier> ids = item.locateResponse()
                    .map(ClientBatchItemResult.LocateResponse::uniqueIdentifiers)
                    .orElse(List.of());
            offset += ids.size();
            if (ids.size() < pageSize) {
                exhausted = true;
            }
            return ids.isEmpty() ? Optional.empty() : Optional.of(ids);
        }
    }
}
