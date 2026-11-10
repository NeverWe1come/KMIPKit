#include "live_driver.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifndef KMIPKIT_C_VERSION_PROPAGATION_AVAILABLE
#define KMIPKIT_C_VERSION_PROPAGATION_AVAILABLE 0
#endif

#define KMIP_MAX_REQUEST_BYTES UINT64_C(16777216)

static kmipkit_byte_span_v1_t borrowed_span(const char *text) {
    kmipkit_byte_span_v1_t span = {0};
    if (text) {
        span.data = (const uint8_t *)text;
        span.length = (uint64_t)strlen(text);
    }
    return span;
}

static kmipkit_timeout_limit_v1_t bounded_timeout(uint64_t seconds) {
    kmipkit_timeout_limit_v1_t timeout = {
        .kind = KMIPKIT_TRANSPORT_TIMEOUT_BOUNDED,
        .reserved = 0,
        .seconds = seconds,
        .nanoseconds = 0,
        .reserved2 = 0
    };
    return timeout;
}

static bool parse_version(const char *text, int32_t *major, int32_t *minor) {
    if (strcmp(text, "2.0") == 0) {
        *major = 2;
        *minor = 0;
        return true;
    }
    if (strcmp(text, "2.1") == 0) {
        *major = 2;
        *minor = 1;
        return true;
    }
    return false;
}

static bool parse_wire_encoding(const char *text, uint32_t *encoding) {
    if (strcmp(text, "TTLV") == 0) *encoding = KMIPKIT_WIRE_ENCODING_TTLV;
    else if (strcmp(text, "JSON") == 0) *encoding = KMIPKIT_WIRE_ENCODING_JSON;
    else if (strcmp(text, "XML") == 0) *encoding = KMIPKIT_WIRE_ENCODING_XML;
    else return false;
    return true;
}

static void set_error(char *error, size_t capacity, const char *message, int32_t status) {
    if (error && capacity > 0) {
        if (status == 0) snprintf(error, capacity, "%s", message);
        else snprintf(error, capacity, "%s (KMIPKit status %d)", message, status);
    }
}

static bool append_ttlv_value(kmip_live_client_t *client,
                              kmipkit_ttlv_structure_t **structure,
                              uint32_t raw_tag_value,
                              kmipkit_ttlv_value_t **value) {
    kmipkit_raw_tag_t *raw_tag = NULL;
    kmipkit_tag_t *tag = NULL;
    kmipkit_ttlv_item_t *item = NULL;
    kmipkit_ttlv_structure_t *updated = NULL;
    int32_t status = kmipkit_ttlv_raw_tag_create(raw_tag_value, &raw_tag);
    if (status == 0) status = kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag);
    if (status == 0) {
        status = kmipkit_ttlv_item_create(tag, *value, client->limits, &item);
        *value = NULL;
    }
    if (status == 0) {
        status = kmipkit_ttlv_structure_with_item(*structure, item, client->limits, &updated);
        *structure = NULL;
        item = NULL;
        if (status == 0) {
            *structure = updated;
            updated = NULL;
        }
    }
    if (item) kmipkit_ttlv_item_release(item);
    if (updated) kmipkit_ttlv_structure_release(updated);
    if (tag) kmipkit_tag_release(tag);
    if (raw_tag) kmipkit_raw_tag_release(raw_tag);
    return status == 0;
}

static bool append_secret_text(kmip_live_client_t *client,
                               kmipkit_ttlv_structure_t **structure,
                               uint32_t raw_tag,
                               const char *text) {
    kmipkit_secret_text_t *secret = NULL;
    kmipkit_ttlv_value_t *value = NULL;
    int32_t status = kmipkit_secret_text_create((const uint8_t *)text,
                                                (uint64_t)strlen(text), &secret);
    if (status == 0) {
        status = kmipkit_secret_text_into_ttlv_value(secret, &value);
        secret = NULL;
    }
    bool appended = status == 0 && append_ttlv_value(client, structure, raw_tag, &value);
    if (secret) kmipkit_secret_text_release(secret);
    if (value) kmipkit_ttlv_value_release(value);
    return appended;
}

static bool structure_into_value(kmip_live_client_t *client,
                                 kmipkit_ttlv_structure_t **structure,
                                 kmipkit_ttlv_value_t **value) {
    int32_t status = kmipkit_ttlv_value_structure(*structure, client->limits, value);
    *structure = NULL;
    return status == 0;
}

static bool create_symmetric_key_attributes(kmip_live_client_t *client,
                                            kmipkit_ttlv_structure_t **out_attributes) {
    int32_t status = kmipkit_ttlv_structure_create(out_attributes);
    kmipkit_ttlv_value_t *value = NULL;
    bool valid = status == 0 && kmipkit_ttlv_value_enumeration(
        KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_AES, &value) == 0 &&
        append_ttlv_value(client, out_attributes, UINT32_C(0x420028), &value) &&
        kmipkit_ttlv_value_integer(128, &value) == 0 &&
        append_ttlv_value(client, out_attributes, UINT32_C(0x42002A), &value);
    if (value) kmipkit_ttlv_value_release(value);
    if (!valid && *out_attributes) {
        kmipkit_ttlv_structure_release(*out_attributes);
        *out_attributes = NULL;
    }
    return valid;
}

static bool item_view_has_tag(kmipkit_ttlv_item_view_t *item,
                              uint32_t expected_tag,
                              uint8_t expected_type,
                              kmipkit_ttlv_value_view_t **out_value) {
    kmipkit_tag_t *tag = NULL;
    uint32_t raw_tag = 0;
    uint8_t item_type = 0;
    bool valid = kmipkit_ttlv_item_view_tag(item, &tag) == 0 &&
        kmipkit_ttlv_tag_value(tag, &raw_tag) == 0 &&
        kmipkit_ttlv_item_view_type(item, &item_type) == 0 &&
        raw_tag == expected_tag && item_type == expected_type;
    if (tag) kmipkit_tag_release(tag);
    if (valid && out_value) valid = kmipkit_ttlv_item_view_value(item, out_value) == 0;
    return valid;
}

static bool attributes_match_created_symmetric_key(kmipkit_ttlv_structure_t *attributes) {
    kmipkit_ttlv_structure_view_t *view = NULL;
    uint64_t count = 0;
    bool valid = attributes && kmipkit_ttlv_structure_view(attributes, &view) == 0 &&
        kmipkit_ttlv_structure_view_item_count(view, &count) == 0;
    bool algorithm_found = false;
    bool length_found = false;
    for (uint64_t index = 0; valid && index < count; ++index) {
        kmipkit_ttlv_item_view_t *item = NULL;
        if (kmipkit_ttlv_structure_view_item_at(view, index, &item) != 0) {
            valid = false;
            break;
        }
        kmipkit_ttlv_value_view_t *value = NULL;
        if (item_view_has_tag(item, UINT32_C(0x420028),
                              KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION, &value)) {
            uint32_t actual = 0;
            algorithm_found = kmipkit_ttlv_value_view_enumeration(value, &actual) == 0 &&
                actual == KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_AES;
        } else if (item_view_has_tag(item, UINT32_C(0x42002A),
                                     KMIPKIT_TTLV_ITEM_TYPE_INTEGER, &value)) {
            int32_t actual = 0;
            length_found = kmipkit_ttlv_value_view_integer(value, &actual) == 0 && actual == 128;
        }
        if (value) kmipkit_ttlv_value_view_release(value);
        kmipkit_ttlv_item_view_release(item);
    }
    if (view) kmipkit_ttlv_structure_view_release(view);
    return valid && algorithm_found && length_found;
}

static bool same_text_identifier(const kmipkit_unique_identifier_v1_t *left,
                                 const kmipkit_unique_identifier_v1_t *right) {
    return left->kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING &&
        right->kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING &&
        left->text_length == right->text_length &&
        left->text_data != NULL && right->text_data != NULL &&
        memcmp(left->text_data, right->text_data, (size_t)left->text_length) == 0;
}

static int32_t set_batch_item_id(kmipkit_client_batch_item_t **item, const char *id) {
    kmipkit_client_batch_item_t *identified_item = NULL;
    int32_t status = kmipkit_client_batch_item_with_unique_batch_item_id(
        *item, (const uint8_t *)id, (uint64_t)strlen(id), &identified_item);
    if (status == 0) *item = identified_item;
    return status;
}

