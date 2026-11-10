package org.kmipkit.validator;

import com.fasterxml.jackson.core.StreamReadFeature;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

import java.io.IOException;
import java.net.URI;
import java.nio.file.Path;
import java.util.Set;

/** Validated server connection and opt-in policy. Loading this value is offline. */
public record ConnectionConfig(
        String serverId,
        String configurationFingerprint,
        String serverProduct,
        String serverVersion,
        String protocolVersion,
        String transport,
        String encoding,
        String apiLevel,
        String host,
        int port,
        URI httpsUri,
        Tls tls,
        UsernamePassword usernamePassword,
        boolean dedicatedTestEnvironmentConfirmed,
        boolean allowMutations,
        boolean probeUnsupported,
        boolean cleanupOnFailure) {

    private static final Set<String> VERSIONS = Set.of("2.0", "2.1");
    private static final Set<String> TRANSPORTS = Set.of("raw_tls", "https");
    private static final Set<String> ENCODINGS = Set.of("TTLV", "JSON", "XML");
    private static final Set<String> API_LEVELS = Set.of("high-level", "typed-protocol", "generic-ttlv");
    private static final ObjectMapper JSON = new ObjectMapper(
            com.fasterxml.jackson.core.JsonFactory.builder()
                    .enable(StreamReadFeature.STRICT_DUPLICATE_DETECTION)
                    .build());

    /** Reads the shared connection contract and checks every field used by a live exchange. */
    public static ConnectionConfig load(Path path) throws IOException {
        JsonNode root = JSON.readTree(path.toFile());
        require(root != null && root.isObject(), "Connection config must be an object");
        String serverId = text(root, "server_id", "config");
        require(serverId.matches("[a-zA-Z0-9_-]+"), "server_id is invalid");
        String fingerprint = text(root, "server_configuration_fingerprint", "config");
        require(fingerprint.matches("[a-f0-9]{64}"), "server_configuration_fingerprint is invalid");
        String serverProduct = optionalText(root, "server_product");
        if (serverProduct.isBlank()) serverProduct = "Unspecified KMIP server";
        String serverVersion = optionalText(root, "server_version");
        if (serverVersion.isBlank()) serverVersion = "not-recorded";
        String version = text(root, "protocol_version", "config");
        require(VERSIONS.contains(version), "Only KMIP 2.0 and 2.1 are supported");
        String transport = text(root, "transport", "config");
        require(TRANSPORTS.contains(transport), "transport must be raw_tls or https");
        String encoding = text(root, "encoding", "config");
        require(ENCODINGS.contains(encoding), "encoding must be TTLV, JSON, or XML");
        String apiLevel = text(root, "api_level", "config");
        require(API_LEVELS.contains(apiLevel), "api_level is invalid");

        Endpoint endpoint = parseEndpoint(root, transport);
        Tls tls = parseTls(root);
        UsernamePassword usernamePassword = parseUsernamePassword(root);

        JsonNode environment = object(root, "environment", "config");
        require("dedicated_test".equals(text(environment, "kind", "environment")),
                "environment.kind must be dedicated_test");
        require(bool(environment, "dedicated_test_environment_confirmed", "environment"),
                "Dedicated test environment confirmation is required");
        JsonNode policy = object(root, "execution_policy", "config");
        boolean allowMutations = bool(policy, "allow_mutations", "execution_policy");
        boolean probeUnsupported = optionalBoolean(policy, "probe_unsupported", true);
        boolean cleanupOnFailure = optionalBoolean(policy, "cleanup_on_failure", true);
        return new ConnectionConfig(serverId, fingerprint, serverProduct, serverVersion,
                version, transport, encoding, apiLevel,
                endpoint.host(), endpoint.port(), endpoint.httpsUri(), tls, usernamePassword, true, allowMutations,
                probeUnsupported, cleanupOnFailure);
    }

    private static Endpoint parseEndpoint(JsonNode root, String transport) {
        JsonNode endpoints = object(root, "endpoints", "config");
        if ("raw_tls".equals(transport)) {
            JsonNode endpoint = object(endpoints, "raw_tls", "endpoints");
            String host = text(endpoint, "host", "endpoints.raw_tls");
            int port = integer(endpoint, "port", "endpoints.raw_tls");
            require(port >= 1 && port <= 65_535, "endpoints.raw_tls.port is invalid");
            return new Endpoint(host, port, null);
        }

        JsonNode endpoint = object(endpoints, "https", "endpoints");
        URI uri;
        try {
            uri = URI.create(text(endpoint, "url", "endpoints.https"));
        } catch (IllegalArgumentException exception) {
            throw new IllegalArgumentException("endpoints.https.url is invalid");
        }
        require("https".equalsIgnoreCase(uri.getScheme()) && uri.getHost() != null
                && uri.getUserInfo() == null && uri.getFragment() == null,
                "endpoints.https.url must be an HTTPS URI without user info or fragment");
        return new Endpoint("", 0, uri);
    }

    private static Tls parseTls(JsonNode root) {
        JsonNode node = object(root, "tls", "config");
        Tls tls = new Tls(optionalText(node, "ca_cert_path"), optionalText(node, "client_cert_path"),
                optionalText(node, "client_key_path"), optionalText(node, "server_name_override"),
                positiveInteger(node, "connect_timeout_seconds", 10),
                positiveInteger(node, "read_timeout_seconds", 30));
        require(!tls.caCertificatePath().isBlank() && !tls.clientCertificatePath().isBlank()
                && !tls.clientKeyPath().isBlank(), "TLS trust and mutual-authentication files are required");
        return tls;
    }

    private static UsernamePassword parseUsernamePassword(JsonNode root) {
        JsonNode authentication = optionalObject(root, "authentication");
        if (authentication == null) return null;
        JsonNode credentials = optionalObject(authentication, "username_password");
        if (credentials == null) return null;
        String usernameEnv = text(credentials, "username_env", "authentication.username_password");
        String passwordEnv = text(credentials, "password_env", "authentication.username_password");
        require(usernameEnv.matches("[A-Z_][A-Z0-9_]*") && passwordEnv.matches("[A-Z_][A-Z0-9_]*"),
                "Credential environment variable names are invalid");
        return new UsernamePassword(usernameEnv, passwordEnv);
    }

    /** Fails before network activity if a live run violates the dedicated environment or mutation policy. */
    public void requireLiveExecution(boolean containsMutations) {
        requireLiveExecution(containsMutations, false);
    }

    /** Requires both file configuration and the explicit command-line opt-in for mutating cases. */
    public void requireLiveExecution(boolean containsMutations, boolean cliMutationOptIn) {
        require(dedicatedTestEnvironmentConfirmed, "A dedicated test environment must be confirmed");
        if (containsMutations) {
            require(allowMutations && cliMutationOptIn,
                    "Mutating cases require configuration and --enable-mutations opt-in");
        }
    }

    /** Checks configured trust and identity files without resolving hosts or opening sockets. */
    public void requireTlsFilesExist() {
        require(Path.of(tls.caCertificatePath()).toFile().isFile()
                && Path.of(tls.clientCertificatePath()).toFile().isFile()
                && Path.of(tls.clientKeyPath()).toFile().isFile(),
                "Configured TLS files must exist and be regular files");
    }

    public record Tls(String caCertificatePath, String clientCertificatePath, String clientKeyPath,
                      String serverNameOverride, int connectTimeoutSeconds, int readTimeoutSeconds) { }
    public record UsernamePassword(String usernameEnvironmentName, String passwordEnvironmentName) { }
    private record Endpoint(String host, int port, URI httpsUri) { }

    private static JsonNode object(JsonNode parent, String field, String location) {
        JsonNode value = optionalObject(parent, field);
        require(value != null, location + "." + field + " must be an object");
        return value;
    }

    private static JsonNode optionalObject(JsonNode parent, String field) {
        JsonNode value = parent.get(field);
        if (value == null || value.isNull()) return null;
        require(value.isObject(), field + " must be an object");
        return value;
    }

    private static String text(JsonNode parent, String field, String location) {
        JsonNode value = parent.get(field);
        require(value != null && value.isTextual() && !value.textValue().isBlank(),
                location + "." + field + " must be a non-empty string");
        return value.textValue();
    }

    private static String optionalText(JsonNode parent, String field) {
        JsonNode value = parent.get(field);
        if (value == null || value.isNull()) return "";
        require(value.isTextual(), field + " must be a string");
        return value.textValue();
    }

    private static int integer(JsonNode parent, String field, String location) {
        JsonNode value = parent.get(field);
        require(value != null && value.canConvertToInt(), location + "." + field + " must be an integer");
        return value.intValue();
    }

    private static int positiveInteger(JsonNode parent, String field, int defaultValue) {
        JsonNode value = parent.get(field);
        if (value == null || value.isNull()) return defaultValue;
        require(value.canConvertToInt() && value.intValue() > 0, field + " must be a positive integer");
        return value.intValue();
    }

    private static boolean bool(JsonNode parent, String field, String location) {
        JsonNode value = parent.get(field);
        require(value != null && value.isBoolean(), location + "." + field + " must be a boolean");
        return value.booleanValue();
    }

    private static boolean optionalBoolean(JsonNode parent, String field, boolean defaultValue) {
        JsonNode value = parent.get(field);
        if (value == null || value.isNull()) return defaultValue;
        require(value.isBoolean(), field + " must be a boolean");
        return value.booleanValue();
    }

    private static void require(boolean condition, String message) {
        if (!condition) throw new IllegalArgumentException(message);
    }
}
