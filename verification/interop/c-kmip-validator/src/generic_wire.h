#ifndef KMIPKIT_VALIDATOR_GENERIC_WIRE_H
#define KMIPKIT_VALIDATOR_GENERIC_WIRE_H

#include "kmipkit.h"

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

bool encode_generic_item(kmipkit_ttlv_item_view_t *item, uint8_t *wire,
                         size_t capacity, size_t *offset);

#endif