static bool request_item_is_valid(kmipkit_client_batch_item_t *item,
                                  char *diagnostic,
                                  size_t diagnostic_capacity) {
    kmipkit_validation_report_t *report = NULL;
    uint8_t is_valid = 0;
    int32_t status = kmipkit_client_batch_item_validation_report(item, &report);
    if (status == 0) status = kmipkit_validation_report_is_valid(report, &is_valid);
    if (status != 0) {
        snprintf(diagnostic, diagnostic_capacity,
                 "KMIPKit request validation failed with status %d", status);
        if (report) kmipkit_validation_report_release(report);
        return false;
    }
    if (is_valid) {
        kmipkit_validation_report_release(report);
        return true;
    }
    uint64_t issue_count = 0;
    if (kmipkit_validation_report_issue_count(report, &issue_count) == 0 && issue_count > 0) {
        kmipkit_validation_issue_view_v1_t issue = {
            .struct_size = (uint32_t)sizeof(kmipkit_validation_issue_view_v1_t),
            .abi_version = 1
        };
        if (kmipkit_validation_report_issue_at_v1(report, 0, &issue) == 0 &&
            issue.code.data && issue.code.length <= 120 &&
            issue.path.data && issue.path.length <= 160) {
            snprintf(diagnostic, diagnostic_capacity,
                     "KMIPKit rejected request schema code=%.*s path=%.*s",
                     (int)issue.code.length, (const char *)issue.code.data,
                     (int)issue.path.length, (const char *)issue.path.data);
        } else {
            snprintf(diagnostic, diagnostic_capacity,
                     "KMIPKit rejected request schema with %llu finding(s)",
                     (unsigned long long)issue_count);
        }
    } else {
        snprintf(diagnostic, diagnostic_capacity,
                 "KMIPKit rejected request schema without a readable finding");
    }
    kmipkit_validation_report_release(report);
    return false;
}

static bool create_authentication(kmip_live_client_t *client,
                                  kmipkit_authentication_t **out_authentication) {
    const char *username = getenv(client->username_env);
    const char *password = getenv(client->password_env);
    if (!username || username[0] == '\0' || !password) return false;

    kmipkit_ttlv_structure_t *credential_value = NULL;
    kmipkit_ttlv_structure_t *credential = NULL;
    kmipkit_ttlv_structure_t *authentication = NULL;
    kmipkit_ttlv_value_t *value = NULL;
    int32_t status = kmipkit_ttlv_structure_create(&credential_value);
    bool valid = status == 0 &&
        append_secret_text(client, &credential_value, UINT32_C(0x420099), username) &&
        append_secret_text(client, &credential_value, UINT32_C(0x4200A1), password) &&
        kmipkit_ttlv_structure_create(&credential) == 0 &&
        kmipkit_ttlv_value_enumeration(
            KMIPKIT_ENUMERATION_CREDENTIAL_TYPE_USERNAME_AND_PASSWORD, &value) == 0 &&
        append_ttlv_value(client, &credential, UINT32_C(0x420024), &value) &&
        structure_into_value(client, &credential_value, &value) &&
        append_ttlv_value(client, &credential, UINT32_C(0x420025), &value);
    if (valid) {
        valid = kmipkit_ttlv_structure_create(&authentication) == 0 &&
            structure_into_value(client, &credential, &value) &&
            append_ttlv_value(client, &authentication, UINT32_C(0x420023), &value);
    }
    if (valid) {
        status = kmipkit_authentication_from_ttlv(authentication, out_authentication);
        authentication = NULL;
        valid = status == 0;
    }
    if (value) kmipkit_ttlv_value_release(value);
    if (credential_value) kmipkit_ttlv_structure_release(credential_value);
    if (credential) kmipkit_ttlv_structure_release(credential);
    if (authentication) kmipkit_ttlv_structure_release(authentication);
    return valid;
}

static bool has_tls_material(const kmip_connection_config_t *configuration) {
    return configuration->ca_cert_path[0] != '\0' &&
           configuration->client_cert_path[0] != '\0' &&
           configuration->client_key_path[0] != '\0';
}

static bool create_client(const kmip_connection_config_t *configuration,
                          kmip_live_client_t *client, char *error,
                          size_t error_capacity) {
    kmipkit_extension_registry_limits_v1_t registry_limits = {
        .struct_size = (uint32_t)sizeof(kmipkit_extension_registry_limits_v1_t),
        .abi_version = 1
    };
    kmipkit_client_extension_registry_t *registry = NULL;
    kmipkit_client_configuration_t *default_configuration = NULL;
    kmipkit_client_configuration_t *version_configuration = NULL;
    kmipkit_client_configuration_t *encoding_configuration = NULL;
    kmipkit_transport_config_t *transport_configuration = NULL;
    kmipkit_transport_input_v1_t certificate = {0};
    kmipkit_transport_input_v1_t trust_certificate = {0};
    kmipkit_transport_config_v1_t transport = {0};
    int32_t status = kmipkit_extension_registry_limits_v1_default_values(&registry_limits);
    if (status != 0) {
        set_error(error, error_capacity, "could not initialize extension registry limits", status);
        goto fail;
    }
    status = kmipkit_client_extension_registry_create_v1(NULL, 0, &registry_limits, &registry);
    if (status != 0) {
        set_error(error, error_capacity, "could not create the standard-only extension registry", status);
        goto fail;
    }
    status = kmipkit_client_configuration_create(registry, &default_configuration);
    registry = NULL;
    if (status != 0) {
        set_error(error, error_capacity, "could not create client configuration", status);
        goto fail;
    }
    status = kmipkit_client_configuration_with_protocol_version(
        default_configuration, client->protocol_major, client->protocol_minor,
        &version_configuration);
    if (status != 0) {
        set_error(error, error_capacity, "could not set the requested KMIP version", status);
        goto fail;
    }
    status = kmipkit_client_configuration_with_wire_encoding(
        version_configuration, client->wire_encoding, &encoding_configuration);
    if (status != 0) {
        set_error(error, error_capacity, "could not set the requested wire encoding", status);
        goto fail;
    }

    certificate.source_kind = KMIPKIT_TRANSPORT_SOURCE_FILE;
    certificate.encoding = KMIPKIT_TRANSPORT_ENCODING_PEM;
    certificate.data = borrowed_span(configuration->client_cert_path);
    trust_certificate.source_kind = KMIPKIT_TRANSPORT_SOURCE_FILE;
    trust_certificate.encoding = KMIPKIT_TRANSPORT_ENCODING_PEM;
    trust_certificate.data = borrowed_span(configuration->ca_cert_path);
    transport.struct_size = (uint32_t)sizeof(transport);
    transport.abi_version = 1;
    transport.trust_kind = KMIPKIT_TRANSPORT_TRUST_CERTIFICATES;
    transport.client_certificates = &certificate;
    transport.client_certificate_count = 1;
    transport.client_private_key.source_kind = KMIPKIT_TRANSPORT_SOURCE_FILE;
    transport.client_private_key.encoding = KMIPKIT_TRANSPORT_ENCODING_PEM;
    transport.client_private_key.data = borrowed_span(configuration->client_key_path);
    transport.trust_certificates = &trust_certificate;
    transport.trust_certificate_count = 1;
    transport.connect_timeout = bounded_timeout(10);
    transport.write_timeout = bounded_timeout(30);
    transport.read_timeout = bounded_timeout(30);
    transport.total_timeout = bounded_timeout(60);
    transport.max_request_bytes = KMIP_MAX_REQUEST_BYTES;
    if (configuration->server_name_override[0] != '\0') {
        transport.flags |= KMIPKIT_TRANSPORT_CONFIG_FLAG_TLS_SERVER_NAME;
        transport.tls_server_name = borrowed_span(configuration->server_name_override);
    }
    if (strcmp(configuration->transport, "raw_tls") == 0) {
        transport.endpoint_kind = KMIPKIT_TRANSPORT_ENDPOINT_RAW_TLS;
        transport.endpoint = borrowed_span(configuration->host);
        transport.endpoint_port = configuration->port;
    } else if (strcmp(configuration->transport, "https") == 0) {
        transport.endpoint_kind = KMIPKIT_TRANSPORT_ENDPOINT_HTTPS;
        transport.endpoint = borrowed_span(configuration->https_url);
        transport.endpoint_port = 0;
    } else {
        set_error(error, error_capacity, "unsupported configured transport", 0);
        goto fail;
    }
    status = kmipkit_transport_config_create_v1(&transport, &transport_configuration);
    if (status != 0) {
        set_error(error, error_capacity, "could not construct the configured mutual-TLS transport", status);
        goto fail;
    }
    status = kmipkit_client_create(encoding_configuration, &transport_configuration, &client->client);
    if (status != 0) {
        set_error(error, error_capacity, "could not construct the KMIPKit client", status);
        goto fail;
    }
    kmipkit_client_configuration_release(encoding_configuration);
    kmipkit_client_configuration_release(version_configuration);
    kmipkit_client_configuration_release(default_configuration);
    return true;

fail:
    if (client->limits) kmipkit_codec_limits_release(client->limits);
    if (client->client) kmipkit_client_release(client->client);
    if (transport_configuration) kmipkit_transport_config_release(transport_configuration);
    if (encoding_configuration) kmipkit_client_configuration_release(encoding_configuration);
    if (version_configuration) kmipkit_client_configuration_release(version_configuration);
    if (default_configuration) kmipkit_client_configuration_release(default_configuration);
    if (registry) kmipkit_client_extension_registry_release(registry);
    return false;
}

