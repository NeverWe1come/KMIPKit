#include "c_validator.h"
#include "json.h"
#include <stdio.h>
#include <string.h>

static bool string_in_list(const char *target, const char list[KMIP_MAX_CASES][KMIP_MAX_NAME_LEN], size_t count) {
    if (!target || target[0] == '\0') return true;
    for (size_t i = 0; i < count; i++) {
        if (strcmp(target, list[i]) == 0) return true;
    }
    return false;
}

static bool copy_member(const json_document_t *document, int object, const char *name,
                        char *destination, size_t capacity, bool required) {
    int token = json_object_get(document, object, name);
    if (token < 0) {
        if (required) return false;
        destination[0] = '\0';
        return true;
    }
    return json_string_copy(document, token, destination, capacity);
}

static bool load_supported_names(const json_document_t *document, int object,
                                 char values[KMIP_MAX_CASES][KMIP_MAX_NAME_LEN],
                                 size_t *out_count) {
    if (object < 0) return true;
    for (size_t i = 0;; ++i) {
        int key = -1;
        int value = -1;
        if (!json_object_member_at(document, object, i, &key, &value)) break;
        bool supported = false;
        int supported_token = json_object_get(document, value, "supported");
        if (!json_boolean(document, supported_token, &supported)) return false;
        if (!supported) continue;
        if (*out_count >= KMIP_MAX_CASES ||
            !json_string_copy(document, key, values[*out_count], KMIP_MAX_NAME_LEN)) return false;
        ++(*out_count);
    }
    return true;
}

bool kmip_load_capabilities(const char *json_path, kmip_server_capabilities_t *out_caps) {
    if (!json_path || !out_caps) return false;
    memset(out_caps, 0, sizeof(*out_caps));
    json_document_t document;
    if (!json_document_load(json_path, &document)) return false;
    int root = 0;
    int server = json_object_get(&document, root, "server");
    int operations = json_object_get(&document, root, "operations");
    int objects = json_object_get(&document, root, "managed_objects");
    int attributes = json_object_get(&document, root, "attributes");
    bool valid = server >= 0 && operations >= 0 &&
        copy_member(&document, server, "id", out_caps->server_id,
                    sizeof(out_caps->server_id), true) &&
        load_supported_names(&document, operations, out_caps->supported_operations,
                            &out_caps->supported_operations_count) &&
        load_supported_names(&document, objects, out_caps->supported_objects,
                            &out_caps->supported_objects_count) &&
        load_supported_names(&document, attributes, out_caps->supported_attributes,
                            &out_caps->supported_attributes_count);
    json_document_release(&document);
    return valid;
}

static bool is_fingerprint(const char *value) {
    if (strlen(value) != 64) return false;
    for (size_t i = 0; i < 64; ++i) {
        if (!((value[i] >= '0' && value[i] <= '9') ||
              (value[i] >= 'a' && value[i] <= 'f'))) return false;
    }
    return true;
}

static bool is_environment_name(const char *value) {
    if (!value || value[0] == '\0' ||
        !((value[0] >= 'A' && value[0] <= 'Z') || value[0] == '_')) return false;
    for (size_t i = 1; value[i] != '\0'; ++i) {
        if (!((value[i] >= 'A' && value[i] <= 'Z') ||
              (value[i] >= '0' && value[i] <= '9') || value[i] == '_')) return false;
    }
    return true;
}

