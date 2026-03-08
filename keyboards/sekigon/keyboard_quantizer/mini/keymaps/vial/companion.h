// Copyright 2024 yacchi
/* SPDX-License-Identifier: GPL-2.0-or-later */

#pragma once

#include <stdint.h>
#include <stdbool.h>

// Companion App Protocol
// Uses command IDs 0x20-0x2F (unused range between VIA and Vial)

#define COMPANION_PROTOCOL_VERSION 0x0001

// Command IDs
enum companion_command_id {
    COMPANION_CMD_GET_STATUS     = 0x20,
    COMPANION_CMD_SET_LAYER      = 0x21,
    COMPANION_CMD_SET_OS_OVERRIDE = 0x22,
    COMPANION_CMD_PING           = 0x23,
    COMPANION_CMD_GET_VERSION    = 0x24,
    COMPANION_CMD_RANGE_END      = 0x2F,
};

// GET_STATUS response layout (data[1..])
// [1]    = current default layer
// [2-5]  = layer_state (uint32_t, little-endian)
// [6]    = os_override setting (0=disable, 1=us_on_jp, 2=jp_on_us)
// [7]    = DYNAMIC_KEYMAP_LAYER_COUNT

// SET_LAYER request
// [1] = target default layer (0-based)

// SET_OS_OVERRIDE request
// [1] = override type (0=disable, 1=us_on_jp, 2=jp_on_us)

// PING request/response
// [1-4] = echo payload (copied back as-is)

// GET_VERSION response
// [1-2] = protocol version (big-endian)
// [3-10] = VIAL_KEYBOARD_UID

bool companion_raw_hid_receive(uint8_t *data, uint8_t length);