bool kmip_live_client_open(const kmip_connection_config_t *configuration,
                           kmip_live_client_t *out_client, char *error,
                           size_t error_capacity) {
    if (!configuration || !out_client) {
        set_error(error, error_capacity, "missing live client configuration", 0);
        return false;
    }
    memset(out_client, 0, sizeof(*out_client));
    if (!parse_version(configuration->protocol_version, &out_client->protocol_major,
                       &out_client->protocol_minor) ||
        !parse_wire_encoding(configuration->encoding, &out_client->wire_encoding)) {
        set_error(error, error_capacity, "unsupported KMIP version or encoding", 0);
        return false;
    }
    if (strcmp(configuration->api_level, "typed-protocol") == 0 &&
        !has_tls_material(configuration)) {
        set_error(error, error_capacity, "CA certificate, client certificate and client key are required", 0);
        return false;
    }
    if (strcmp(configuration->api_level, "typed-protocol") == 0 &&
        (configuration->username_env[0] != '\0' || configuration->password_env[0] != '\0')) {
        if (configuration->username_env[0] == '\0' || configuration->password_env[0] == '\0' ||
            !getenv(configuration->username_env) || getenv(configuration->username_env)[0] == '\0' ||
            !getenv(configuration->password_env)) {
            set_error(error, error_capacity,
                      "configured Username/Password environment variables are unavailable", 0);
            return false;
        }
        snprintf(out_client->username_env, sizeof(out_client->username_env), "%s",
                 configuration->username_env);
        snprintf(out_client->password_env, sizeof(out_client->password_env), "%s",
                 configuration->password_env);
    }
    snprintf(out_client->api_level, sizeof(out_client->api_level), "%s", configuration->api_level);
    out_client->configured_version_propagation_available =
        KMIPKIT_C_VERSION_PROPAGATION_AVAILABLE != 0;
    if (strcmp(out_client->api_level, "typed-protocol") == 0 &&
        out_client->protocol_minor == 0 &&
        !out_client->configured_version_propagation_available) {
        set_error(error, error_capacity,
                  "pinned KMIPKit C artifact does not preserve the configured KMIP 2.0 version", 0);
        return false;
    }
    int32_t limits_status = kmipkit_codec_limits_defaults(&out_client->limits);
    if (limits_status != 0) {
        set_error(error, error_capacity, "could not create codec limits", limits_status);
        kmip_live_client_close(out_client);
        return false;
    }
    if (strcmp(out_client->api_level, "typed-protocol") == 0 &&
        !create_client(configuration, out_client, error, error_capacity)) {
        kmip_live_client_close(out_client);
        return false;
    }
    return true;
}

void kmip_live_client_close(kmip_live_client_t *client) {
    if (!client) return;
    if (client->limits) kmipkit_codec_limits_release(client->limits);
    if (client->client) kmipkit_client_release(client->client);
    memset(client, 0, sizeof(*client));
}

static kmip_execution_outcome_t outcome_not_run(const char *reason) {
    kmip_execution_outcome_t outcome = {0};
    if (reason) snprintf(outcome.result_reason, sizeof(outcome.result_reason), "%s", reason);
    return outcome;
}

typedef const char *(*kmip_enumeration_name_fn)(uint32_t);
typedef uint32_t (*kmip_bitmask_split_fn)(uint32_t);

typedef struct {
    const char *name;
    uint32_t tag;
    kmip_enumeration_name_fn name_for_value;
} kmip_enum_probe_t;