bool kmip_load_connection(const char *json_path, kmip_connection_config_t *out_connection) {
    if (!json_path || !out_connection) return false;
    memset(out_connection, 0, sizeof(*out_connection));
    json_document_t document;
    if (!json_document_load(json_path, &document)) return false;
    int root = 0;
    int endpoints = json_object_get(&document, root, "endpoints");
    int raw_tls = json_object_get(&document, endpoints, "raw_tls");
    int https = json_object_get(&document, endpoints, "https");
    int tls = json_object_get(&document, root, "tls");
    int environment = json_object_get(&document, root, "environment");
    int policy = json_object_get(&document, root, "execution_policy");
    int authentication = json_object_get(&document, root, "authentication");
    int username_password = json_object_get(&document, authentication, "username_password");
    int confirmed = json_object_get(&document, environment, "dedicated_test_environment_confirmed");
    int probe = json_object_get(&document, policy, "probe_unsupported");
    int mutations = json_object_get(&document, policy, "allow_mutations");
    long port = 0;
    bool valid = copy_member(&document, root, "server_id", out_connection->server_id,
                             sizeof(out_connection->server_id), true) &&
        copy_member(&document, root, "server_vendor", out_connection->server_vendor,
                    sizeof(out_connection->server_vendor), false) &&
        copy_member(&document, root, "server_product", out_connection->server_product,
                    sizeof(out_connection->server_product), true) &&
        copy_member(&document, root, "server_version", out_connection->server_version,
                    sizeof(out_connection->server_version), true) &&
        copy_member(&document, root, "server_configuration_fingerprint",
                    out_connection->server_configuration_fingerprint,
                    sizeof(out_connection->server_configuration_fingerprint), true) &&
        copy_member(&document, root, "protocol_version", out_connection->protocol_version,
                    sizeof(out_connection->protocol_version), true) &&
        copy_member(&document, root, "transport", out_connection->transport,
                    sizeof(out_connection->transport), true) &&
        copy_member(&document, root, "encoding", out_connection->encoding,
                    sizeof(out_connection->encoding), true) &&
        copy_member(&document, root, "api_level", out_connection->api_level,
                    sizeof(out_connection->api_level), true) &&
        json_boolean(&document, confirmed, &out_connection->dedicated_test_environment_confirmed) &&
        json_boolean(&document, probe, &out_connection->probe_unsupported) &&
        json_boolean(&document, mutations, &out_connection->allow_mutations);
    char environment_kind[32];
    valid = valid && copy_member(&document, environment, "kind", environment_kind,
                                 sizeof(environment_kind), true) &&
        strcmp(environment_kind, "dedicated_test") == 0 &&
        out_connection->dedicated_test_environment_confirmed &&
        is_fingerprint(out_connection->server_configuration_fingerprint) &&
        (strcmp(out_connection->protocol_version, "2.0") == 0 ||
         strcmp(out_connection->protocol_version, "2.1") == 0) &&
        (strcmp(out_connection->transport, "raw_tls") == 0 ||
         strcmp(out_connection->transport, "https") == 0) &&
        (strcmp(out_connection->encoding, "TTLV") == 0 ||
         strcmp(out_connection->encoding, "JSON") == 0 ||
         strcmp(out_connection->encoding, "XML") == 0) &&
        (strcmp(out_connection->api_level, "high-level") == 0 ||
         strcmp(out_connection->api_level, "typed-protocol") == 0 ||
         strcmp(out_connection->api_level, "generic-ttlv") == 0);
    if (valid && strcmp(out_connection->transport, "raw_tls") == 0) {
        valid = raw_tls >= 0 &&
            copy_member(&document, raw_tls, "host", out_connection->host,
                        sizeof(out_connection->host), true) &&
            json_integer(&document, json_object_get(&document, raw_tls, "port"), &port) &&
            port > 0 && port <= UINT16_MAX;
        if (valid) out_connection->port = (uint16_t)port;
    } else if (valid) {
        valid = https >= 0 && copy_member(&document, https, "url", out_connection->https_url,
                                          sizeof(out_connection->https_url), true);
    }
    if (valid && tls >= 0) {
        valid = copy_member(&document, tls, "ca_cert_path", out_connection->ca_cert_path,
                            sizeof(out_connection->ca_cert_path), false) &&
            copy_member(&document, tls, "client_cert_path", out_connection->client_cert_path,
                        sizeof(out_connection->client_cert_path), false) &&
            copy_member(&document, tls, "client_key_path", out_connection->client_key_path,
                        sizeof(out_connection->client_key_path), false) &&
            copy_member(&document, tls, "server_name_override", out_connection->server_name_override,
                        sizeof(out_connection->server_name_override), false);
    }
    if (valid && authentication >= 0 && username_password >= 0) {
        valid = copy_member(&document, username_password, "username_env", out_connection->username_env,
                            sizeof(out_connection->username_env), true) &&
            copy_member(&document, username_password, "password_env", out_connection->password_env,
                        sizeof(out_connection->password_env), true) &&
            is_environment_name(out_connection->username_env) &&
            is_environment_name(out_connection->password_env);
    }
    json_document_release(&document);
    return valid;
}

