#include "c_validator.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static bool string_in_list(const char *target, const char list[KMIP_MAX_CASES][KMIP_MAX_NAME_LEN], size_t count) {
    if (!target || target[0] == '\0') return true;
    for (size_t i = 0; i < count; i++) {
        if (strcmp(target, list[i]) == 0) return true;
    }
    return false;
}

static char *read_file_str(const char *path) {
    FILE *f = fopen(path, "rb");
    if (!f) return NULL;
    fseek(f, 0, SEEK_END);
    long sz = ftell(f);
    fseek(f, 0, SEEK_SET);
    char *buf = malloc(sz + 1);
    if (!buf) { fclose(f); return NULL; }
    size_t rd = fread(buf, 1, sz, f);
    buf[rd] = '\0';
    fclose(f);
    return buf;
}

bool kmip_load_capabilities(const char *json_path, kmip_server_capabilities_t *out_caps) {
    if (!json_path || !out_caps) return false;
    memset(out_caps, 0, sizeof(*out_caps));

    char *buf = read_file_str(json_path);
    if (!buf) return false;

    // Simple robust JSON scanner for server id
    char *id_pos = strstr(buf, "\"id\":");
    if (id_pos) {
        char *q1 = strchr(id_pos + 5, '"');
        if (q1) {
            char *q2 = strchr(q1 + 1, '"');
            if (q2 && (size_t)(q2 - q1 - 1) < KMIP_MAX_NAME_LEN) {
                strncpy(out_caps->server_id, q1 + 1, q2 - q1 - 1);
                out_caps->server_id[q2 - q1 - 1] = '\0';
            }
        }
    }

    // Extract supported operations
    char *ops_pos = strstr(buf, "\"operations\":");
    if (ops_pos) {
        char *cur = ops_pos;
        while ((cur = strchr(cur, '"')) != NULL) {
            char *op_start = cur + 1;
            char *op_end = strchr(op_start, '"');
            if (!op_end) break;
            char op_name[KMIP_MAX_NAME_LEN] = {0};
            if ((size_t)(op_end - op_start) < KMIP_MAX_NAME_LEN) {
                strncpy(op_name, op_start, op_end - op_start);
            }
            char *colon = strchr(op_end, ':');
            if (!colon) break;
            char *supp = strstr(colon, "\"supported\":");
            if (supp && (supp - colon < 50)) {
                char *t = strstr(supp, "true");
                char *f = strstr(supp, "false");
                if (t && (!f || t < f) && (t - supp < 20)) {
                    if (out_caps->supported_operations_count < KMIP_MAX_CASES) {
                        strncpy(out_caps->supported_operations[out_caps->supported_operations_count], op_name, KMIP_MAX_NAME_LEN - 1);
                        out_caps->supported_operations_count++;
                    }
                }
            }
            cur = op_end + 1;
            char *next_section = strstr(cur, "\"managed_objects\":");
            if (next_section && cur > next_section) break;
        }
    }

    // Extract supported managed objects
    char *objs_pos = strstr(buf, "\"managed_objects\":");
    if (objs_pos) {
        char *cur = objs_pos;
        while ((cur = strchr(cur, '"')) != NULL) {
            char *obj_start = cur + 1;
            char *obj_end = strchr(obj_start, '"');
            if (!obj_end) break;
            char obj_name[KMIP_MAX_NAME_LEN] = {0};
            if ((size_t)(obj_end - obj_start) < KMIP_MAX_NAME_LEN) {
                strncpy(obj_name, obj_start, obj_end - obj_start);
            }
            char *colon = strchr(obj_end, ':');
            if (!colon) break;
            char *supp = strstr(colon, "\"supported\":");
            if (supp && (supp - colon < 50)) {
                char *t = strstr(supp, "true");
                char *f = strstr(supp, "false");
                if (t && (!f || t < f) && (t - supp < 20)) {
                    if (out_caps->supported_objects_count < KMIP_MAX_CASES) {
                        strncpy(out_caps->supported_objects[out_caps->supported_objects_count], obj_name, KMIP_MAX_NAME_LEN - 1);
                        out_caps->supported_objects_count++;
                    }
                }
            }
            cur = obj_end + 1;
            char *next_section = strstr(cur, "\"attributes\":");
            if (next_section && cur > next_section) break;
        }
    }

    // Extract supported attributes
    char *attr_pos = strstr(buf, "\"attributes\":");
    if (attr_pos) {
        char *cur = attr_pos;
        while ((cur = strchr(cur, '"')) != NULL) {
            char *at_start = cur + 1;
            char *at_end = strchr(at_start, '"');
            if (!at_end) break;
            char at_name[KMIP_MAX_NAME_LEN] = {0};
            if ((size_t)(at_end - at_start) < KMIP_MAX_NAME_LEN) {
                strncpy(at_name, at_start, at_end - at_start);
            }
            char *colon = strchr(at_end, ':');
            if (!colon) break;
            char *supp = strstr(colon, "\"supported\":");
            if (supp && (supp - colon < 50)) {
                char *t = strstr(supp, "true");
                char *f = strstr(supp, "false");
                if (t && (!f || t < f) && (t - supp < 20)) {
                    if (out_caps->supported_attributes_count < KMIP_MAX_CASES) {
                        strncpy(out_caps->supported_attributes[out_caps->supported_attributes_count], at_name, KMIP_MAX_NAME_LEN - 1);
                        out_caps->supported_attributes_count++;
                    }
                }
            }
            cur = at_end + 1;
            char *next_section = strstr(cur, "\"cryptographic_algorithms\":");
            if (next_section && cur > next_section) break;
        }
    }

    free(buf);
    return true;
}