static const kmip_enum_probe_t KMIP_ENUM_PROBES[] = {
    {"Adjustment Type", UINT32_C(0x420158), KMIPKIT_ENUMERATION_ADJUSTMENT_TYPE_name},
    {"Alternative Name Type", UINT32_C(0x4200C1), KMIPKIT_ENUMERATION_ALTERNATIVE_NAME_TYPE_name},
    {"Asynchronous Indicator", UINT32_C(0x420007), KMIPKIT_ENUMERATION_ASYNCHRONOUS_INDICATOR_name},
    {"Attestation Type", UINT32_C(0x4200C7), KMIPKIT_ENUMERATION_ATTESTATION_TYPE_name},
    {"Batch Error Continuation Option", UINT32_C(0x42000E), KMIPKIT_ENUMERATION_BATCH_ERROR_CONTINUATION_OPTION_name},
    {"Block Cipher Mode", UINT32_C(0x420011), KMIPKIT_ENUMERATION_BLOCK_CIPHER_MODE_name},
    {"Cancellation Result", UINT32_C(0x420012), KMIPKIT_ENUMERATION_CANCELLATION_RESULT_name},
    {"Certificate Request Type", UINT32_C(0x420019), KMIPKIT_ENUMERATION_CERTIFICATE_REQUEST_TYPE_name},
    {"Certificate Type", UINT32_C(0x42001D), KMIPKIT_ENUMERATION_CERTIFICATE_TYPE_name},
    {"Client Registration Method", UINT32_C(0x4200F6), KMIPKIT_ENUMERATION_CLIENT_REGISTRATION_METHOD_name},
    {"Credential Type", UINT32_C(0x420024), KMIPKIT_ENUMERATION_CREDENTIAL_TYPE_name},
    {"Cryptographic Algorithm", UINT32_C(0x420028), KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_name},
    {"DRBG Algorithm", UINT32_C(0x4200DB), KMIPKIT_ENUMERATION_DRBG_ALGORITHM_name},
    {"Data", UINT32_C(0x4200C2), KMIPKIT_ENUMERATION_DATA_name},
    {"Derivation Method", UINT32_C(0x420031), KMIPKIT_ENUMERATION_DERIVATION_METHOD_name},
    {"Destroy Action", UINT32_C(0x4200F3), KMIPKIT_ENUMERATION_DESTROY_ACTION_name},
    {"Digital Signature Algorithm", UINT32_C(0x4200AE), KMIPKIT_ENUMERATION_DIGITAL_SIGNATURE_ALGORITHM_name},
    {"Encoding Option", UINT32_C(0x4200A3), KMIPKIT_ENUMERATION_ENCODING_OPTION_name},
    {"Endpoint Role", UINT32_C(0x420151), KMIPKIT_ENUMERATION_ENDPOINT_ROLE_name},
    {"FIPS186 Variation", UINT32_C(0x4200DC), KMIPKIT_ENUMERATION_FIPS186_VARIATION_name},
    {"Hashing Algorithm", UINT32_C(0x420038), KMIPKIT_ENUMERATION_HASHING_ALGORITHM_name},
    {"Interop Function", UINT32_C(0x420160), KMIPKIT_ENUMERATION_INTEROP_FUNCTION_name},
    {"Item Type", 0, NULL},
    {"Key Compression Type", UINT32_C(0x420041), KMIPKIT_ENUMERATION_KEY_COMPRESSION_TYPE_name},
    {"Key Format Type", UINT32_C(0x420042), KMIPKIT_ENUMERATION_KEY_FORMAT_TYPE_name},
    {"Key Role Type", UINT32_C(0x420083), KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_name},
    {"Key Value Location Type", UINT32_C(0x4200BA), KMIPKIT_ENUMERATION_KEY_VALUE_LOCATION_TYPE_name},
    {"Key Wrap Type", UINT32_C(0x4200F8), KMIPKIT_ENUMERATION_KEY_WRAP_TYPE_name},
    {"Link Type", UINT32_C(0x42004B), KMIPKIT_ENUMERATION_LINK_TYPE_name},
    {"Mask Generator", UINT32_C(0x420101), KMIPKIT_ENUMERATION_MASK_GENERATOR_name},
    {"NIST Key Type", UINT32_C(0x42013A), KMIPKIT_ENUMERATION_NIST_KEY_TYPE_name},
    {"Name Type", UINT32_C(0x420054), KMIPKIT_ENUMERATION_NAME_TYPE_name},
    {"Object Group Member", UINT32_C(0x4200AC), KMIPKIT_ENUMERATION_OBJECT_GROUP_MEMBER_name},
    {"Object Type", UINT32_C(0x420057), KMIPKIT_ENUMERATION_OBJECT_TYPE_name},
    {"Opaque Data Type", UINT32_C(0x420059), KMIPKIT_ENUMERATION_OPAQUE_DATA_TYPE_name},
    {"Operation", UINT32_C(0x42005C), KMIPKIT_ENUMERATION_OPERATION_name},
    {"PKCS#11 Function", UINT32_C(0x42015A), KMIPKIT_ENUMERATION_PKCS_11_FUNCTION_name},
    {"PKCS#11 Return Code", UINT32_C(0x42015D), KMIPKIT_ENUMERATION_PKCS_11_RETURN_CODE_name},
    {"Padding Method", UINT32_C(0x42005F), KMIPKIT_ENUMERATION_PADDING_METHOD_name},
    {"Processing Stage", UINT32_C(0x420175), KMIPKIT_ENUMERATION_PROCESSING_STAGE_name},
    {"Profile Name", UINT32_C(0x4200EC), KMIPKIT_ENUMERATION_PROFILE_NAME_name},
    {"Protection Level", UINT32_C(0x420145), KMIPKIT_ENUMERATION_PROTECTION_LEVEL_name},
    {"Put Function", UINT32_C(0x420070), KMIPKIT_ENUMERATION_PUT_FUNCTION_name},
    {"Query Function", UINT32_C(0x420074), KMIPKIT_ENUMERATION_QUERY_FUNCTION_name},
    {"RNG Algorithm", UINT32_C(0x4200DA), KMIPKIT_ENUMERATION_RNG_ALGORITHM_name},
    {"RNG Mode", UINT32_C(0x4200F5), KMIPKIT_ENUMERATION_RNG_MODE_name},
    {"Recommended Curve", UINT32_C(0x420075), KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_name},
    {"Result Reason", UINT32_C(0x42007E), KMIPKIT_ENUMERATION_RESULT_REASON_name},
    {"Result Status", UINT32_C(0x42007F), KMIPKIT_ENUMERATION_RESULT_STATUS_name},
    {"Revocation Reason Code", UINT32_C(0x420082), KMIPKIT_ENUMERATION_REVOCATION_REASON_CODE_name},
    {"Rotate Name Type", UINT32_C(0x420171), KMIPKIT_ENUMERATION_ROTATE_NAME_TYPE_name},
    {"Secret Data Type", UINT32_C(0x420086), KMIPKIT_ENUMERATION_SECRET_DATA_TYPE_name},
    {"Shredding Algorithm", UINT32_C(0x4200F4), KMIPKIT_ENUMERATION_SHREDDING_ALGORITHM_name},
    {"Split Key Method", UINT32_C(0x42008A), KMIPKIT_ENUMERATION_SPLIT_KEY_METHOD_name},
    {"State", UINT32_C(0x42008D), KMIPKIT_ENUMERATION_STATE_name},
    {"Tag", UINT32_C(0x420138), KMIPKIT_ENUMERATION_TAG_name},
    {"Ticket Type", UINT32_C(0x42014A), KMIPKIT_ENUMERATION_TICKET_TYPE_name},
    {"Unique Identifier", UINT32_C(0x420094), KMIPKIT_ENUMERATION_UNIQUE_IDENTIFIER_name},
    {"Unwrap Mode", UINT32_C(0x4200F2), KMIPKIT_ENUMERATION_UNWRAP_MODE_name},
    {"Usage Limits Unit", UINT32_C(0x420098), KMIPKIT_ENUMERATION_USAGE_LIMITS_UNIT_name},
    {"Validation Authority Type", UINT32_C(0x4200E0), KMIPKIT_ENUMERATION_VALIDATION_AUTHORITY_TYPE_name},
    {"Validation Type", UINT32_C(0x4200E5), KMIPKIT_ENUMERATION_VALIDATION_TYPE_name},
    {"Validity Indicator", UINT32_C(0x42009B), KMIPKIT_ENUMERATION_VALIDITY_INDICATOR_name},
    {"Wrapping Method", UINT32_C(0x42009E), KMIPKIT_ENUMERATION_WRAPPING_METHOD_name}
};

typedef struct {
    const char *name;
    uint32_t raw_value;
} kmip_enum_fallback_probe_t;

/* These standard value forms have no assigned name in the installed C helper. */
static const kmip_enum_fallback_probe_t KMIP_ENUM_FALLBACK_PROBES[] = {
    {"Opaque Data Type", UINT32_C(0x80000000)}, /* OASIS KMIP 2.1 §11.35 */
    {"PKCS#11 Function", UINT32_C(1)}, /* OASIS KMIP 2.1 §11.38: one-based offset */
    {"PKCS#11 Return Code", UINT32_C(0)}, /* OASIS KMIP 2.1 §11.39: CK_RV value */
    {"Tag", UINT32_C(0x420001)} /* OASIS KMIP 2.1 §11.56: assigned KMIP tag */
};

typedef struct {
    const char *name;
    uint32_t tag;
    uint32_t known_mask;
    kmip_bitmask_split_fn known_bits;
    kmip_bitmask_split_fn unknown_bits;
} kmip_bitmask_probe_t;

static const kmip_bitmask_probe_t KMIP_BITMASK_PROBES[] = {
    {"Cryptographic Usage Mask", UINT32_C(0x42002C), KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_KNOWN_MASK,
     KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_known_bits, KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_unknown_bits},
    {"Protection Storage Mask", UINT32_C(0x42015E), KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_KNOWN_MASK,
     KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_known_bits, KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_unknown_bits},
    {"Storage Status Mask", UINT32_C(0x42008E), KMIPKIT_BITMASK_STORAGE_STATUS_MASK_KNOWN_MASK,
     KMIPKIT_BITMASK_STORAGE_STATUS_MASK_known_bits, KMIPKIT_BITMASK_STORAGE_STATUS_MASK_unknown_bits}
};

static const kmip_enum_probe_t *find_enum_probe(const char *name) {
    for (size_t index = 0; index < sizeof(KMIP_ENUM_PROBES) / sizeof(KMIP_ENUM_PROBES[0]); ++index) {
        if (strcmp(name, KMIP_ENUM_PROBES[index].name) == 0) return &KMIP_ENUM_PROBES[index];
    }
    return NULL;
}

static const kmip_enum_fallback_probe_t *find_enum_fallback_probe(const char *name) {
    for (size_t index = 0; index < sizeof(KMIP_ENUM_FALLBACK_PROBES) /
                            sizeof(KMIP_ENUM_FALLBACK_PROBES[0]); ++index) {
        if (strcmp(name, KMIP_ENUM_FALLBACK_PROBES[index].name) == 0)
            return &KMIP_ENUM_FALLBACK_PROBES[index];
    }
    return NULL;
}

static const kmip_bitmask_probe_t *find_bitmask_probe(const char *name) {
    for (size_t index = 0; index < sizeof(KMIP_BITMASK_PROBES) / sizeof(KMIP_BITMASK_PROBES[0]); ++index) {
        if (strcmp(name, KMIP_BITMASK_PROBES[index].name) == 0) return &KMIP_BITMASK_PROBES[index];
    }
    return NULL;
}