static bool load_optional_requires(const json_document_t *document, int requires,
                                   const char *field, char *destination, size_t capacity) {
    return copy_member(document, requires, field, destination, capacity, false);
}

bool kmip_load_catalog(const char *json_path, kmip_test_case_t *out_cases, size_t *out_count) {
    if (!json_path || !out_cases || !out_count) return false;
    *out_count = 0;
    json_document_t document;
    if (!json_document_load(json_path, &document)) return false;
    int cases = json_object_get(&document, 0, "test_cases");
    size_t count = json_array_length(&document, cases);
    if (cases < 0 || count == 0 || count > KMIP_MAX_CASES) {
        json_document_release(&document);
        return false;
    }
    bool valid = true;
    for (size_t i = 0; i < count && valid; ++i) {
        int row = json_array_get(&document, cases, i);
        int requires = json_object_get(&document, row, "requires");
        int execution = json_object_get(&document, row, "execution");
        int versions = json_object_get(&document, execution, "protocol_versions");
        int normative = json_object_get(&document, row, "normative_refs");
        kmip_test_case_t *test_case = &out_cases[i];
        memset(test_case, 0, sizeof(*test_case));
        valid = row >= 0 && execution >= 0 && requires >= 0 && versions >= 0 &&
            copy_member(&document, row, "case_id", test_case->case_id,
                        sizeof(test_case->case_id), true) &&
            copy_member(&document, row, "operation", test_case->operation,
                        sizeof(test_case->operation), false) &&
            copy_member(&document, row, "category", test_case->category,
                        sizeof(test_case->category), true) &&
            copy_member(&document, execution, "mode", test_case->mode,
                        sizeof(test_case->mode), true) &&
            copy_member(&document, execution, "mutation", test_case->mutation,
                        sizeof(test_case->mutation), true) &&
            load_optional_requires(&document, requires, "object_type", test_case->object_type,
                                   sizeof(test_case->object_type)) &&
            load_optional_requires(&document, requires, "attribute", test_case->attribute,
                                   sizeof(test_case->attribute)) &&
            load_optional_requires(&document, requires, "workflow_type", test_case->workflow_type,
                                   sizeof(test_case->workflow_type)) &&
            load_optional_requires(&document, requires, "enum_type", test_case->enum_type,
                                   sizeof(test_case->enum_type)) &&
            load_optional_requires(&document, requires, "bitmask_type", test_case->bitmask_type,
                                   sizeof(test_case->bitmask_type));
        if (!valid) break;
        size_t version_count = json_array_length(&document, versions);
        if (version_count == 0 || version_count > KMIP_MAX_VERSIONS) { valid = false; break; }
        for (size_t j = 0; j < version_count; ++j) {
            char version[8];
            int version_token = json_array_get(&document, versions, j);
            if (!json_string_copy(&document, version_token, version, sizeof(version))) {
                valid = false;
                break;
            }
            if (strcmp(version, "2.0") == 0) test_case->protocol_version_mask |= KMIP_VERSION_MASK_2_0;
            else if (strcmp(version, "2.1") == 0) test_case->protocol_version_mask |= KMIP_VERSION_MASK_2_1;
            else { valid = false; break; }
        }
        for (size_t j = 0; valid && j < json_array_length(&document, normative); ++j) {
            int reference = json_array_get(&document, normative, j);
            char name[64];
            char section[32];
            if (!copy_member(&document, reference, "document", name, sizeof(name), true) ||
                !copy_member(&document, reference, "section", section, sizeof(section), true)) {
                valid = false;
                break;
            }
            size_t version_index = strcmp(name, "OASIS KMIP 2.0 Specification") == 0 ? 0 :
                                  strcmp(name, "OASIS KMIP 2.1 Specification") == 0 ? 1 : KMIP_MAX_VERSIONS;
            if (version_index >= KMIP_MAX_VERSIONS || test_case->normative_sections[version_index][0] != '\0') {
                valid = false;
                break;
            }
            memcpy(test_case->normative_sections[version_index], section, strlen(section) + 1);
        }
        if (test_case->protocol_version_mask == 0 ||
            (strcmp(test_case->mode, "local_model") != 0 && strcmp(test_case->mode, "server_exchange") != 0) ||
            (strcmp(test_case->mutation, "read_only") != 0 && strcmp(test_case->mutation, "mutating") != 0)) {
            valid = false;
        }
    }
    json_document_release(&document);
    if (!valid) return false;
    *out_count = count;
    return true;
}

