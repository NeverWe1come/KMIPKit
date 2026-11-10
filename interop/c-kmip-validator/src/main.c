#include "c_validator.h"
#include "live_driver.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#ifndef KMIPKIT_LIBRARY_COMMIT
#define KMIPKIT_LIBRARY_COMMIT "unknown"
#endif
#ifndef KMIPKIT_C_INSTALL_DIR
#define KMIPKIT_C_INSTALL_DIR ""
#endif
#ifndef KMIPKIT_C_ARTIFACT_SHA256
#define KMIPKIT_C_ARTIFACT_SHA256 ""
#endif
#ifndef KMIPKIT_C_CATALOG_SHA256
#define KMIPKIT_C_CATALOG_SHA256 ""
#endif

static void print_usage(const char *program) {
    fprintf(stderr,
            "Usage:\n"
            "  %s plan --connection <path> [--capabilities <path>] [--catalog <path>]\n"
            "  %s preflight --connection <path> [--catalog <path>]\n"
            "  %s run --connection <path> [--capabilities <path>] [--catalog <path>] "
            "[--output <json>] [--output-markdown <md>] [--output-html <html>] "
            "[--allow-mutations]\n",
            program, program, program);
}

static bool file_is_readable(const char *path) {
    if (!path || path[0] == '\0') return false;
    FILE *file = fopen(path, "rb");
    if (!file) return false;
    fclose(file);
    return true;
}

static bool authentication_environment_is_available(const kmip_connection_config_t *connection) {
    if (connection->username_env[0] == '\0' && connection->password_env[0] == '\0') return true;
    if (connection->username_env[0] == '\0' || connection->password_env[0] == '\0') return false;
    const char *username = getenv(connection->username_env);
    const char *password = getenv(connection->password_env);
    return username && username[0] != '\0' && password;
}

static bool installed_artifact_is_current(void) {
    char commit_path[KMIP_MAX_NAME_LEN * 2];
    char provenance_path[KMIP_MAX_NAME_LEN * 2];
    if (snprintf(commit_path, sizeof(commit_path), "%s/library-commit", KMIPKIT_C_INSTALL_DIR) >=
            (int)sizeof(commit_path) ||
        snprintf(provenance_path, sizeof(provenance_path), "%s/provenance.json", KMIPKIT_C_INSTALL_DIR) >=
            (int)sizeof(provenance_path)) return false;
    FILE *commit_file = fopen(commit_path, "rb");
    if (!commit_file) return false;
    char commit[80] = {0};
    bool read_ok = fgets(commit, sizeof(commit), commit_file) != NULL;
    fclose(commit_file);
    commit[strcspn(commit, "\r\n")] = '\0';
    return read_ok && strcmp(commit, KMIPKIT_LIBRARY_COMMIT) == 0 &&
        file_is_readable(provenance_path);
}

static void set_report_identity(kmip_validation_summary_t *summary,
                                const kmip_connection_config_t *connection) {
    struct timespec now = {0};
    if (timespec_get(&now, TIME_UTC) != TIME_UTC) now.tv_sec = time(NULL);
    struct tm utc = {0};
#if defined(_WIN32)
    gmtime_s(&utc, &now.tv_sec);
#else
    struct tm *utc_pointer = gmtime(&now.tv_sec);
    if (utc_pointer) utc = *utc_pointer;
#endif
    char timestamp[24];
    size_t timestamp_length = strftime(timestamp, sizeof(timestamp), "%Y-%m-%dT%H:%M:%S", &utc);
    if (timestamp_length == 19 && sizeof(summary->tested_at) >= 25) {
        int milliseconds = (int)(now.tv_nsec / 1000000L);
        if (milliseconds < 0 || milliseconds > 999) milliseconds = 0;
        memcpy(summary->tested_at, timestamp, 19);
        summary->tested_at[19] = '.';
        summary->tested_at[20] = (char)('0' + milliseconds / 100);
        summary->tested_at[21] = (char)('0' + milliseconds / 10 % 10);
        summary->tested_at[22] = (char)('0' + milliseconds % 10);
        summary->tested_at[23] = 'Z';
        summary->tested_at[24] = '\0';
    } else {
        summary->tested_at[0] = '\0';
    }
    snprintf(summary->run_id, sizeof(summary->run_id), "c-%lld-%09ld",
             (long long)now.tv_sec, now.tv_nsec);
    snprintf(summary->server_id, sizeof(summary->server_id), "%s", connection->server_id);
    snprintf(summary->server_vendor, sizeof(summary->server_vendor), "%s", connection->server_vendor);
    snprintf(summary->server_product, sizeof(summary->server_product), "%s", connection->server_product);
    snprintf(summary->server_version, sizeof(summary->server_version), "%s", connection->server_version);
    snprintf(summary->server_configuration_fingerprint,
             sizeof(summary->server_configuration_fingerprint), "%s",
             connection->server_configuration_fingerprint);
    snprintf(summary->protocol_version, sizeof(summary->protocol_version), "%s",
             connection->protocol_version);
    snprintf(summary->transport, sizeof(summary->transport), "%s", connection->transport);
    snprintf(summary->encoding, sizeof(summary->encoding), "%s", connection->encoding);
    snprintf(summary->api_level, sizeof(summary->api_level), "%s", connection->api_level);
    snprintf(summary->library_commit, sizeof(summary->library_commit), "%s", KMIPKIT_LIBRARY_COMMIT);
    snprintf(summary->artifact_sha256, sizeof(summary->artifact_sha256), "%s",
             KMIPKIT_C_ARTIFACT_SHA256);
    snprintf(summary->catalog_sha256, sizeof(summary->catalog_sha256), "%s",
             KMIPKIT_C_CATALOG_SHA256);
}

