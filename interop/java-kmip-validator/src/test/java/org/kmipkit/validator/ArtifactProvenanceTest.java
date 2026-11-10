package org.kmipkit.validator;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.junit.jupiter.api.Test;
import org.kmipkit.extensions.ClientBatchItem;

import java.nio.file.Files;
import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

class ArtifactProvenanceTest {
    private static final String COMMIT = "9692057c9ade474fb0c4e0b7130539da368eb794";

    @Test
    void verifiesPinnedCommitAndJarDigest() throws Exception {
        Path directory = Files.createTempDirectory("java-artifact-provenance");
        Path jar = directory.resolve("kmipkit-java.jar");
        Files.write(jar, new byte[] {1, 2, 3, 4});
        Path manifest = writeManifest(directory, jar, COMMIT);

        var provenance = ArtifactProvenance.load(manifest, COMMIT);

        assertEquals(COMMIT, provenance.libraryCommit());
        assertEquals(jar.toAbsolutePath(), provenance.artifactPath());
        assertEquals("org.kmipkit:kmipkit-java:1.0.0:" + ArtifactProvenance.hostClassifier(),
                provenance.mavenCoordinate());
        assertThrows(IllegalArgumentException.class, () -> ArtifactProvenance.load(manifest, "0".repeat(40)));

        Files.write(jar, new byte[] {4, 3, 2, 1});
        assertThrows(IllegalArgumentException.class, () -> ArtifactProvenance.load(manifest, COMMIT));
    }

    @Test
    void executesPublicApiFromTheInstalledJar() throws Exception {
        Path manifest = Path.of(System.getProperty("kmipkit.artifactManifest"));
        var provenance = ArtifactProvenance.load(manifest, COMMIT);
        Path loadedJar = Path.of(ClientBatchItem.class.getProtectionDomain().getCodeSource().getLocation().toURI())
                .toAbsolutePath().normalize();
        assertEquals(provenance.artifactPath(), loadedJar);
        try (ClientBatchItem ping = ClientBatchItem.ping()) {
            assertEquals(0, ping.extensionCount());
        }
    }

    private static Path writeManifest(Path directory, Path jar, String commit) throws Exception {
        ObjectNode manifest = new ObjectMapper().createObjectNode();
        manifest.put("schema_version", "1.0");
        manifest.put("release_ref", "release/1.0.0");
        manifest.put("library_commit", commit);
        manifest.put("maven_coordinate", "org.kmipkit:kmipkit-java:1.0.0:" + ArtifactProvenance.hostClassifier());
        manifest.put("native_classifier", ArtifactProvenance.hostClassifier());
        manifest.put("artifact_file", jar.toAbsolutePath().toString());
        manifest.put("artifact_sha256", ArtifactProvenance.sha256(jar));
        Path path = directory.resolve("manifest.json");
        new ObjectMapper().writeValue(path.toFile(), manifest);
        return path;
    }
}