bool kmip_is_case_supported(
    const kmip_server_capabilities_t *caps,
    const kmip_test_case_t *test_case,
    char out_reasons[KMIP_MAX_REASONS][KMIP_MAX_NAME_LEN],
    size_t *out_reasons_count
) {
    if (!caps || !test_case || !out_reasons || !out_reasons_count) return false;
    *out_reasons_count = 0;

    if (test_case->operation[0] != '\0' && !string_in_list(test_case->operation, caps->supported_operations, caps->supported_operations_count)) {
        snprintf(out_reasons[*out_reasons_count], KMIP_MAX_NAME_LEN, "Operation '%.100s' unsupported", test_case->operation);
        (*out_reasons_count)++;
    }

    if (test_case->object_type[0] != '\0' && !string_in_list(test_case->object_type, caps->supported_objects, caps->supported_objects_count)) {
        if (*out_reasons_count < KMIP_MAX_REASONS) {
            snprintf(out_reasons[*out_reasons_count], KMIP_MAX_NAME_LEN, "Managed object '%.100s' unsupported", test_case->object_type);
            (*out_reasons_count)++;
        }
    }

    if (test_case->attribute[0] != '\0' && !string_in_list(test_case->attribute, caps->supported_attributes, caps->supported_attributes_count)) {
        if (*out_reasons_count < KMIP_MAX_REASONS) {
            snprintf(out_reasons[*out_reasons_count], KMIP_MAX_NAME_LEN, "Attribute '%.100s' unsupported", test_case->attribute);
            (*out_reasons_count)++;
        }
    }

    return (*out_reasons_count == 0);
}

void kmip_init_attempt_context(
    kmip_attempt_context_t *ctx,
    const char *run_id,
    const char *lang,
    const char *enc,
    const char *level,
    const char *prefix
) {
    if (!ctx) return;
    memset(ctx, 0, sizeof(*ctx));
    strncpy(ctx->run_id, run_id ? run_id : "run-c-1", sizeof(ctx->run_id) - 1);
    strncpy(ctx->language, lang ? lang : "c", sizeof(ctx->language) - 1);
    strncpy(ctx->encoding, enc ? enc : "TTLV", sizeof(ctx->encoding) - 1);
    strncpy(ctx->api_level, level ? level : "level1", sizeof(ctx->api_level) - 1);

    static unsigned int counter = 1000;
    snprintf(ctx->unique_name, sizeof(ctx->unique_name), "%s-%s-%s-%s-%u",
             prefix ? prefix : "kmipkit", ctx->language, ctx->encoding, ctx->api_level, counter++);
}

