// Copyright 2024 yacchi
/* SPDX-License-Identifier: GPL-2.0-or-later */

#include "companion.h"
#include "raw_hid.h"
#include "action_layer.h"
#include "eeconfig.h"
#include "dynamic_keymap.h"

#include "keymap.h"
#include "os_key_override.h"

#ifndef VIAL_KEYBOARD_UID
#    define VIAL_KEYBOARD_UID {0}
#endif

static const uint8_t vial_uid[] = VIAL_KEYBOARD_UID;

static void companion_handle_get_status(uint8_t *data) {
    extern user_config_t user_config;

    data[1] = get_highest_layer(default_layer_state);

    uint32_t ls = layer_state;
    data[2] = (ls >> 0)  & 0xFF;
    data[3] = (ls >> 8)  & 0xFF;
    data[4] = (ls >> 16) & 0xFF;
    data[5] = (ls >> 24) & 0xFF;

    data[6] = user_config.key_os_override;
    data[7] = dynamic_keymap_get_layer_count();
}

static void companion_handle_set_layer(uint8_t *data) {
    uint8_t target_layer = data[1];
    if (target_layer < dynamic_keymap_get_layer_count()) {
        default_layer_set(1U << target_layer);
        data[1] = target_layer; // echo back confirmed layer
    } else {
        data[0] = 0xFF; // error
    }
}

static void companion_handle_set_os_override(uint8_t *data) {
    extern user_config_t user_config;
    uint8_t override_type = data[1];

    switch (override_type) {
        case KEY_OS_OVERRIDE_DISABLE:
            remove_all_os_key_overrides();
            user_config.key_os_override = KEY_OS_OVERRIDE_DISABLE;
            eeconfig_update_user(user_config.raw);
            break;
        case US_KEY_JP_OS_OVERRIDE_DISABLE:
            register_us_key_on_jp_os_overrides();
            user_config.key_os_override = US_KEY_JP_OS_OVERRIDE_DISABLE;
            eeconfig_update_user(user_config.raw);
            break;
        case JP_KEY_US_OS_OVERRIDE_DISABLE:
            register_jp_key_on_us_os_overrides();
            user_config.key_os_override = JP_KEY_US_OS_OVERRIDE_DISABLE;
            eeconfig_update_user(user_config.raw);
            break;
        default:
            data[0] = 0xFF; // error
            return;
    }
    data[1] = override_type; // echo back confirmed
}

static void companion_handle_get_version(uint8_t *data) {
    data[1] = (COMPANION_PROTOCOL_VERSION >> 8) & 0xFF;
    data[2] = COMPANION_PROTOCOL_VERSION & 0xFF;
    for (uint8_t i = 0; i < sizeof(vial_uid) && i < 8; i++) {
        data[3 + i] = vial_uid[i];
    }
}

bool companion_raw_hid_receive(uint8_t *data, uint8_t length) {
    uint8_t cmd = data[0];

    if (cmd < COMPANION_CMD_GET_STATUS || cmd > COMPANION_CMD_RANGE_END) {
        return false; // not our command
    }

    switch (cmd) {
        case COMPANION_CMD_GET_STATUS:
            companion_handle_get_status(data);
            break;
        case COMPANION_CMD_SET_LAYER:
            companion_handle_set_layer(data);
            break;
        case COMPANION_CMD_SET_OS_OVERRIDE:
            companion_handle_set_os_override(data);
            break;
        case COMPANION_CMD_PING:
            // echo back as-is (data already contains payload)
            break;
        case COMPANION_CMD_GET_VERSION:
            companion_handle_get_version(data);
            break;
        default:
            data[0] = 0xFF;
            break;
    }

    raw_hid_send(data, length);
    return true;
}
