package org.kmipkit.validator;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.jupiter.api.Test;
import org.kmipkit.ttlv.TtlvCodec;
import org.kmipkit.ttlv.TtlvDecodeErrorKind;
import org.kmipkit.ttlv.TtlvDecodeException;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.util.HexFormat;
import java.util.UUID;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assertions.assertThrows;

class RouteExecutionEvidenceTest {
    private static final byte[] UNALLOCATED_TAG_INTEGER = {
            0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00
    };

    @Test
    void recordsInstalledDecoderCallsForTheUnallocatedTagCaseAndBothProtocolVersions() throws Exception {
        var provenance = ArtifactProvenance.load(Path.of(System.getProperty("kmipkit.artifactManifest")),
                "9a16cc90a12205101d22a38501a5e74c665df4dc");
        Path loadedJar = Path.of(TtlvCodec.class.getProtectionDomain().getCodeSource().getLocation().toURI())
                .toAbsolutePath().normalize();
        assertEquals(provenance.artifactPath(), loadedJar,
                "The decoder invocation must resolve to the independently installed package JAR.");

        for (String protocolVersion : new String[] {"2.0", "2.1"}) {
            TtlvDecodeException rejection = assertThrows(TtlvDecodeException.class,
                    () -> TtlvCodec.decode(UNALLOCATED_TAG_INTEGER));
            assertEquals(TtlvDecodeErrorKind.UNALLOCATED_TAG, rejection.kind());
        }

        Path reportPath = Path.of(System.getProperty("kmipkit.routeEvidenceOutput",
                "build/route-execution-evidence.json"));
        writeRouteEvidence(reportPath);
        assertTrue(Files.isRegularFile(reportPath),
                "The installed API invocation must produce route execution evidence.");
        JsonNode report = new ObjectMapper().readTree(reportPath.toFile());
        assertEquals("java", report.path("language").asText());
        assertEquals("test-norm-tag-range-kmipkit-range-001", report.path("case_ids").get(0).asText());
        assertEquals(2, report.path("events").size());
        assertEquals("org.kmipkit.ttlv.TtlvCodec.decode",
                report.path("events").get(0).path("public_api_symbol").asText());
    }

    private static void writeRouteEvidence(Path reportPath) throws Exception {
        ObjectMapper mapper = new ObjectMapper();
        Path catalogPath = Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json");
        Path manifestPath = Path.of("../shared/coverage/route-capabilities.json");
        Path pinPath = Path.of("library-pin.json");
        String libraryCommit = mapper.readTree(pinPath.toFile()).path("commit").asText();
        var artifact = ArtifactProvenance.load(Path.of(System.getProperty("kmipkit.artifactManifest")), libraryCommit);

        ObjectNode report = mapper.createObjectNode();
        report.put("schema_version", "1.0");
        report.put("run_id", "java-local-tag-range-" + UUID.randomUUID());
        report.put("language", "java");
        ObjectNode server = report.putObject("server");
        server.put("id", "local-no-peer");
        server.put("product", "local Java TTLV decoder");
        server.put("version", "none");
        server.put("configuration_fingerprint", sha256("java-local-ttlv-unallocated-tag"));
        report.put("library_commit", artifact.libraryCommit());
        report.put("artifact_sha256", artifact.artifactSha256());
        report.put("catalog_sha256", ArtifactProvenance.sha256(catalogPath));
        report.put("route_manifest_sha256", ArtifactProvenance.sha256(manifestPath));
        report.putArray("case_ids").add("test-norm-tag-range-kmipkit-range-001");
        report.putArray("protocol_versions").add("2.0").add("2.1");
        ObjectNode combination = mapper.createObjectNode();
        combination.put("encoding", "TTLV");
        combination.put("transport", "raw_tls");
        report.putArray("network_combinations").add(combination);

        ArrayNode events = report.putArray("events");
        for (String protocolVersion : new String[] {"2.0", "2.1"}) {
            ObjectNode event = events.addObject();
            event.put("case_id", "test-norm-tag-range-kmipkit-range-001");
            event.put("route_id", "java.generic-ttlv.ttlv_codec.decode");
            event.put("target_id", "KMIPKIT-RANGE-001");
            event.put("public_api_symbol", "org.kmipkit.ttlv.TtlvCodec.decode");
            event.putNull("execution_route_id");
            event.putNull("execution_api_symbol");
            event.put("invocation_id", UUID.randomUUID().toString());
            event.put("action", "decode");
            event.put("protocol_version", protocolVersion);
            event.put("encoding", "TTLV");
            event.putNull("transport");
            event.put("status", "passed");
            event.put("invocation_count", 1);
            event.put("decode_outcome", "rejected");
            event.put("decode_rejection_code", "tag_not_allocated");
            event.put("decode_input_sha256", sha256(UNALLOCATED_TAG_INTEGER));
        }
        Files.createDirectories(reportPath.toAbsolutePath().normalize().getParent());
        mapper.writerWithDefaultPrettyPrinter().writeValue(reportPath.toFile(), report);
    }

    private static String sha256(String value) throws Exception {
        return sha256(value.getBytes(StandardCharsets.UTF_8));
    }

    private static String sha256(byte[] value) throws Exception {
        return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(value));
    }
}