void kmip_run_suite(
    const kmip_server_capabilities_t *caps,
    const kmip_test_case_t *cases,
    size_t case_count,
    const kmip_driver_vtable_t *driver,
    bool probe_unsupported,
    kmip_validation_summary_t *out_summary
) {
    if (!caps || !cases || !driver || !out_summary) return;
    /* Capability profiles are unverified hypotheses; they never suppress a probe. */
    (void)probe_unsupported;
    memset(out_summary, 0, sizeof(*out_summary));
    strncpy(out_summary->server_id, caps->server_id, sizeof(out_summary->server_id) - 1);
    strncpy(out_summary->language, "c", sizeof(out_summary->language) - 1);
    out_summary->total_cases = case_count;

    for (size_t i = 0; i < case_count && i < KMIP_MAX_CASES; i++) {
        const kmip_test_case_t *tc = &cases[i];
        kmip_case_result_t *res = &out_summary->results[out_summary->results_count++];
        strncpy(res->case_id, tc->case_id, sizeof(res->case_id) - 1);
        strncpy(res->operation, tc->operation, sizeof(res->operation) - 1);

        kmip_attempt_context_t attempt;
        kmip_init_attempt_context(&attempt, "run-c", "c", "TTLV", "typed-protocol", tc->case_id);
        kmip_execution_outcome_t outcome = driver->execute_case(tc, &attempt, driver->user_data);
        if (!outcome.executed) {
            res->status = CASE_STATUS_NOT_RUN;
            strncpy(res->detail, outcome.result_reason, sizeof(res->detail) - 1);
            out_summary->not_run++;
        } else if (outcome.policy_restricted) {
            res->status = CASE_STATUS_POLICY_RESTRICTED;
            strncpy(res->detail, outcome.result_reason, sizeof(res->detail) - 1);
            out_summary->policy_restricted++;
        } else if (outcome.server_unsupported) {
            res->status = CASE_STATUS_SERVER_UNSUPPORTED;
            strncpy(res->detail, outcome.result_reason, sizeof(res->detail) - 1);
            out_summary->unsupported++;
        } else if (outcome.success) {
            res->status = CASE_STATUS_PASSED;
            out_summary->passed++;
        } else {
            res->status = CASE_STATUS_FAILED;
            strncpy(res->detail, outcome.result_reason, sizeof(res->detail) - 1);
            out_summary->failed++;
        }
        res->has_result_status_code = outcome.has_result_status_code;
        res->has_result_reason_code = outcome.has_result_reason_code;
        res->result_status_code = outcome.result_status_code;
        res->result_reason_code = outcome.result_reason_code;
        const char *correctness = outcome.library_failed ? "failed" :
            outcome.library_validated ? "passed" : "not_checked";
        strncpy(res->library_correctness, correctness, sizeof(res->library_correctness) - 1);
    }
}

static const char *case_status_name(kmip_case_status_t status) {
    switch (status) {
        case CASE_STATUS_PASSED: return "passed";
        case CASE_STATUS_FAILED: return "failed";
        case CASE_STATUS_SERVER_UNSUPPORTED: return "server_unsupported";
        case CASE_STATUS_POLICY_RESTRICTED: return "policy_restricted";
        case CASE_STATUS_NOT_RUN: return "not_run";
    }
    return "not_run";
}

static const char *library_correctness_name(const kmip_case_result_t *result);
static const char *server_compatibility_name(const kmip_case_result_t *result);

void kmip_render_ansi(const kmip_validation_summary_t *summary) {
    if (!summary) return;
    printf("\n=== KMIP %s Validation Matrix: Server [%s] (Lang: %s) ===\n",
           summary->protocol_version, summary->server_id, summary->language);
    printf("Total: %zu | Passed: %zu | Failed: %zu | Server unsupported: %zu | "
           "Policy restricted: %zu | Not run: %zu\n",
           summary->total_cases, summary->passed, summary->failed, summary->unsupported,
           summary->policy_restricted, summary->not_run);
    printf("--------------------------------------------------------------------------------\n");
    printf("%-35s | %-25s | %-10s\n", "Case ID", "Operation", "Status");
    printf("--------------------------------------------------------------------------------\n");
    for (size_t i = 0; i < summary->results_count; i++) {
        const kmip_case_result_t *r = &summary->results[i];
        const char *st = case_status_name(r->status);
        printf("%-35s | %-25s | %-10s\n", r->case_id, r->operation, st);
    }
    printf("--------------------------------------------------------------------------------\n");
}