int main(int argc, char **argv) {
    if (argc < 2) {
        print_usage(argv[0]);
        return 1;
    }
    const char *command = argv[1];
    const char *connection_path = NULL;
    const char *capabilities_path = NULL;
    const char *catalog_path = "interop/shared/catalog/kmipkit-1.0-test-catalog.json";
    const char *json_output_path = "validation-results.json";
    const char *markdown_output_path = NULL;
    const char *html_output_path = NULL;
    bool allow_mutations = false;

    for (int index = 2; index < argc; ++index) {
        if (strcmp(argv[index], "--connection") == 0 && index + 1 < argc) {
            connection_path = argv[++index];
        } else if ((strcmp(argv[index], "--capabilities") == 0 ||
                    strcmp(argv[index], "--profile") == 0) && index + 1 < argc) {
            capabilities_path = argv[++index];
        } else if (strcmp(argv[index], "--catalog") == 0 && index + 1 < argc) {
            catalog_path = argv[++index];
        } else if (strcmp(argv[index], "--output") == 0 && index + 1 < argc) {
            json_output_path = argv[++index];
        } else if (strcmp(argv[index], "--output-markdown") == 0 && index + 1 < argc) {
            markdown_output_path = argv[++index];
        } else if (strcmp(argv[index], "--output-html") == 0 && index + 1 < argc) {
            html_output_path = argv[++index];
        } else if (strcmp(argv[index], "--allow-mutations") == 0) {
            allow_mutations = true;
        } else {
            fprintf(stderr, "Error: unrecognized or incomplete option.\n");
            print_usage(argv[0]);
            return 1;
        }
    }
    if (!connection_path) {
        fprintf(stderr, "Error: --connection is required.\n");
        return 1;
    }

    kmip_connection_config_t connection;
    kmip_test_case_t cases[KMIP_MAX_CASES];
    size_t case_count = 0;
    if (!kmip_load_connection(connection_path, &connection)) {
        fprintf(stderr, "Error: invalid connection configuration.\n");
        return 1;
    }
    if (!kmip_load_catalog(catalog_path, cases, &case_count)) {
        fprintf(stderr, "Error: invalid or unsupported test catalog.\n");
        return 1;
    }

    if (strcmp(command, "plan") == 0) {
        if (capabilities_path) {
            kmip_server_capabilities_t profile;
            if (!kmip_load_capabilities(capabilities_path, &profile)) {
                fprintf(stderr, "Error: invalid capability profile.\n");
                return 1;
            }
        }
        size_t applicable = 0;
        size_t local_model = 0;
        size_t server_exchange = 0;
        size_t mutations = 0;
        size_t incoming = 0;
        unsigned int selected_version = strcmp(connection.protocol_version, "2.0") == 0
            ? KMIP_VERSION_MASK_2_0 : KMIP_VERSION_MASK_2_1;
        for (size_t index = 0; index < case_count; ++index) {
            const kmip_test_case_t *test_case = &cases[index];
            if ((test_case->protocol_version_mask & selected_version) == 0) continue;
            ++applicable;
            if (strcmp(test_case->mode, "local_model") == 0) ++local_model;
            else ++server_exchange;
            if (strcmp(test_case->mutation, "mutating") == 0) ++mutations;
            if (strcmp(test_case->workflow_type, "server_to_client") == 0) ++incoming;
        }
        printf("server_id=%s protocol_version=%s catalog_cases=%zu applicable=%zu "
               "server_exchange=%zu local_model=%zu mutations=%zu incoming=%zu\n",
               connection.server_id, connection.protocol_version, case_count, applicable,
               server_exchange, local_model, mutations, incoming);
        printf("capability profiles are hypotheses and do not suppress probes\n");
        printf("public C routes: typed-protocol and local generic TTLV; "
               "high-level and outbound generic TTLV are unavailable\n");
        return 0;
    }

    if (strcmp(command, "preflight") == 0) {
        if (!connection.dedicated_test_environment_confirmed) {
            fprintf(stderr, "Preflight: dedicated test environment confirmation is required.\n");
            return 2;
        }
        if (strcmp(connection.api_level, "typed-protocol") == 0 &&
            (!file_is_readable(connection.ca_cert_path) ||
             !file_is_readable(connection.client_cert_path) ||
             !file_is_readable(connection.client_key_path))) {
            fprintf(stderr, "Preflight: a TLS CA, client certificate or client key is unavailable.\n");
            return 2;
        }
        if (strcmp(connection.api_level, "typed-protocol") == 0 &&
            !authentication_environment_is_available(&connection)) {
            fprintf(stderr, "Preflight: configured Username/Password environment variables are unavailable.\n");
            return 2;
        }
        if (!installed_artifact_is_current()) {
            fprintf(stderr, "Preflight: installed KMIPKit C artifact provenance does not match this build.\n");
            return 2;
        }
        printf("preflight=ready server_id=%s; no DNS or network exchange was attempted\n",
               connection.server_id);
        return 0;
    }

    if (strcmp(command, "run") == 0) {
        kmip_server_capabilities_t capabilities = {0};
        kmip_test_case_t *applicable = calloc(KMIP_MAX_CASES, sizeof(*applicable));
        if (!applicable) {
            fprintf(stderr, "Run refused: could not allocate the bounded catalog view.\n");
            return 2;
        }
        strncpy(capabilities.server_id, connection.server_id,
                sizeof(capabilities.server_id) - 1);
        if (capabilities_path && !kmip_load_capabilities(capabilities_path, &capabilities)) {
            fprintf(stderr, "Run refused: capability profile is malformed.\n");
            free(applicable);
            return 2;
        }
        unsigned int selected_version = strcmp(connection.protocol_version, "2.0") == 0
            ? KMIP_VERSION_MASK_2_0 : KMIP_VERSION_MASK_2_1;
        size_t applicable_count = 0;
        for (size_t index = 0; index < case_count; ++index) {
            if ((cases[index].protocol_version_mask & selected_version) == 0) continue;
            applicable[applicable_count++] = cases[index];
        }
        if (!connection.dedicated_test_environment_confirmed) {
            fprintf(stderr, "Run refused: dedicated test environment confirmation is required.\n");
            free(applicable);
            return 2;
        }
        if (!connection.probe_unsupported) {
            fprintf(stderr, "Run refused: full KMIP catalog coverage requires probing unannounced capabilities.\n");
            free(applicable);
            return 2;
        }
        if (!installed_artifact_is_current()) {
            fprintf(stderr, "Run refused: installed KMIPKit C artifact provenance does not match this build.\n");
            free(applicable);
            return 2;
        }
        kmip_live_client_t live_client = {0};
        char error[KMIP_MAX_NAME_LEN] = {0};
        if (!kmip_live_client_open(&connection, &live_client, error, sizeof(error))) {
            fprintf(stderr, "Run refused: %s.\n", error[0] ? error : "live C client setup failed");
            free(applicable);
            return 2;
        }
        live_client.allow_mutations = allow_mutations && connection.allow_mutations;
        kmip_driver_vtable_t driver = {
            .execute_case = kmip_live_execute_case,
            .user_data = &live_client
        };
        kmip_validation_summary_t summary;
        kmip_run_suite(&capabilities, applicable, applicable_count, &driver,
                       connection.probe_unsupported, &summary);
        set_report_identity(&summary, &connection);
        kmip_render_ansi(&summary);
        bool reports_written = kmip_render_json(&summary, json_output_path);
        if (markdown_output_path)
            reports_written = kmip_render_markdown(&summary, markdown_output_path) && reports_written;
        if (html_output_path)
            reports_written = kmip_render_html(&summary, html_output_path) && reports_written;
        kmip_live_client_close(&live_client);
        free(applicable);
        if (!reports_written) {
            fprintf(stderr, "Run completed but one or more requested reports could not be written.\n");
            return 4;
        }
        return summary.failed == 0 ? 0 : 3;
    }

    print_usage(argv[0]);
    return 1;
}