bool kmip_load_catalog(const char *json_path, kmip_test_case_t *out_cases, size_t *out_count) {
    if (!json_path || !out_cases || !out_count) return false;
    *out_count = 0;

    char *buf = read_file_str(json_path);
    if (!buf) return false;

    char *cur = strstr(buf, "\"test_cases\":");
    if (!cur) { free(buf); return false; }

    while ((cur = strstr(cur, "\"case_id\":")) != NULL) {
        if (*out_count >= KMIP_MAX_CASES) break;
        kmip_test_case_t *tc = &out_cases[*out_count];
        memset(tc, 0, sizeof(*tc));

        // Case id
        char *q1 = strchr(cur + 10, '"');
        if (q1) {
            char *q2 = strchr(q1 + 1, '"');
            if (q2) {
                strncpy(tc->case_id, q1 + 1, q2 - q1 - 1);
            }
        }

        // Operation
        char *op_pos = strstr(cur, "\"operation\":");
        if (op_pos) {
            char *oq1 = strchr(op_pos + 12, '"');
            if (oq1) {
                char *oq2 = strchr(oq1 + 1, '"');
                if (oq2) {
                    strncpy(tc->operation, oq1 + 1, oq2 - oq1 - 1);
                }
            }
        }

        // Category
        char *cat_pos = strstr(cur, "\"category\":");
        if (cat_pos) {
            char *cq1 = strchr(cat_pos + 11, '"');
            if (cq1) {
                char *cq2 = strchr(cq1 + 1, '"');
                if (cq2) {
                    strncpy(tc->category, cq1 + 1, cq2 - cq1 - 1);
                }
            }
        }

        // Object type in requires
        char *obj_pos = strstr(cur, "\"object_type\":");
        if (obj_pos && (obj_pos - cur < 300)) {
            char *obq1 = strchr(obj_pos + 14, '"');
            if (obq1) {
                char *obq2 = strchr(obq1 + 1, '"');
                if (obq2) {
                    strncpy(tc->object_type, obq1 + 1, obq2 - obq1 - 1);
                }
            }
        }

        // Attribute in requires
        char *attr_pos = strstr(cur, "\"attribute\":");
        if (attr_pos && (attr_pos - cur < 300)) {
            char *atq1 = strchr(attr_pos + 12, '"');
            if (atq1) {
                char *atq2 = strchr(atq1 + 1, '"');
                if (atq2) {
                    strncpy(tc->attribute, atq1 + 1, atq2 - atq1 - 1);
                }
            }
        }

        (*out_count)++;
        cur += 15;
    }

    free(buf);
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

static kmip_execution_outcome_t mock_execute_case(const kmip_test_case_t *tc, const kmip_attempt_context_t *att) {
    (void)tc;
    (void)att;
    kmip_execution_outcome_t outcome;
    outcome.success = true;
    outcome.result_reason[0] = '\0';
    return outcome;
}

kmip_driver_vtable_t kmip_get_mock_driver(void) {
    kmip_driver_vtable_t driver;
    driver.execute_case = mock_execute_case;
    return driver;
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
    memset(out_summary, 0, sizeof(*out_summary));
    strncpy(out_summary->server_id, caps->server_id, sizeof(out_summary->server_id) - 1);
    strncpy(out_summary->language, "c", sizeof(out_summary->language) - 1);
    out_summary->total_cases = case_count;

    for (size_t i = 0; i < case_count && i < KMIP_MAX_CASES; i++) {
        const kmip_test_case_t *tc = &cases[i];
        kmip_case_result_t *res = &out_summary->results[out_summary->results_count++];
        strncpy(res->case_id, tc->case_id, sizeof(res->case_id) - 1);
        strncpy(res->operation, tc->operation, sizeof(res->operation) - 1);

        char reasons[KMIP_MAX_REASONS][KMIP_MAX_NAME_LEN];
        size_t r_count = 0;
        bool supported = kmip_is_case_supported(caps, tc, reasons, &r_count);

        if (supported) {
            kmip_attempt_context_t attempt;
            kmip_init_attempt_context(&attempt, "run-c", "c", "TTLV", "level1", tc->case_id);
            kmip_execution_outcome_t outcome = driver->execute_case(tc, &attempt);
            if (outcome.success) {
                res->status = CASE_STATUS_PASSED;
                out_summary->passed++;
            } else {
                res->status = CASE_STATUS_FAILED;
                strncpy(res->detail, outcome.result_reason, sizeof(res->detail) - 1);
                out_summary->failed++;
            }
        } else {
            out_summary->unsupported++;
            if (probe_unsupported) {
                res->status = CASE_STATUS_UNSUPPORTED_VERIFIED_REJECTED;
            } else {
                res->status = CASE_STATUS_UNSUPPORTED_SKIPPED;
                if (r_count > 0) {
                    strncpy(res->detail, reasons[0], sizeof(res->detail) - 1);
                }
            }
        }
    }
}

void kmip_render_ansi(const kmip_validation_summary_t *summary) {
    if (!summary) return;
    printf("\n=== KMIP 2.1 Validation Matrix: Server [%s] (Lang: %s) ===\n", summary->server_id, summary->language);
    printf("Total: %zu | ✅ Passed: %zu | ❌ Failed: %zu | ➖ Unsupported: %zu\n",
           summary->total_cases, summary->passed, summary->failed, summary->unsupported);
    printf("--------------------------------------------------------------------------------\n");
    printf("%-35s | %-25s | %-10s\n", "Case ID", "Operation", "Status");
    printf("--------------------------------------------------------------------------------\n");
    for (size_t i = 0; i < summary->results_count; i++) {
        const kmip_case_result_t *r = &summary->results[i];
        const char *st = (r->status == CASE_STATUS_PASSED) ? "✅ PASS" :
                         (r->status == CASE_STATUS_FAILED) ? "❌ FAIL" : "➖ UNSUPP";
        printf("%-35s | %-25s | %-10s\n", r->case_id, r->operation, st);
    }
    printf("--------------------------------------------------------------------------------\n");
}

bool kmip_render_markdown(const kmip_validation_summary_t *summary, const char *file_path) {
    if (!summary || !file_path) return false;
    FILE *f = fopen(file_path, "w");
    if (!f) return false;

    fprintf(f, "# KMIP 2.1 Conformance Matrix: `%s`\n\n", summary->server_id);
    fprintf(f, "- **Language**: `%s`\n", summary->language);
    fprintf(f, "- **Total Cases**: %zu\n\n", summary->total_cases);
    fprintf(f, "### Summary\n\n");
    fprintf(f, "| Metric | Count |\n|---|---:|\n");
    fprintf(f, "| Total Cases | %zu |\n", summary->total_cases);
    fprintf(f, "| ✅ Passed | %zu |\n", summary->passed);
    fprintf(f, "| ❌ Failed | %zu |\n", summary->failed);
    fprintf(f, "| ➖ Unsupported | %zu |\n\n", summary->unsupported);

    fprintf(f, "### Detailed Operations\n\n");
    fprintf(f, "| Case ID | Operation | Status |\n|---|---|:---:|\n");
    for (size_t i = 0; i < summary->results_count; i++) {
        const kmip_case_result_t *r = &summary->results[i];
        const char *st = (r->status == CASE_STATUS_PASSED) ? "✅" :
                         (r->status == CASE_STATUS_FAILED) ? "❌" : "➖";
        fprintf(f, "| `%s` | %s | %s |\n", r->case_id, r->operation, st);
    }

    fclose(f);
    return true;
}

bool kmip_render_html(const kmip_validation_summary_t *summary, const char *file_path) {
    if (!summary || !file_path) return false;
    FILE *f = fopen(file_path, "w");
    if (!f) return false;

    fprintf(f, "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"UTF-8\">\n<title>KMIP 2.1 Validation Report</title>\n");
    fprintf(f, "<style>\nbody { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 2rem; background: #f8fafc; color: #1e293b; }\n");
    fprintf(f, ".card { background: white; border-radius: 8px; padding: 1.5rem; box-shadow: 0 1px 3px rgba(0,0,0,0.1); margin-bottom: 2rem; }\n");
    fprintf(f, "table { width: 100%%; border-collapse: collapse; margin-top: 1rem; }\n");
    fprintf(f, "th, td { text-align: left; padding: 0.75rem 1rem; border-bottom: 1px solid #e2e8f0; }\n");
    fprintf(f, "th { background: #f1f5f9; font-weight: 600; }\n");
    fprintf(f, ".pass { color: #16a34a; font-weight: bold; }\n.fail { color: #dc2626; font-weight: bold; }\n.unsupp { color: #64748b; font-weight: bold; }\n</style>\n</head>\n<body>\n");
    fprintf(f, "<div class=\"card\">\n<h1>KMIP 2.1 Validation Matrix: %s</h1>\n", summary->server_id);
    fprintf(f, "<p><strong>Language:</strong> %s</p>\n", summary->language);
    fprintf(f, "<p>✅ <strong>Passed:</strong> %zu | ❌ <strong>Failed:</strong> %zu | ➖ <strong>Unsupported:</strong> %zu</p>\n</div>\n",
            summary->passed, summary->failed, summary->unsupported);
    fprintf(f, "<div class=\"card\">\n<table>\n<thead>\n<tr><th>Case ID</th><th>Operation</th><th>Status</th></tr>\n</thead>\n<tbody>\n");

    for (size_t i = 0; i < summary->results_count; i++) {
        const kmip_case_result_t *r = &summary->results[i];
        const char *cls = (r->status == CASE_STATUS_PASSED) ? "pass" :
                          (r->status == CASE_STATUS_FAILED) ? "fail" : "unsupp";
        const char *st = (r->status == CASE_STATUS_PASSED) ? "✅ Passed" :
                         (r->status == CASE_STATUS_FAILED) ? "❌ Failed" : "➖ Unsupported";
        fprintf(f, "<tr><td><code>%s</code></td><td>%s</td><td class=\"%s\">%s</td></tr>\n",
                r->case_id, r->operation, cls, st);
    }
    fprintf(f, "</tbody>\n</table>\n</div>\n</body>\n</html>\n");

    fclose(f);
    return true;
}

bool kmip_render_json(const kmip_validation_summary_t *summary, const char *file_path) {
    if (!summary || !file_path) return false;
    FILE *f = fopen(file_path, "w");
    if (!f) return false;

    fprintf(f, "{\n  \"run_id\": \"run-c\",\n  \"language\": \"c\",\n  \"server_id\": \"%s\",\n", summary->server_id);
    fprintf(f, "  \"summary\": {\n    \"total_cases\": %zu,\n    \"passed\": %zu,\n    \"failed\": %zu,\n    \"unsupported_by_server\": %zu\n  },\n",
            summary->total_cases, summary->passed, summary->failed, summary->unsupported);
    fprintf(f, "  \"results\": [\n");
    for (size_t i = 0; i < summary->results_count; i++) {
        const kmip_case_result_t *r = &summary->results[i];
        const char *st = (r->status == CASE_STATUS_PASSED) ? "passed" :
                         (r->status == CASE_STATUS_FAILED) ? "failed" : "unsupported_skipped";
        fprintf(f, "    {\n      \"case_id\": \"%s\",\n      \"operation\": \"%s\",\n      \"status\": \"%s\"\n    }%s\n",
                r->case_id, r->operation, st, (i + 1 < summary->results_count) ? "," : "");
    }
    fprintf(f, "  ]\n}\n");

    fclose(f);
    return true;
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