bool kmip_render_markdown(const kmip_validation_summary_t *summary, const char *file_path) {
    if (!summary || !file_path) return false;
    FILE *f = fopen(file_path, "w");
    if (!f) return false;

    fprintf(f, "# KMIP %s Interoperability Report: `%s`\n\n",
            summary->protocol_version, summary->server_id);
    fprintf(f, "- **Product/version**: `%s` / `%s`\n",
            summary->server_product, summary->server_version);
    fprintf(f, "- **Configuration fingerprint**: `%s`\n", summary->server_configuration_fingerprint);
    fprintf(f, "- **Library commit**: `%s`\n", summary->library_commit);
    fprintf(f, "- **Library artifact SHA-256**: `%s`\n", summary->artifact_sha256);
    fprintf(f, "- **Catalog SHA-256**: `%s`\n", summary->catalog_sha256);
    fprintf(f, "- **Language**: `%s`\n", summary->language);
    fprintf(f, "- **Total Cases**: %zu\n\n", summary->total_cases);
    fprintf(f, "### Summary\n\n");
    fprintf(f, "| Metric | Count |\n|---|---:|\n");
    fprintf(f, "| Total Cases | %zu |\n", summary->total_cases);
    fprintf(f, "| ✅ Passed | %zu |\n", summary->passed);
    fprintf(f, "| ❌ Failed | %zu |\n", summary->failed);
    fprintf(f, "| Server unsupported | %zu |\n", summary->unsupported);
    fprintf(f, "| Policy restricted | %zu |\n", summary->policy_restricted);
    fprintf(f, "| Not run | %zu |\n\n", summary->not_run);

    fprintf(f, "### Detailed Operations\n\n");
    fprintf(f, "| Case ID | Operation | Status | Library correctness | Server compatibility | Parity |\n");
    fprintf(f, "|---|---|:---:|---|---|---|\n");
    for (size_t i = 0; i < summary->results_count; i++) {
        const kmip_case_result_t *r = &summary->results[i];
        const char *st = case_status_name(r->status);
        fprintf(f, "| `%s` | %s | %s | %s | %s | not_compared |\n",
                r->case_id, r->operation, st, library_correctness_name(r),
                server_compatibility_name(r));
    }

    fclose(f);
    return true;
}

bool kmip_render_html(const kmip_validation_summary_t *summary, const char *file_path) {
    if (!summary || !file_path) return false;
    FILE *f = fopen(file_path, "w");
    if (!f) return false;

    fprintf(f, "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"UTF-8\">\n<title>KMIP %s Validation Report</title>\n",
            summary->protocol_version);
    fprintf(f, "<style>\nbody { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 2rem; background: #f8fafc; color: #1e293b; }\n");
    fprintf(f, ".card { background: white; border-radius: 8px; padding: 1.5rem; box-shadow: 0 1px 3px rgba(0,0,0,0.1); margin-bottom: 2rem; }\n");
    fprintf(f, "table { width: 100%%; border-collapse: collapse; margin-top: 1rem; }\n");
    fprintf(f, "th, td { text-align: left; padding: 0.75rem 1rem; border-bottom: 1px solid #e2e8f0; }\n");
    fprintf(f, "th { background: #f1f5f9; font-weight: 600; }\n");
    fprintf(f, ".pass { color: #16a34a; font-weight: bold; }\n.fail { color: #dc2626; font-weight: bold; }\n.unsupp { color: #64748b; font-weight: bold; }\n</style>\n</head>\n<body>\n");
    fprintf(f, "<div class=\"card\">\n<h1>KMIP %s Interoperability Report: %s</h1>\n",
            summary->protocol_version, summary->server_id);
    fprintf(f, "<p><strong>Product/version:</strong> %s / %s<br>\n<strong>Configuration fingerprint:</strong> %s<br>\n<strong>Library commit:</strong> %s<br>\n<strong>Artifact SHA-256:</strong> %s<br>\n<strong>Catalog SHA-256:</strong> %s</p>\n",
            summary->server_product, summary->server_version,
            summary->server_configuration_fingerprint, summary->library_commit,
            summary->artifact_sha256, summary->catalog_sha256);
    fprintf(f, "<p><strong>Language:</strong> %s | <strong>Passed:</strong> %zu | <strong>Failed:</strong> %zu | <strong>Unsupported:</strong> %zu | <strong>Policy restricted:</strong> %zu | <strong>Not run:</strong> %zu</p>\n</div>\n",
            summary->language, summary->passed, summary->failed, summary->unsupported,
            summary->policy_restricted, summary->not_run);
    fprintf(f, "<div class=\"card\">\n<table>\n<thead>\n<tr><th>Case ID</th><th>Operation</th><th>Status</th><th>Library correctness</th><th>Server compatibility</th><th>Parity</th></tr>\n</thead>\n<tbody>\n");

    for (size_t i = 0; i < summary->results_count; i++) {
        const kmip_case_result_t *r = &summary->results[i];
        const char *cls = case_status_name(r->status);
        const char *st = case_status_name(r->status);
        fprintf(f, "<tr><td><code>%s</code></td><td>%s</td><td class=\"%s\">%s</td><td>%s</td><td>%s</td><td>not_compared</td></tr>\n",
                r->case_id, r->operation, cls, st, library_correctness_name(r),
                server_compatibility_name(r));
    }
    fprintf(f, "</tbody>\n</table>\n</div>\n</body>\n</html>\n");

    fclose(f);
    return true;
}

