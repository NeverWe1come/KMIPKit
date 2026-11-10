package org.kmipkit.validator;

import com.fasterxml.jackson.annotation.JsonInclude;

import java.util.List;

/** Immutable run evidence matching interop/shared/schemas/validation-results.schema.json. */
@JsonInclude(JsonInclude.Include.NON_NULL)
public record RunReport(String schemaVersion, String runId, String testedAt, String language,
                        Server server, String protocolVersion, String catalogSha256,
                        Execution execution, Summary summary, List<CaseEvidence> cases) {
    public RunReport {
        cases = List.copyOf(cases);
    }

    public record Server(String id, String product, String version, String configurationFingerprint) { }

    public record Execution(String transport, String encoding, String apiLevel,
                            String libraryCommit, String artifactSha256) { }

    public record Summary(int total, int passed, int failed, int unsupported,
                          int policyRestricted, int notRun) { }

    @JsonInclude(JsonInclude.Include.NON_NULL)
    public record CaseEvidence(String caseId, String status, String libraryCorrectness,
                               String serverCompatibility, String parity, double durationMs,
                               Long resultStatusCode, Long resultReasonCode, String diagnostic) { }
}
