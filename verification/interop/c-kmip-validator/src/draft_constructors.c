#include "kmipkit.h"

#include <inttypes.h>
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

int main(int argc, char **argv) {
    int passed = 1;
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
