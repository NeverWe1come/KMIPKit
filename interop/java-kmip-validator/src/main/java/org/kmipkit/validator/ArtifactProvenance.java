package org.kmipkit.validator;

import com.fasterxml.jackson.core.StreamReadFeature;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.HexFormat;

/** Verified provenance for the independently installed native Java package. */
public record ArtifactProvenance(String libraryCommit, String artifactSha256,
                                 String mavenCoordinate, String nativeClassifier, Path artifactPath) {
    private static final String RELEASE_REF = "release/1.0.0";
    private static final String PACKAGE_VERSION = "1.0.0";
    private static final ObjectMapper JSON = new ObjectMapper(
            com.fasterxml.jackson.core.JsonFactory.builder()
                    .enable(StreamReadFeature.STRICT_DUPLICATE_DETECTION)
                    .build());

    /** Loads the installer manifest and verifies the release pin, host classifier, and JAR bytes. */
    public static ArtifactProvenance load(Path manifestPath, String expectedCommit) throws IOException {
        require(expectedCommit != null && expectedCommit.matches("[0-9a-f]{40}"),
                "Expected release commit is invalid");
        JsonNode manifest = JSON.readTree(manifestPath.toFile());
        require(manifest != null && manifest.isObject(), "Artifact manifest must be an object");
        require("1.0".equals(text(manifest, "schema_version")), "Artifact manifest schema is unsupported");
        require(RELEASE_REF.equals(text(manifest, "release_ref")), "Artifact was not built from release/1.0.0");
        String commit = text(manifest, "library_commit");
        require(expectedCommit.equals(commit), "Installed artifact commit differs from the exact release pin");
        String classifier = text(manifest, "native_classifier");
        require(hostClassifier().equals(classifier), "Installed native artifact does not match this host");
        String coordinate = text(manifest, "maven_coordinate");
        String expectedCoordinate = "org.kmipkit:kmipkit-java:" + PACKAGE_VERSION + ":" + classifier;
        require(expectedCoordinate.equals(coordinate), "Installed artifact Maven coordinate is invalid");
        String digest = text(manifest, "artifact_sha256");
        require(digest.matches("[a-f0-9]{64}"), "Installed artifact SHA-256 is invalid");
        Path artifact = Path.of(text(manifest, "artifact_file")).toAbsolutePath().normalize();
        require(Files.isRegularFile(artifact) && Files.size(artifact) > 0,
                "Installed artifact is missing or empty");
        require(digest.equals(sha256(artifact)), "Installed artifact bytes do not match their SHA-256");
        return new ArtifactProvenance(commit, digest, coordinate, classifier, artifact);
    }

    /** Calculates the SHA-256 digest of a file using lowercase hexadecimal. */
    public static String sha256(Path path) throws IOException {
        final MessageDigest digest;
        try {
            digest = MessageDigest.getInstance("SHA-256");
        } catch (NoSuchAlgorithmException impossible) {
            throw new IllegalStateException("The Java runtime does not provide SHA-256", impossible);
        }
        try (var input = Files.newInputStream(path)) {
            byte[] buffer = new byte[32 * 1024];
            int count;
            while ((count = input.read(buffer)) >= 0) {
                if (count > 0) digest.update(buffer, 0, count);
            }
        }
        return HexFormat.of().formatHex(digest.digest());
    }

    /** Returns the native classifier used by the KMIPKit Java package on this host. */
    public static String hostClassifier() {
        String os = System.getProperty("os.name").toLowerCase();
        String machine = System.getProperty("os.arch").toLowerCase();
        String arch = machine.equals("amd64") || machine.equals("x86_64") ? "x86_64"
                : machine.equals("aarch64") || machine.equals("arm64") ? "aarch64" : "";
        if (os.startsWith("windows") && arch.equals("x86_64")) return "windows-x86_64-msvc";
        if (os.equals("linux") && !arch.isEmpty()) return "linux-glibc-" + arch;
        if ((os.startsWith("mac") || os.equals("darwin")) && !arch.isEmpty()) return "macos-" + arch;
        throw new IllegalStateException("This host is outside the KMIPKit Java native target matrix");
    }

    private static String text(JsonNode node, String field) {
        JsonNode value = node.get(field);
        require(value != null && value.isTextual() && !value.textValue().isBlank(),
                "Artifact manifest field " + field + " is invalid");
        return value.textValue();
    }

    private static void require(boolean condition, String message) {
        if (!condition) throw new IllegalArgumentException(message);
    }
}
