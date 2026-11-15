package org.kmipkit.validator;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.jupiter.api.Test;

import java.time.Instant;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;

class ReportWriterTest {
    @Test
    void emitsTheSharedSchemaAndDoesNotAcceptRemotePayloadText() throws Exception {
        String digest = "a".repeat(64);
        var evidence = new RunReport.CaseEvidence("test-op-ping", "passed", "passed", "supported",
                "not_compared", 4.2, 0L, null, null);
        var report = new RunReport("2.0", "java-run", Instant.parse("2026-11-10T12:00:00Z").toString(),
                "java", new RunReport.Server("demo", "KMIP server", "2.1", digest), "2.1", digest,
                new RunReport.Execution("raw_tls", "TTLV", "typed-protocol", "9".repeat(40), digest),
                new RunReport.Summary(1, 1, 0, 0, 0, 0), List.of(evidence));

        String json = MatrixReporter.toJson(report);
        var tree = new ObjectMapper().readTree(json);
        assertEquals("2.0", tree.path("schema_version").asText());
        assertEquals("java", tree.path("language").asText());
        assertEquals("raw_tls", tree.path("execution").path("transport").asText());
        assertEquals("passed", tree.path("cases").get(0).path("status").asText());
        assertEquals(1, tree.path("summary").path("total").asInt());
        assertFalse(tree.path("cases").get(0).has("result_reason_code"));
        assertFalse(json.contains("KMIP body"));
    }

    @Test
    void rejectsInconsistentSummaryCountsBeforeWriting() {
        String digest = "b".repeat(64);
        var report = new RunReport("2.0", "java-run", Instant.now().toString(), "java",
                new RunReport.Server("demo", "KMIP server", "2.1", digest), "2.1", digest,
                new RunReport.Execution("raw_tls", "TTLV", "typed-protocol", "9".repeat(40), digest),
                new RunReport.Summary(2, 0, 0, 0, 0, 0), List.of());
        org.junit.jupiter.api.Assertions.assertThrows(IllegalArgumentException.class,
                () -> MatrixReporter.toJson(report));
    }
}