static bool roundtrip_scalar_for_inspection(bool enumeration, uint32_t tag_code,
                                            int64_t raw_value) {
    kmipkit_raw_tag_t *raw_tag = NULL;
    kmipkit_tag_t *tag = NULL;
    kmipkit_ttlv_value_t *value = NULL;
    kmipkit_ttlv_item_t *item = NULL;
    kmipkit_ttlv_structure_t *structure = NULL;
    kmipkit_ttlv_structure_t *updated_structure = NULL;
    kmipkit_ttlv_structure_view_t *structure_view = NULL;
    kmipkit_ttlv_item_view_t *item_view = NULL;
    kmipkit_ttlv_value_view_t *value_view = NULL;
    kmipkit_codec_limits_t *limits = NULL;
    uint64_t item_count = 0;
    uint32_t actual_enumeration = 0;
    int32_t actual_integer = 0;
    uint8_t item_type = 0;
    bool valid = false;
    int32_t status = kmipkit_codec_limits_defaults(&limits);
    if (status == 0) status = kmipkit_ttlv_raw_tag_create(tag_code, &raw_tag);
    if (status == 0) status = kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag);
    if (status == 0) {
        status = enumeration
            ? kmipkit_ttlv_value_enumeration((uint32_t)raw_value, &value)
            : kmipkit_ttlv_value_integer((int32_t)raw_value, &value);
    }
    if (status == 0) status = kmipkit_ttlv_item_create(tag, value, limits, &item);
    if (status == 0) {
        value = NULL;
        status = kmipkit_ttlv_structure_create(&structure);
    }
    if (status == 0)
        status = kmipkit_ttlv_structure_with_item(structure, item, limits, &updated_structure);
    if (status == 0) {
        structure = updated_structure;
        updated_structure = NULL;
        item = NULL;
        status = kmipkit_ttlv_structure_view(structure, &structure_view);
    }
    if (status == 0) status = kmipkit_ttlv_structure_view_item_count(structure_view, &item_count);
    if (status == 0 && item_count == 1)
        status = kmipkit_ttlv_structure_view_item_at(structure_view, 0, &item_view);
    if (status == 0) status = kmipkit_ttlv_item_view_type(item_view, &item_type);
    if (status == 0) status = kmipkit_ttlv_item_view_value(item_view, &value_view);
    if (status == 0 && enumeration) {
        status = kmipkit_ttlv_value_view_enumeration(value_view, &actual_enumeration);
        valid = status == 0 && item_type == KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION &&
                actual_enumeration == (uint32_t)raw_value;
    } else if (status == 0) {
        status = kmipkit_ttlv_value_view_integer(value_view, &actual_integer);
        valid = status == 0 && item_type == KMIPKIT_TTLV_ITEM_TYPE_INTEGER &&
                (int64_t)actual_integer == raw_value;
    }
    if (value_view) kmipkit_ttlv_value_view_release(value_view);
    if (item_view) kmipkit_ttlv_item_view_release(item_view);
    if (structure_view) kmipkit_ttlv_structure_view_release(structure_view);
    if (updated_structure) kmipkit_ttlv_structure_release(updated_structure);
    if (structure) kmipkit_ttlv_structure_release(structure);
    if (item) kmipkit_ttlv_item_release(item);
    if (value) kmipkit_ttlv_value_release(value);
    if (tag) kmipkit_tag_release(tag);
    if (raw_tag) kmipkit_raw_tag_release(raw_tag);
    if (limits) kmipkit_codec_limits_release(limits);
    return valid;
}

static bool roundtrip_ttlv_item_type(uint32_t expected_type) {
    static const uint8_t bigint_bytes[] = {UINT8_C(1)};
    static const uint8_t byte_string[] = {UINT8_C(0xA5)};
    static const uint8_t text_string[] = {'i', 't', 'e', 'm'};
    kmipkit_raw_tag_t *raw_tag = NULL;
    kmipkit_tag_t *tag = NULL;
    kmipkit_ttlv_value_t *value = NULL;
    kmipkit_ttlv_item_t *item = NULL;
    kmipkit_ttlv_structure_t *nested_structure = NULL;
    kmipkit_ttlv_structure_t *structure = NULL;
    kmipkit_ttlv_structure_t *updated_structure = NULL;
    kmipkit_ttlv_structure_view_t *structure_view = NULL;
    kmipkit_ttlv_item_view_t *item_view = NULL;
    kmipkit_codec_limits_t *limits = NULL;
    uint64_t item_count = 0;
    uint8_t actual_type = 0;
    int32_t status = kmipkit_codec_limits_defaults(&limits);
    if (status == 0) status = kmipkit_ttlv_raw_tag_create(UINT32_C(0x420057), &raw_tag);
    if (status == 0) status = kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag);

    if (status == 0) {
        switch (expected_type) {
            case KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE:
                status = kmipkit_ttlv_structure_create(&nested_structure);
                if (status == 0)
                    status = kmipkit_ttlv_value_structure(nested_structure, limits, &value);
                break;
            case KMIPKIT_TTLV_ITEM_TYPE_INTEGER:
                status = kmipkit_ttlv_value_integer(-7, &value);
                break;
            case KMIPKIT_TTLV_ITEM_TYPE_LONG_INTEGER:
                status = kmipkit_ttlv_value_long_integer(INT64_C(-7000000001), &value);
                break;
            case KMIPKIT_TTLV_ITEM_TYPE_BIG_INTEGER:
                status = kmipkit_ttlv_value_big_integer(
                    limits, bigint_bytes, sizeof(bigint_bytes), &value);
                break;
            case KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION:
                status = kmipkit_ttlv_value_enumeration(UINT32_C(0xF1234567), &value);
                break;
            case KMIPKIT_TTLV_ITEM_TYPE_BOOLEAN:
                status = kmipkit_ttlv_value_boolean(1, &value);
                break;
            case KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING:
                status = kmipkit_ttlv_value_text_string(
                    limits, text_string, sizeof(text_string), &value);
                break;
            case KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING:
                status = kmipkit_ttlv_value_byte_string(
                    limits, byte_string, sizeof(byte_string), &value);
                break;
            case KMIPKIT_TTLV_ITEM_TYPE_DATE_TIME:
                status = kmipkit_ttlv_value_date_time(INT64_C(1700000000), &value);
                break;
            case KMIPKIT_TTLV_ITEM_TYPE_INTERVAL:
                status = kmipkit_ttlv_value_interval(UINT32_C(60), &value);
                break;
            case KMIPKIT_TTLV_ITEM_TYPE_DATE_TIME_EXTENDED:
                status = kmipkit_ttlv_value_date_time_extended(INT64_C(1700000000), &value);
                break;
            default:
                status = -1;
                break;
        }
    }
    if (status == 0) status = kmipkit_ttlv_item_create(tag, value, limits, &item);
    if (status == 0) {
        value = NULL;
        status = kmipkit_ttlv_structure_create(&structure);
    }
    if (status == 0)
        status = kmipkit_ttlv_structure_with_item(structure, item, limits, &updated_structure);
    if (status == 0) {
        structure = updated_structure;
        updated_structure = NULL;
        item = NULL;
        status = kmipkit_ttlv_structure_view(structure, &structure_view);
    }
    if (status == 0) status = kmipkit_ttlv_structure_view_item_count(structure_view, &item_count);
    if (status == 0 && item_count == 1)
        status = kmipkit_ttlv_structure_view_item_at(structure_view, 0, &item_view);
    if (status == 0) status = kmipkit_ttlv_item_view_type(item_view, &actual_type);

    bool valid = status == 0 && item_count == 1 && actual_type == expected_type;
    if (item_view) kmipkit_ttlv_item_view_release(item_view);
    if (structure_view) kmipkit_ttlv_structure_view_release(structure_view);
    if (updated_structure) kmipkit_ttlv_structure_release(updated_structure);
    if (structure) kmipkit_ttlv_structure_release(structure);
    if (item) kmipkit_ttlv_item_release(item);
    if (nested_structure) kmipkit_ttlv_structure_release(nested_structure);
    if (value) kmipkit_ttlv_value_release(value);
    if (tag) kmipkit_tag_release(tag);
    if (raw_tag) kmipkit_raw_tag_release(raw_tag);
    if (limits) kmipkit_codec_limits_release(limits);
    return valid;
}

