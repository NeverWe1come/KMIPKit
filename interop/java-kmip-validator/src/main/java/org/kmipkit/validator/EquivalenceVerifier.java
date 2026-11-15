package org.kmipkit.validator;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Objects;

public class EquivalenceVerifier {
    public record NormalizedResponse(
            String status,
            String resultReason,
            String objectType,
            String sanitizedPayloadHex
    ) {}

    public record EquivalenceResult(boolean isEquivalent, List<String> discrepancies) {}

    public static EquivalenceResult verifyEquivalence(Map<String, NormalizedResponse> responses) {
        List<String> discrepancies = new ArrayList<>();
        if (responses == null || responses.size() <= 1) {
            return new EquivalenceResult(true, discrepancies);
        }

        var entryList = new ArrayList<>(responses.entrySet());
        var baseline = entryList.get(0);
        String baseLabel = baseline.getKey();
        NormalizedResponse baseResp = baseline.getValue();

        for (int i = 1; i < entryList.size(); i++) {
            var cur = entryList.get(i);
            String label = cur.getKey();
            NormalizedResponse resp = cur.getValue();

            if (!Objects.equals(baseResp.status(), resp.status())) {
                discrepancies.add("[" + label + "] status '" + resp.status() + "' differs from [" + baseLabel + "] '" + baseResp.status() + "'");
            }
            if (!Objects.equals(baseResp.resultReason(), resp.resultReason())) {
                discrepancies.add("[" + label + "] resultReason '" + resp.resultReason() + "' differs from [" + baseLabel + "] '" + baseResp.resultReason() + "'");
            }
            if (!Objects.equals(baseResp.objectType(), resp.objectType())) {
                discrepancies.add("[" + label + "] objectType '" + resp.objectType() + "' differs from [" + baseLabel + "] '" + baseResp.objectType() + "'");
            }
            if (!Objects.equals(baseResp.sanitizedPayloadHex(), resp.sanitizedPayloadHex())) {
                discrepancies.add("[" + label + "] payload differs from [" + baseLabel + "]");
            }
        }

        return new EquivalenceResult(discrepancies.isEmpty(), discrepancies);
    }
}
