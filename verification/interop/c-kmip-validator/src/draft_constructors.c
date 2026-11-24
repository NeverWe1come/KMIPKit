#include "kmipkit.h"

#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef int32_t (*constructor_t)(kmipkit_ttlv_structure_t *, kmipkit_client_batch_item_t **);

struct draft_route {
    const char *symbol;
    constructor_t construct;
    uint32_t operation;
};

#define ROUTE(name, operation) { #name, name, operation }

static const struct draft_route ROUTES[] = {
    ROUTE(kmipkit_client_batch_item_create_group, 0x3c),
    ROUTE(kmipkit_client_batch_item_obliterate, 0x3d),
    ROUTE(kmipkit_client_batch_item_create_user, 0x3e),
    ROUTE(kmipkit_client_batch_item_create_credential, 0x3f),
    ROUTE(kmipkit_client_batch_item_deactivate, 0x40),
    ROUTE(kmipkit_client_batch_item_encapsulate, 0x41),
    ROUTE(kmipkit_client_batch_item_decapsulate, 0x42),
};

static int check_route(const struct draft_route *route) {
    kmipkit_ttlv_structure_t *payload = NULL;
    kmipkit_client_batch_item_t *item = NULL;
    uint32_t actual = 0;
    int32_t status = kmipkit_ttlv_structure_create(&payload);
    if (status == 0) {
        status = route->construct(payload, &item);
    }
    if (status == 0) {
        status = kmipkit_client_batch_item_operation(item, &actual);
    }
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(payload);
    return status == 0 && actual == route->operation;
}

static int check_tags(void) {
    uint32_t code;
    unsigned assigned;
    int input;
    while ((input = scanf("%" SCNx32 " %u", &code, &assigned)) == 2) {
        kmipkit_raw_tag_t *raw = NULL;
        kmipkit_tag_t *checked = NULL;
        uint32_t actual = 0;
        int raw_ok = kmipkit_ttlv_raw_tag_create(code, &raw) == 0 &&
                     kmipkit_ttlv_raw_tag_value(raw, &actual) == 0 && actual == code;
        int32_t status = raw_ok ? kmipkit_ttlv_raw_tag_try_checked(raw, &checked) : KMIPKIT_ERROR_INVALID_SCHEMA;
        int checked_ok = raw_ok && (assigned
            ? status == 0 && kmipkit_ttlv_tag_value(checked, &actual) == 0 && actual == code
            : status == KMIPKIT_ERROR_INVALID_SCHEMA && checked == NULL);
        kmipkit_tag_release(checked);
        kmipkit_raw_tag_release(raw);
        printf("%s %s\n", raw_ok ? "passed" : "failed", checked_ok ? "passed" : "failed");
    }
    return input == EOF && !ferror(stdin) ? EXIT_SUCCESS : EXIT_FAILURE;
}

static bool print_item(kmipkit_ttlv_item_view_t *view, bool first) {
    uint8_t type = 0;
    kmipkit_tag_t *tag = NULL;
    kmipkit_ttlv_value_view_t *value = NULL;
    kmipkit_ttlv_structure_view_t *structure = NULL;
    uint32_t code = 0;
    uint64_t count = 0;
    bool ok = kmipkit_ttlv_item_view_tag(view, &tag) == 0 &&
              kmipkit_ttlv_tag_value(tag, &code) == 0 &&
              kmipkit_ttlv_item_view_type(view, &type) == 0;
    if (ok && type == 1) {
        ok = kmipkit_ttlv_item_view_value(view, &value) == 0 &&
             kmipkit_ttlv_value_view_structure(value, &structure) == 0 &&
             kmipkit_ttlv_structure_view_item_count(structure, &count) == 0;
    }
    if (ok) {
        printf("%s[%" PRIu32 ",%u,%" PRIu64 "]", first ? "" : ",", code, (unsigned)type, count);
        for (uint64_t index = 0; ok && index < count; ++index) {
            kmipkit_ttlv_item_view_t *child = NULL;
            ok = kmipkit_ttlv_structure_view_item_at(structure, index, &child) == 0 && print_item(child, false);
            kmipkit_ttlv_item_view_release(child);
        }
    }
    kmipkit_ttlv_structure_view_release(structure);
    kmipkit_ttlv_value_view_release(value);
    kmipkit_tag_release(tag);
    return ok;
}

static int hex_digit(char character) {
    if (character >= '0' && character <= '9') return character - '0';
    if (character >= 'a' && character <= 'f') return character - 'a' + 10;
    return -1;
}

static bool inspect_line(char *line, uint8_t *wire) {
    size_t length = strcspn(line, "\n");
    if (line[length] != '\n' || length % 2 != 0) return false;
    for (size_t index = 0; index < length; index += 2) {
        int high = hex_digit(line[index]);
        int low = hex_digit(line[index + 1]);
        if (high < 0 || low < 0) return false;
        wire[index / 2] = (uint8_t)(high * 16 + low);
    }
    kmipkit_ttlv_item_t *item = NULL;
    kmipkit_ttlv_decode_error_v1_t error = {
        .struct_size = (uint32_t)sizeof(error), .abi_version = 1,
    };
    if (kmipkit_ttlv_decode(wire, length / 2, &error, &item) != 0) {
        puts("null");
        return true;
    }
    kmipkit_ttlv_item_view_t *view = NULL;
    fputc('[', stdout);
    bool ok = kmipkit_ttlv_item_view_from_item(item, &view) == 0 && print_item(view, true);
    puts("]");
    kmipkit_ttlv_item_view_release(view);
    kmipkit_ttlv_item_release(item);
    return ok;
}

static int inspect_ttlv(void) {
    enum { MAX_HEX_LINE = 2 * 1024 * 1024 + 2 };
    char *line = malloc(MAX_HEX_LINE);
    uint8_t *wire = malloc(MAX_HEX_LINE / 2);
    bool ok = line != NULL && wire != NULL;
    if (ok) {
        printf("{\"library_commit\":\"%s\",\"artifact_sha256\":\"%s\"}\n",
               KMIPKIT_LIBRARY_COMMIT, KMIPKIT_C_ARTIFACT_SHA256);
    }
    while (ok && fgets(line, MAX_HEX_LINE, stdin)) {
        ok = inspect_line(line, wire);
    }
    free(wire);
    free(line);
    return ok && !ferror(stdin) ? EXIT_SUCCESS : EXIT_FAILURE;
}

int main(int argc, char **argv) {
    int passed = 1;
    if (argc == 2 && strcmp(argv[1], "--inspect-oasis-ttlv") == 0) {
        return inspect_ttlv();
    }
    puts(KMIPKIT_LIBRARY_COMMIT);
    puts(KMIPKIT_C_ARTIFACT_SHA256);
    if (argc == 2 && strcmp(argv[1], "--tags") == 0) {
        return check_tags();
    }
    if (argc != 1) {
        return EXIT_FAILURE;
    }
    for (size_t index = 0; index < sizeof ROUTES / sizeof ROUTES[0]; ++index) {
        int result = check_route(&ROUTES[index]);
        printf("%s %s\n", ROUTES[index].symbol, result ? "passed" : "failed");
        passed &= result;
    }
    return passed ? EXIT_SUCCESS : EXIT_FAILURE;
}