static bool roundtrip_all_ttlv_item_types(void) {
    static const uint32_t item_types[] = {
        KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE,
        KMIPKIT_TTLV_ITEM_TYPE_INTEGER,
        KMIPKIT_TTLV_ITEM_TYPE_LONG_INTEGER,
        KMIPKIT_TTLV_ITEM_TYPE_BIG_INTEGER,
        KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION,
        KMIPKIT_TTLV_ITEM_TYPE_BOOLEAN,
        KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING,
        KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING,
        KMIPKIT_TTLV_ITEM_TYPE_DATE_TIME,
        KMIPKIT_TTLV_ITEM_TYPE_INTERVAL,
        KMIPKIT_TTLV_ITEM_TYPE_DATE_TIME_EXTENDED
    };
    for (size_t index = 0; index < sizeof(item_types) / sizeof(item_types[0]); ++index) {
        if (!KMIPKIT_ENUMERATION_ITEM_TYPE_name(item_types[index]) ||
            !roundtrip_ttlv_item_type(item_types[index])) return false;
    }
    return true;
}

static kmip_execution_outcome_t local_model_failure(const char *message,
                                                    const char *type_name) {
    kmip_execution_outcome_t outcome = { .executed = true, .library_failed = true };
    snprintf(outcome.result_reason, sizeof(outcome.result_reason), "%s: %s",
             message, type_name);
    return outcome;
}

static kmip_execution_outcome_t local_model_success(void) {
    return (kmip_execution_outcome_t){
        .executed = true, .success = true, .library_validated = true
    };
}

static kmip_execution_outcome_t execute_local_model_case(const kmip_test_case_t *test_case) {
    if (test_case->enum_type[0]) {
        if (strcmp(test_case->enum_type, "Item Type") == 0) {
            return roundtrip_all_ttlv_item_types()
                ? local_model_success()
                : local_model_failure("generic TTLV item-type roundtrip failed", test_case->enum_type);
        }
        const kmip_enum_probe_t *probe = find_enum_probe(test_case->enum_type);
        if (!probe || !probe->name_for_value || probe->tag == 0) {
            return outcome_not_run(
                "no standard field tag and named-value helper are mapped for this KMIP enumeration");
        }
        uint32_t known_value = 0;
        bool found_known_value = false;
        for (uint32_t candidate = 0; candidate <= UINT16_MAX; ++candidate) {
            if (probe->name_for_value(candidate)) {
                known_value = candidate;
                found_known_value = true;
                break;
            }
        }
        if (!found_known_value) {
            const kmip_enum_fallback_probe_t *fallback =
                find_enum_fallback_probe(test_case->enum_type);
            if (!fallback) {
                return outcome_not_run(
                    "the installed C named-value helper exposes no assigned value and no normative fallback is mapped");
            }
            known_value = fallback->raw_value;
        }
        if (!roundtrip_scalar_for_inspection(true, probe->tag, known_value) ||
            !roundtrip_scalar_for_inspection(true, probe->tag, UINT32_MAX)) {
            return local_model_failure("KMIP Enumeration roundtrip failed", test_case->enum_type);
        }
        return local_model_success();
    }
    if (test_case->bitmask_type[0]) {
        const kmip_bitmask_probe_t *probe = find_bitmask_probe(test_case->bitmask_type);
        if (!probe || !probe->known_bits || !probe->unknown_bits || probe->tag == 0) {
            return outcome_not_run("no standard tag and bit-preservation helpers are mapped for this KMIP mask");
        }
        uint32_t unknown_bit = UINT32_C(0x80000000);
        while (unknown_bit != 0 && (probe->known_mask & unknown_bit) != 0) unknown_bit >>= 1;
        uint32_t combined_mask = probe->known_mask | unknown_bit;
        int64_t unknown_as_integer = unknown_bit == UINT32_C(0x80000000)
            ? INT32_MIN : (int64_t)unknown_bit;
        if (unknown_bit == 0 ||
            probe->known_bits(combined_mask) != probe->known_mask ||
            probe->unknown_bits(combined_mask) != unknown_bit ||
            !roundtrip_scalar_for_inspection(false, probe->tag, probe->known_mask) ||
            !roundtrip_scalar_for_inspection(false, probe->tag, unknown_as_integer)) {
            return local_model_failure("KMIP bitmask construction or roundtrip failed",
                                       test_case->bitmask_type);
        }
        return local_model_success();
    }
    return outcome_not_run("local model catalog case has no enum or bitmask discriminator");
}