static void json_write_string(FILE *file, const char *value) {
    fputc('"', file);
    for (const unsigned char *cursor = (const unsigned char *)value; *cursor; ++cursor) {
        switch (*cursor) {
            case '"': fputs("\\\"", file); break;
            case '\\': fputs("\\\\", file); break;
            case '\b': fputs("\\b", file); break;
            case '\f': fputs("\\f", file); break;
            case '\n': fputs("\\n", file); break;
            case '\r': fputs("\\r", file); break;
            case '\t': fputs("\\t", file); break;
            default:
                if (*cursor < 0x20) fprintf(file, "\\u%04x", *cursor);
                else fputc(*cursor, file);
        }
    }
    fputc('"', file);
}

static const char *library_correctness_name(const kmip_case_result_t *result) {
    return result->library_correctness[0] ? result->library_correctness : "not_checked";
}

static const char *server_compatibility_name(const kmip_case_result_t *result) {
    if (!result) return "unknown";
    if (result->status == CASE_STATUS_SERVER_UNSUPPORTED) return "unsupported";
    if (result->status == CASE_STATUS_POLICY_RESTRICTED) return "policy_restricted";
    if (!result->has_result_status_code) return "unknown";
    switch (result->status) {
        case CASE_STATUS_PASSED: return "supported";
        case CASE_STATUS_SERVER_UNSUPPORTED: return "unsupported";
        case CASE_STATUS_POLICY_RESTRICTED: return "policy_restricted";
        case CASE_STATUS_FAILED:
        case CASE_STATUS_NOT_RUN: return "unknown";
    }
    return "unknown";
}

bool kmip_render_json(const kmip_validation_summary_t *summary, const char *file_path) {
    if (!summary || !file_path) return false;
    FILE *f = fopen(file_path, "w");
    if (!f) return false;

    fputs("{\n  \"schema_version\": \"2.0\",\n  \"run_id\": ", f);
    json_write_string(f, summary->run_id);
    fputs(",\n  \"tested_at\": ", f);
    json_write_string(f, summary->tested_at);
    fputs(",\n  \"language\": \"c\",\n  \"server\": {\n    \"id\": ", f);
    json_write_string(f, summary->server_id);
    if (summary->server_vendor[0]) {
        fputs(",\n    \"vendor\": ", f);
        json_write_string(f, summary->server_vendor);
    }
    fputs(",\n    \"product\": ", f);
    json_write_string(f, summary->server_product);
    fputs(",\n    \"version\": ", f);
    json_write_string(f, summary->server_version);
    fputs(",\n    \"configuration_fingerprint\": ", f);
    json_write_string(f, summary->server_configuration_fingerprint);
    fputs("\n  },\n  \"protocol_version\": ", f);
    json_write_string(f, summary->protocol_version);
    fputs(",\n  \"catalog_sha256\": ", f);
    json_write_string(f, summary->catalog_sha256);
    fputs(",\n  \"execution\": {\n    \"transport\": ", f);
    json_write_string(f, summary->transport);
    fputs(",\n    \"encoding\": ", f);
    json_write_string(f, summary->encoding);
    fputs(",\n    \"api_level\": ", f);
    json_write_string(f, summary->api_level);
    fputs(",\n    \"library_commit\": ", f);
    json_write_string(f, summary->library_commit);
    fputs(",\n    \"artifact_sha256\": ", f);
    json_write_string(f, summary->artifact_sha256);
    fprintf(f, "\n  },\n  \"summary\": {\n    \"total\": %zu,\n    \"passed\": %zu,\n    \"failed\": %zu,\n    \"unsupported\": %zu,\n    \"policy_restricted\": %zu,\n    \"not_run\": %zu\n  },\n  \"cases\": [\n",
            summary->total_cases, summary->passed, summary->failed, summary->unsupported,
            summary->policy_restricted, summary->not_run);
    for (size_t i = 0; i < summary->results_count; i++) {
        const kmip_case_result_t *r = &summary->results[i];
        fputs("    {\n      \"case_id\": ", f);
        json_write_string(f, r->case_id);
        fputs(",\n      \"status\": ", f);
        json_write_string(f, case_status_name(r->status));
        fputs(",\n      \"library_correctness\": ", f);
        json_write_string(f, library_correctness_name(r));
        fputs(",\n      \"server_compatibility\": ", f);
        json_write_string(f, server_compatibility_name(r));
        fputs(",\n      \"parity\": \"not_compared\"", f);
        if (r->has_result_status_code)
            fprintf(f, ",\n      \"result_status_code\": %u", r->result_status_code);
        if (r->has_result_reason_code)
            fprintf(f, ",\n      \"result_reason_code\": %u", r->result_reason_code);
        if (r->detail[0]) {
            fputs(",\n      \"diagnostic\": ", f);
            json_write_string(f, r->detail);
        }
        fprintf(f, "\n    }%s\n", (i + 1 < summary->results_count) ? "," : "");
    }
    fprintf(f, "  ]\n}\n");

    return fclose(f) == 0;
}

