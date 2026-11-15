#include "c_validator.h"
#include <stdio.h>
#include <string.h>

static void print_usage(const char *prog) {
    printf("Usage:\n");
    printf("  %s plan --capabilities <path> [--catalog <path>]\n", prog);
    printf("  %s run --capabilities <path> [--catalog <path>] [--markdown <path>] [--html <path>] [--output <path>] [--probe-unsupported]\n", prog);
}

int main(int argc, char **argv) {
    if (argc < 2) {
        print_usage(argv[0]);
        return 1;
    }

    const char *cmd = argv[1];
    const char *caps_path = NULL;
    const char *cat_path = "../shared/catalog/kmipkit-1.0-test-catalog.json";
    const char *md_path = NULL;
    const char *html_path = NULL;
    const char *out_path = NULL;
    bool probe_unsupported = false;

    for (int i = 2; i < argc; i++) {
        if ((strcmp(argv[i], "--capabilities") == 0 || strcmp(argv[i], "-c") == 0) && i + 1 < argc) {
            caps_path = argv[++i];
        } else if ((strcmp(argv[i], "--catalog") == 0 || strcmp(argv[i], "-t") == 0) && i + 1 < argc) {
            cat_path = argv[++i];
        } else if (strcmp(argv[i], "--markdown") == 0 && i + 1 < argc) {
            md_path = argv[++i];
        } else if (strcmp(argv[i], "--html") == 0 && i + 1 < argc) {
            html_path = argv[++i];
        } else if ((strcmp(argv[i], "--output") == 0 || strcmp(argv[i], "-o") == 0) && i + 1 < argc) {
            out_path = argv[++i];
        } else if (strcmp(argv[i], "--probe-unsupported") == 0) {
            probe_unsupported = true;
        }
    }

    if (!caps_path) {
        fprintf(stderr, "Error: --capabilities <path> is required.\n");
        return 1;
    }

    kmip_server_capabilities_t caps;
    if (!kmip_load_capabilities(caps_path, &caps)) {
        fprintf(stderr, "Error: Failed to load capabilities from %s\n", caps_path);
        return 1;
    }

    kmip_test_case_t cases[KMIP_MAX_CASES];
    size_t case_count = 0;
    if (!kmip_load_catalog(cat_path, cases, &case_count)) {
        fprintf(stderr, "Error: Failed to load test catalog from %s\n", cat_path);
        return 1;
    }

    if (strcmp(cmd, "plan") == 0) {
        size_t supported = 0;
        size_t unsupported = 0;
        for (size_t i = 0; i < case_count; i++) {
            char reasons[KMIP_MAX_REASONS][KMIP_MAX_NAME_LEN];
            size_t r_count = 0;
            if (kmip_is_case_supported(&caps, &cases[i], reasons, &r_count)) {
                supported++;
            } else {
                unsupported++;
            }
        }
        printf("Planning execution for server: [%s]\n", caps.server_id);
        printf("  Total cases: %zu\n", case_count);
        printf("  Supported (✅ / ❌): %zu\n", supported);
        printf("  Unsupported (➖): %zu\n", unsupported);
        return 0;
    } else if (strcmp(cmd, "run") == 0) {
        kmip_driver_vtable_t driver = kmip_get_mock_driver();
        kmip_validation_summary_t summary;
        kmip_run_suite(&caps, cases, case_count, &driver, probe_unsupported, &summary);

        kmip_render_ansi(&summary);

        if (md_path) {
            kmip_render_markdown(&summary, md_path);
            printf("Saved Markdown matrix: %s\n", md_path);
        }
        if (html_path) {
            kmip_render_html(&summary, html_path);
            printf("Saved HTML dashboard: %s\n", html_path);
        }
        if (out_path) {
            kmip_render_json(&summary, out_path);
            printf("Saved Results JSON: %s\n", out_path);
        }
        return 0;
    } else {
        print_usage(argv[0]);
        return 1;
    }
}
