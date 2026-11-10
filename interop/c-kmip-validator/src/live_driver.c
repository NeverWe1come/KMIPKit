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

static bool roundtrip_scalar_for_inspection(bool enumeration, int32_t raw_value) {
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
    if (status == 0) status = kmipkit_ttlv_raw_tag_create(UINT32_C(0x420057), &raw_tag);
    if (status == 0) status = kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag);
    if (status == 0) {
        status = enumeration
            ? kmipkit_ttlv_value_enumeration((uint32_t)raw_value, &value)
            : kmipkit_ttlv_value_integer(raw_value, &value);
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
                actual_integer == raw_value;
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

static kmip_execution_outcome_t execute_local_model_case(const kmip_test_case_t *test_case) {
    if (test_case->enum_type[0]) {
        if (roundtrip_scalar_for_inspection(true, 1) &&
            roundtrip_scalar_for_inspection(true, INT32_MAX)) {
            return (kmip_execution_outcome_t){
                .executed = true, .success = true, .library_validated = true
            };
        }
    } else if (test_case->bitmask_type[0]) {
        if (roundtrip_scalar_for_inspection(false, 1) &&
            roundtrip_scalar_for_inspection(false, INT32_MIN)) {
            return (kmip_execution_outcome_t){
                .executed = true, .success = true, .library_validated = true
            };
        }
    }
    return outcome_not_run(
        "generic enum and bitmask construction/read routes do not cover this local case shape");
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

    kmipkit_client_batch_item_t *item = NULL;
    kmipkit_ttlv_structure_t *request_attributes = NULL;
    int32_t status = 0;
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
    } else {
        return outcome_not_run("no installed typed C request route is mapped for this case");
    }
    if (status != 0) {
        kmip_execution_outcome_t outcome = outcome_not_run("KMIPKit rejected typed request construction");
        outcome.executed = true;
        outcome.library_failed = true;
        snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                 "KMIPKit typed request construction failed with status %d", status);
        return outcome;
    }

    kmipkit_client_batch_t *batch = NULL;
    kmipkit_client_batch_response_t *response = NULL;
    kmipkit_client_batch_item_response_t *response_item = NULL;
    kmip_execution_outcome_t outcome = { .executed = true };
    status = kmipkit_client_batch_create(&batch);
    if (status == 0) status = kmipkit_client_batch_add_item(batch, &item);
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
    if (status != 0 || response_count != 1) {
        outcome.library_failed = true;
        snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                 "KMIPKit returned an unexpected response item count");
        goto cleanup;
    }
    status = kmipkit_client_batch_response_item_at(response, 0, &response_item);
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
    }

cleanup:
    if (response_item) kmipkit_client_batch_item_response_release(response_item);
    if (response) kmipkit_client_batch_response_release(response);
    if (batch) kmipkit_client_batch_release(batch);
    if (item) kmipkit_client_batch_item_release(item);
    if (request_attributes) kmipkit_ttlv_structure_release(request_attributes);
    return outcome;
}