bool kmip_verify_equivalence(
    const kmip_normalized_response_t *responses,
    size_t count,
    char out_discrepancies[KMIP_MAX_REASONS][KMIP_MAX_NAME_LEN],
    size_t *out_discrepancies_count
) {
    if (!responses || !out_discrepancies || !out_discrepancies_count) return false;
    *out_discrepancies_count = 0;
    if (count <= 1) return true;

    const kmip_normalized_response_t *base = &responses[0];
    for (size_t i = 1; i < count; i++) {
        const kmip_normalized_response_t *cur = &responses[i];
        if (strcmp(base->status, cur->status) != 0) {
            if (*out_discrepancies_count < KMIP_MAX_REASONS) {
                snprintf(out_discrepancies[(*out_discrepancies_count)++], KMIP_MAX_NAME_LEN,
                         "Response %zu status '%s' differs from baseline '%s'", i, cur->status, base->status);
            }
        }
        if (strcmp(base->result_reason, cur->result_reason) != 0) {
            if (*out_discrepancies_count < KMIP_MAX_REASONS) {
                snprintf(out_discrepancies[(*out_discrepancies_count)++], KMIP_MAX_NAME_LEN,
                         "Response %zu result_reason '%.64s' differs from baseline '%.64s'", i, cur->result_reason, base->result_reason);
            }
        }
        if (strcmp(base->object_type, cur->object_type) != 0) {
            if (*out_discrepancies_count < KMIP_MAX_REASONS) {
                snprintf(out_discrepancies[(*out_discrepancies_count)++], KMIP_MAX_NAME_LEN,
                         "Response %zu object_type '%.64s' differs from baseline '%.64s'", i, cur->object_type, base->object_type);
            }
        }
        if (strcmp(base->sanitized_payload_hex, cur->sanitized_payload_hex) != 0) {
            if (*out_discrepancies_count < KMIP_MAX_REASONS) {
                snprintf(out_discrepancies[(*out_discrepancies_count)++], KMIP_MAX_NAME_LEN,
                         "Response %zu payload differs from baseline", i);
            }
        }
    }
    return (*out_discrepancies_count == 0);
}

bool kmip_run_recipe_with_teardown(
    bool (*setup_fn)(void *user_data),
    bool (*target_fn)(void *user_data),
    bool (*teardown_fn)(void *user_data),
    void *user_data
) {
    if (setup_fn && !setup_fn(user_data)) {
        return false;
    }

    bool target_ok = false;
    if (target_fn) {
        target_ok = target_fn(user_data);
    }

    /* Teardown is guaranteed to execute */
    bool teardown_ok = true;
    if (teardown_fn) {
        teardown_ok = teardown_fn(user_data);
    }

    return target_ok && teardown_ok;
}