kmip_execution_outcome_t kmip_live_execute_case(const kmip_test_case_t *test_case,
                                                const kmip_attempt_context_t *attempt,
                                                void *user_data) {
    (void)attempt;
    kmip_live_client_t *client = user_data;
    if (!test_case || !client) {
        return outcome_not_run("live client is not initialized");
    }
    if (strcmp(test_case->mutation, "mutating") == 0 && !client->allow_mutations) {
        kmip_execution_outcome_t outcome = { .executed = true, .policy_restricted = true };
        snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                 "mutating operation requires configuration and command-line opt-in");
        return outcome;
    }
    if (strcmp(test_case->workflow_type, "server_to_client") == 0) {
        return outcome_not_run(
            "case requires server-initiated request reception, which the public C client API does not expose");
    }
    if (strcmp(client->api_level, "high-level") == 0) {
        return outcome_not_run(
            "the C binding exposes no high-level client route for this case");
    }
    if (strcmp(test_case->mode, "local_model") == 0) {
        return execute_local_model_case(test_case);
    }
    if (strcmp(client->api_level, "generic-ttlv") == 0) {
        return outcome_not_run(
            "the public C API can construct and inspect generic TTLV but cannot dispatch a generic request");
    }
    if (!client->client || !client->limits) {
        return outcome_not_run("live client is not initialized");
    }
    if (strcmp(test_case->mode, "server_exchange") != 0) {
        return outcome_not_run("catalog mode is not supported by the C validator");
    }
    if (strcmp(test_case->operation, "Discover Versions") == 0 &&
        client->protocol_minor == 0) {
        return outcome_not_run(
            "the public C Discover Versions builder only offers KMIP 2.1 and cannot construct a KMIP 2.0 offer");
    }
    if (strcmp(test_case->operation, "Login") == 0 && client->protocol_minor == 0) {
        return outcome_not_run(
            "the public C Login builder only supports KMIP 2.1");
    }
    bool get_attributes_workflow = strcmp(test_case->case_id, "test-op-get-attributes") == 0;
    if (strcmp(test_case->operation, "Get Attributes") == 0 && !get_attributes_workflow) {
        return outcome_not_run(
            "attribute-specific Get Attributes recipes are not yet mapped to isolated C fixtures");
    }

    kmipkit_client_batch_item_t *item = NULL;
    kmipkit_client_batch_item_t *setup_item = NULL;
    kmipkit_client_batch_item_t *cleanup_item = NULL;
    kmipkit_ttlv_structure_t *request_attributes = NULL;
    int32_t status = 0;
    const char *request_construction_step = "typed request";
    char request_validation_diagnostic[256] = {0};
    if (strcmp(test_case->operation, "Discover Versions") == 0) {
        status = kmipkit_client_batch_item_discover_versions(&item);
    } else if (strcmp(test_case->operation, "Ping") == 0 && client->protocol_minor == 1) {
        status = kmipkit_client_batch_item_ping(&item);
    } else if (strcmp(test_case->operation, "Locate") == 0) {
        status = kmipkit_ttlv_structure_create(&request_attributes);
        if (status == 0) {
            status = kmipkit_client_batch_item_locate(1, 3, 0, 0, 0, 0, 0, 0,
                                                      request_attributes, &item);
        }
    } else if (strcmp(test_case->operation, "Query") == 0) {
        const uint32_t query_function = KMIPKIT_ENUMERATION_QUERY_FUNCTION_QUERY_OPERATIONS;
        status = kmipkit_client_batch_item_query(&query_function, 1, 0, NULL, 0, &item);
    } else if (strcmp(test_case->operation, "Log") == 0) {
        static const uint8_t message[] = "KMIPKit interoperability validation probe";
        status = kmipkit_client_batch_item_log(message, sizeof(message) - 1, &item);
    } else if (strcmp(test_case->operation, "Login") == 0) {
        status = kmipkit_client_batch_item_login(0, 0, 0, 0, NULL, &item);
    } else if (get_attributes_workflow) {
        kmipkit_unique_identifier_v1_t placeholder = {
            .struct_size = (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
            .abi_version = 1,
            .kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION,
            .enumeration_value = KMIPKIT_ENUMERATION_UNIQUE_IDENTIFIER_ID_PLACEHOLDER
        };
        request_construction_step = "symmetric key Attributes";
        status = create_symmetric_key_attributes(client, &request_attributes) ? 0 : -1;
        if (status == 0) {
            request_construction_step = "Create Symmetric Key";
            status = kmipkit_client_batch_item_create(
                KMIPKIT_ENUMERATION_OBJECT_TYPE_SYMMETRIC_KEY, request_attributes, NULL, &setup_item);
            if (status == 0 && !request_item_is_valid(
                    setup_item, request_validation_diagnostic, sizeof(request_validation_diagnostic))) {
                status = KMIPKIT_ERROR_INVALID_INPUT;
            }
        }
        if (status == 0) {
            request_construction_step = "Get Attributes using ID Placeholder";
            status = kmipkit_client_batch_item_get_attributes(NULL, NULL, 0, &item);
            if (status == 0 && !request_item_is_valid(
                    item, request_validation_diagnostic, sizeof(request_validation_diagnostic))) {
                status = KMIPKIT_ERROR_INVALID_INPUT;
            }
        }
        if (status == 0) {
            request_construction_step = "Destroy using ID Placeholder";
            status = kmipkit_client_batch_item_destroy(&placeholder, &cleanup_item);
            if (status == 0 && !request_item_is_valid(
                    cleanup_item, request_validation_diagnostic, sizeof(request_validation_diagnostic))) {
                status = KMIPKIT_ERROR_INVALID_INPUT;
            }
        }
        if (status == 0) {
            request_construction_step = "Unique Batch Item IDs";
            status = set_batch_item_id(&setup_item, "interop-create");
            if (status == 0) status = set_batch_item_id(&item, "interop-attributes");
            if (status == 0) status = set_batch_item_id(&cleanup_item, "interop-destroy");
        }
    } else {
        return outcome_not_run("no installed typed C request route is mapped for this case");
    }
    if (status != 0) {
        kmip_execution_outcome_t outcome = outcome_not_run("KMIPKit rejected typed request construction");
        int32_t last_status = 0;
        uint32_t category = 0;
        uint32_t cause = 0;
        uint32_t delivery = 0;
        (void)kmipkit_last_error_get(&last_status, &category, &cause, &delivery);
        outcome.executed = true;
        outcome.library_failed = true;
        if (request_validation_diagnostic[0] != '\0') {
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit request validation failed at %.64s: %.128s",
                     request_construction_step, request_validation_diagnostic);
        } else {
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit request construction failed at %s (status %d, category=%u cause=%u)",
                     request_construction_step, status, category, cause);
        }
        return outcome;
    }

    kmipkit_client_batch_t *batch = NULL;
    kmipkit_client_batch_response_t *response = NULL;
    kmipkit_client_batch_item_response_t *response_item = NULL;
    kmipkit_client_batch_item_response_t *setup_response_item = NULL;
    kmipkit_client_batch_item_response_t *cleanup_response_item = NULL;
    kmipkit_ttlv_structure_t *response_attributes = NULL;
    kmipkit_unique_identifier_v1_t created_identifier = {
        .struct_size = (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
        .abi_version = 1
    };
    uint8_t created_identifier_present = 0;
    kmip_execution_outcome_t outcome = { .executed = true };
    status = kmipkit_client_batch_create(&batch);
    if (status == 0 && get_attributes_workflow) {
        status = kmipkit_client_batch_set_order_option(batch, 1);
        if (status == 0) status = kmipkit_client_batch_add_error_continuation_value(
            batch, KMIPKIT_ENUMERATION_BATCH_ERROR_CONTINUATION_OPTION_CONTINUE);
        if (status == 0) status = kmipkit_client_batch_add_item(batch, &setup_item);
        if (status == 0) status = kmipkit_client_batch_add_item(batch, &item);
        if (status == 0) status = kmipkit_client_batch_add_item(batch, &cleanup_item);
    } else if (status == 0) {
        status = kmipkit_client_batch_add_item(batch, &item);
    }
    if (status == 0 && client->username_env[0] != '\0') {
        kmipkit_authentication_t *authentication = NULL;
        if (!create_authentication(client, &authentication)) {
            outcome.library_failed = true;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit could not construct configured Username/Password authentication");
            goto cleanup;
        }
        status = kmipkit_client_batch_set_authentication(batch, authentication);
        if (status != 0) {
            kmipkit_authentication_release(authentication);
            outcome.library_failed = true;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit rejected configured authentication (status %d)", status);
            goto cleanup;
        }
    }
    if (status != 0) {
        outcome.library_failed = true;
        snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                 "KMIPKit could not assemble the typed request batch (status %d)", status);
        goto cleanup;
    }
    status = kmipkit_client_execute(client->client, &batch, client->limits, &response);
    if (status != 0) {
        int32_t last_status = 0;
        uint32_t category = 0;
        uint32_t cause = 0;
        uint32_t delivery = 0;
        (void)kmipkit_last_error_get(&last_status, &category, &cause, &delivery);
        snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                 "KMIPKit exchange failed with status %d (category=%u cause=%u delivery=%u)",
                 status, category, cause, delivery);
        outcome.success = false;
        goto cleanup;
    }

    uint64_t response_count = 0;
    status = kmipkit_client_batch_response_item_count(response, &response_count);
    uint64_t expected_response_count = get_attributes_workflow ? 3 : 1;
    if (status != 0 || response_count != expected_response_count) {
        outcome.library_failed = true;
        snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                 "KMIPKit returned an unexpected response item count");
        goto cleanup;
    }
    if (get_attributes_workflow) {
        kmipkit_client_batch_item_metadata_v2_t setup_metadata = {
            .struct_size = (uint32_t)sizeof(kmipkit_client_batch_item_metadata_v2_t),
            .abi_version = 2
        };
        kmipkit_client_batch_item_metadata_v2_t cleanup_metadata = {
            .struct_size = (uint32_t)sizeof(kmipkit_client_batch_item_metadata_v2_t),
            .abi_version = 2
        };
        status = kmipkit_client_batch_response_item_at(response, 0, &setup_response_item);
        if (status == 0) status = kmipkit_client_batch_response_item_at(response, 2, &cleanup_response_item);
        if (status == 0) status = kmipkit_client_batch_item_response_metadata_v2(
            setup_response_item, &setup_metadata);
        if (status == 0) status = kmipkit_client_batch_item_response_metadata_v2(
            cleanup_response_item, &cleanup_metadata);
        if (status != 0) {
            outcome.library_failed = true;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit could not read setup or cleanup result metadata (status %d)", status);
            goto cleanup;
        }
        if (setup_metadata.result_status != KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS ||
            cleanup_metadata.result_status != KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS) {
            const kmipkit_client_batch_item_metadata_v2_t *failure =
                setup_metadata.result_status != KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS
                    ? &setup_metadata : &cleanup_metadata;
            outcome.library_validated = true;
            outcome.has_result_status_code = true;
            outcome.result_status_code = failure->result_status;
            outcome.has_result_reason_code = failure->has_result_reason != 0;
            outcome.result_reason_code = failure->result_reason;
            outcome.server_unsupported = failure->has_result_reason != 0 &&
                (failure->result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_OPERATION_NOT_SUPPORTED ||
                 failure->result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_FEATURE_NOT_SUPPORTED);
            outcome.policy_restricted = failure->has_result_reason != 0 &&
                (failure->result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_NOT_AUTHORISED ||
                 failure->result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_PERMISSION_DENIED);
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "server rejected the isolated Create/Get Attributes/Destroy batch (status=%u reason=%u)",
                     failure->result_status, failure->result_reason);
            goto cleanup;
        }
        uint32_t created_object_type = 0;
        status = kmipkit_client_batch_item_response_create(
            setup_response_item, &created_identifier_present, &created_object_type, &created_identifier);
        if (status != 0 || !created_identifier_present ||
            created_object_type != KMIPKIT_ENUMERATION_OBJECT_TYPE_SYMMETRIC_KEY ||
            created_identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
            created_identifier.text_length == 0 || !created_identifier.text_data) {
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit did not expose the typed Create response for the isolated fixture");
            goto cleanup;
        }
    }
    status = kmipkit_client_batch_response_item_at(
        response, get_attributes_workflow ? 1 : 0, &response_item);
    if (status != 0) {
        outcome.library_failed = true;
        snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                 "KMIPKit could not read the batch result (status %d)", status);
        goto cleanup;
    }
    kmipkit_client_batch_item_metadata_v2_t metadata = {
        .struct_size = (uint32_t)sizeof(kmipkit_client_batch_item_metadata_v2_t),
        .abi_version = 2
    };
    status = kmipkit_client_batch_item_response_metadata_v2(response_item, &metadata);
    if (status != 0) {
        outcome.library_failed = true;
        snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                 "KMIPKit could not read KMIP result metadata (status %d)", status);
        goto cleanup;
    }
    outcome.has_result_status_code = true;
    outcome.library_validated = true;
    outcome.result_status_code = metadata.result_status;
    outcome.has_result_reason_code = metadata.has_result_reason != 0;
    outcome.result_reason_code = metadata.result_reason;
    outcome.success = metadata.result_status == KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS;
    if (!outcome.success && outcome.has_result_reason_code) {
        outcome.server_unsupported =
            metadata.result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_OPERATION_NOT_SUPPORTED ||
            metadata.result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_FEATURE_NOT_SUPPORTED;
        outcome.policy_restricted =
            metadata.result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_NOT_AUTHORISED ||
            metadata.result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_PERMISSION_DENIED;
    }
    if (!outcome.success) {
        snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                 "server result status=%u reason=%u",
                 metadata.result_status, metadata.result_reason);
        goto cleanup;
    }
    if (strcmp(test_case->operation, "Discover Versions") == 0) {
        kmipkit_protocol_version_v1_t versions[16] = {0};
        uint8_t present = 0;
        uint64_t version_count = 0;
        status = kmipkit_client_batch_item_response_discover_versions_supported_versions(
            response_item, &present, versions, 16, &version_count);
        if (status != 0) {
            outcome.success = false;
            outcome.library_failed = true;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit could not read the typed Discover Versions payload (status %d)", status);
            goto cleanup;
        }
        bool configured_version_found = false;
        for (uint64_t index = 0; index < version_count; ++index) {
            if (versions[index].major == client->protocol_major &&
                versions[index].minor == client->protocol_minor) configured_version_found = true;
        }
        if (!present || !configured_version_found) {
            outcome.success = false;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "server did not advertise the configured KMIP protocol version");
        }
    } else if (strcmp(test_case->operation, "Login") == 0) {
        uint8_t present = 0;
        uint8_t ticket_present = 0;
        uint32_t ticket_type = 0;
        kmipkit_byte_span_v1_t ticket_value = {0};
        status = kmipkit_client_batch_item_response_login(
            response_item, &present, &ticket_present, &ticket_type, &ticket_value);
        if (status != 0 || !present || !ticket_present ||
            ticket_type != KMIPKIT_ENUMERATION_TICKET_TYPE_LOGIN ||
            ticket_value.length == 0 || !ticket_value.data) {
            outcome.success = false;
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit did not expose a valid typed Login response");
        }
    } else if (strcmp(test_case->operation, "Ping") == 0) {
        uint8_t present = 0;
        status = kmipkit_client_batch_item_response_ping(response_item, &present);
        if (status != 0 || !present) {
            outcome.success = false;
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit did not expose a valid typed Ping response");
        }
    } else if (strcmp(test_case->operation, "Locate") == 0) {
        uint8_t present = 0;
        uint8_t payload_present = 0;
        uint8_t located_items_present = 0;
        int32_t located_items = 0;
        uint64_t identifier_count = 0;
        status = kmipkit_client_batch_item_response_locate(
            response_item, &present, &payload_present, &located_items_present,
            &located_items, &identifier_count);
        bool valid_response = status == 0 && present && payload_present;
        if (!valid_response) {
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit could not read the typed Locate payload (status=%d present=%u payload=%u)",
                     status, present, payload_present);
        }
        if (valid_response && located_items_present && located_items < 0) {
            valid_response = false;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "server returned a negative Located Items value");
        }
        for (uint64_t index = 0; valid_response && index < identifier_count; ++index) {
            kmipkit_unique_identifier_v1_t identifier = {
                .struct_size = (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
                .abi_version = 1
            };
            status = kmipkit_client_batch_item_response_locate_unique_identifier_at(
                response_item, index, &identifier);
            bool valid_identifier = identifier.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION ||
                identifier.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER ||
                (identifier.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING &&
                 (identifier.text_length == 0 || identifier.text_data != NULL));
            valid_response = status == 0 && valid_identifier;
            if (!valid_response) {
                snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                         "KMIPKit could not read Locate identifier %llu (status=%d kind=%u)",
                         (unsigned long long)index, status, (unsigned int)identifier.kind);
            }
        }
        if (!valid_response) {
            outcome.success = false;
            outcome.library_failed = status != 0;
        }
    } else if (strcmp(test_case->operation, "Query") == 0) {
        uint8_t present = 0;
        uint64_t field_count = 0;
        status = kmipkit_client_batch_item_response_query_field_count(
            response_item, &present, &field_count);
        if (status != 0 || !present || field_count == 0) {
            outcome.success = false;
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit did not expose Query response fields");
            goto cleanup;
        }
        bool operation_field_found = false;
        for (uint64_t index = 0; index < field_count; ++index) {
            uint8_t field_present = 0;
            uint8_t field_kind = 0;
            uint32_t field_tag = 0;
            uint32_t enumeration = 0;
            kmipkit_ttlv_item_t *field_item = NULL;
            status = kmipkit_client_batch_item_response_query_field_at(
                response_item, index, &field_present, &field_tag, &field_kind,
                &enumeration, &field_item);
            bool field_has_item = field_item != NULL;
            bool valid_field = status == 0 && field_present && field_kind >= 1 && field_kind <= 4 &&
                ((field_kind == 1 && !field_has_item) ||
                 (field_kind != 1 && field_has_item));
            if (valid_field && field_tag == UINT32_C(0x42005C) &&
                field_kind == 1 && enumeration != 0) {
                operation_field_found = true;
            }
            if (field_item) kmipkit_ttlv_item_release(field_item);
            if (!valid_field) {
                outcome.success = false;
                outcome.library_failed = status != 0 || field_present != 0;
                snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                         "KMIPKit returned a malformed typed Query response field");
                goto cleanup;
            }
        }
        if (!operation_field_found) {
            outcome.success = false;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit Query response omitted Operation enumeration fields");
        }
    } else if (get_attributes_workflow) {
        uint8_t present = 0;
        kmipkit_unique_identifier_v1_t response_identifier = {
            .struct_size = (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
            .abi_version = 1
        };
        status = kmipkit_client_batch_item_response_get_attributes(
            response_item, &present, &response_identifier, &response_attributes);
        bool valid_response = status == 0 && present && response_attributes &&
            same_text_identifier(&created_identifier, &response_identifier) &&
            attributes_match_created_symmetric_key(response_attributes);
        if (!valid_response) {
            outcome.success = false;
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit Get Attributes did not return the created AES-128 attributes");
            goto cleanup;
        }
        uint8_t destroyed_present = 0;
        kmipkit_unique_identifier_v1_t destroyed_identifier = {
            .struct_size = (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
            .abi_version = 1
        };
        status = kmipkit_client_batch_item_response_destroy(
            cleanup_response_item, &destroyed_present, &destroyed_identifier);
        if (status != 0 || !destroyed_present ||
            !same_text_identifier(&created_identifier, &destroyed_identifier)) {
            outcome.success = false;
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit did not confirm cleanup of the isolated test object");
        }
    }

cleanup:
    if (response_attributes) kmipkit_ttlv_structure_release(response_attributes);
    if (setup_response_item) kmipkit_client_batch_item_response_release(setup_response_item);
    if (cleanup_response_item) kmipkit_client_batch_item_response_release(cleanup_response_item);
    if (response_item) kmipkit_client_batch_item_response_release(response_item);
    if (response) kmipkit_client_batch_response_release(response);
    if (batch) kmipkit_client_batch_release(batch);
    if (item) kmipkit_client_batch_item_release(item);
    if (setup_item) kmipkit_client_batch_item_release(setup_item);
    if (cleanup_item) kmipkit_client_batch_item_release(cleanup_item);
    if (request_attributes) kmipkit_ttlv_structure_release(request_attributes);
    return outcome;
}
