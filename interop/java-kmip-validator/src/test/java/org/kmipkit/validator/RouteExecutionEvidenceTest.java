package org.kmipkit.validator;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.jupiter.api.Test;
import org.kmipkit.ttlv.TtlvCodec;
import org.kmipkit.ttlv.TtlvDecodeErrorKind;
import org.kmipkit.ttlv.TtlvDecodeException;

import java.nio.file.Files;
import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assertions.assertThrows;

class RouteExecutionEvidenceTest {
    private static final byte[] UNALLOCATED_TAG_INTEGER = {
            0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00
    };

    @Test
    void recordsInstalledDecoderCallsForTheUnallocatedTagCaseAndBothProtocolVersions() throws Exception {
        for (String protocolVersion : new String[] {"2.0", "2.1"}) {
            TtlvDecodeException rejection = assertThrows(TtlvDecodeException.class,
                    () -> TtlvCodec.decode(UNALLOCATED_TAG_INTEGER));
            assertEquals(TtlvDecodeErrorKind.UNALLOCATED_TAG, rejection.kind());
        }

        Path reportPath = Path.of(System.getProperty("kmipkit.routeEvidenceOutput",
                "build/route-execution-evidence.json"));
        assertTrue(Files.isRegularFile(reportPath),
                "The installed API invocation must produce route execution evidence.");
        JsonNode report = new ObjectMapper().readTree(reportPath.toFile());
        assertEquals("java", report.path("language").asText());
        assertEquals("test-norm-tag-range-kmipkit-range-001", report.path("case_ids").get(0).asText());
        assertEquals(2, report.path("events").size());
        assertEquals("org.kmipkit.ttlv.TtlvCodec.decode",
                report.path("events").get(0).path("public_api_symbol").asText());
    }
}
