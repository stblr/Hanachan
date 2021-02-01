#include "rkg.h"

#include "yaz.h"

#include <stdio.h>
#include <stdlib.h>

static bool track_is_valid(u8 track) {
        if (track >= 0x20) {
                return false;
        }
        if (track != 0x18) {
                printf("Warning: only rMC3 is supported for now!\n");
        }
        return true;
}

static bool vehicle_is_valid(u8 vehicle) {
        if (vehicle >= 0x24) {
                return false;
        }
        if (vehicle != 0x17) {
                printf("Warning: only Flame Runner is supported for now!\n");
        }
        return true;
}

static bool character_is_valid(u8 character) {
        if (character >= 0x18) {
                // TODO: support Miis
                return false;
        }
        if (character != 0x16) {
                printf("Warning: only Funky Kong is supported for now!\n");
        }
        return true;
}

static bool rkg_parse_header(struct rkg *rkg, u32 size, u8 *data) {
        if (size < 0x88) {
                return false;
        }

        if (get_u32(data + 0x00) != pack_u32('R', 'K', 'G', 'D')) {
                return false;
        }

        rkg->minutes = get_u8(data + 0x04) >> 1;
        rkg->seconds = get_u16(data + 0x04) >> 2 & 0x7f;
        rkg->milliseconds = get_u16(data + 0x05) & 0x3ff;
        if (rkg->minutes > 59 || rkg->seconds > 59 || rkg->milliseconds > 999) {
                return false;
        }

        rkg->track = get_u8(data + 0x07) >> 2;
        if (!track_is_valid(rkg->track)) {
                return false;
        }

        rkg->vehicle = get_u8(data + 0x08) >> 2;
        rkg->character = get_u16(data + 0x08) >> 4 & 0x3f;
        if (!vehicle_is_valid(rkg->vehicle) || !character_is_valid(rkg->character)) {
                return false;
        }

        rkg->year = 2000 + (get_u16(data + 0x09) >> 5 & 0x7f);
        rkg->month = get_u8(data + 0x0a) >> 1 & 0xf;
        rkg->day = get_u16(data + 0x0a) >> 4 & 0x1f;
        // TODO: check if the date exists

        rkg->controller = get_u8(data + 0x0b) & 0xf;
        if (rkg->controller >= 4) {
                return false;
        }

        rkg->compressed = get_u8(data + 0x0c) >> 3 & 1;
        if (!rkg->compressed) {
                printf("Uncompressed data is unsupported for now!\n"); // TODO: support it!
                return false;
        }

        rkg->ghost_type = get_u16(data + 0x0c) >> 2 & 0x7f;

        rkg->automatic_drift = get_u8(data + 0x0d) >> 6 & 1;
        if (rkg->automatic_drift) {
                printf("Warning: only manual drift is supported for now!\n");
        }

        return true;
}

static bool rkg_parse_input(struct rkg *rkg, u32 size, u8 *data) {
        u16 button_input_count = get_u16(data + 0);
        u16 direction_input_count = get_u16(data + 2);
        u16 trick_input_count = get_u16(data + 4);
        u32 total_input_count = button_input_count + direction_input_count + trick_input_count;
        if (total_input_count * 2 != size - 8) {
                return false;
        }

        u8 *ptr = data + 8;
        u32 button_frame_count = 0;
        for (u16 i = 0; i < button_input_count; i++) {
                ptr++;
                button_frame_count += get_u8(ptr++);
        }

        u32 direction_frame_count = 0;
        for (u16 i = 0; i < direction_input_count; i++) {
                ptr++;
                direction_frame_count += get_u8(ptr++);
        }

        u32 trick_frame_count = 0;
        for (u16 i = 0; i < trick_input_count; i++) {
                trick_frame_count += get_u16(ptr) & 0xfff;
                ptr += 2;
        }

        rkg->frame_count = button_frame_count;
        if (direction_frame_count != rkg->frame_count || trick_frame_count != rkg->frame_count) {
                return false;
        }

        rkg->inputs = alloc(rkg->frame_count, sizeof(u16));

        ptr = data + 8;
        u16 *inputs = rkg->inputs;
        for (u16 i = 0; i < button_input_count; i++) {
                u8 state = get_u8(ptr++);
                u8 frame_count = get_u8(ptr++);
                for (u8 j = 0; j < frame_count; j++) {
                        *inputs++ = state & 0x1f;
                }
        }

        inputs = rkg->inputs;
        for (u16 i = 0; i < direction_input_count; i++) {
                u8 state = get_u8(ptr++);
                u8 frame_count = get_u8(ptr++);
                for (u8 j = 0; j < frame_count; j++) {
                        *inputs++ |= state << 8;
                }
        }

        inputs = rkg->inputs;
        for (u16 i = 0; i < trick_input_count; i++) {
                u8 state = get_u8(ptr) >> 4;
                u16 frame_count = get_u16(ptr) & 0xfff;
                ptr += 2;
                if (state & 0x8) { // TODO stronger validation
                        free(rkg->inputs);
                        return false;
                }
                for (u16 j = 0; j < frame_count; j++) {
                        *inputs++ |= state << 5;
                }
        }

        return true;
}

bool rkg_load(struct rkg *rkg, const char *path) {
        bool ret = false;

        u32 size;
        u8 *data;
        if (!read_file(path, &size, &data)) {
                return ret;
        }

        if (!rkg_parse_header(rkg, size, data)) {
                goto cleanup;
        }

        if (size < 0x88 + 0x04 + 0x04) {
                goto cleanup;
        }
        u32 src_size = get_u32(data + 0x88);
        //if (src_size != size - 0x88 - 0x04 - 0x04) {
        if (src_size > size - 0x88 - 0x04 - 0x04) { // TODO figure out exact calculation
                goto cleanup;
        }
        u32 dst_size;
        u8 *dst;
        if (!yaz_decompress(&dst_size, &dst, src_size, data + 0x88 + 0x04)) {
                goto cleanup;
        }

        if (!rkg_parse_input(rkg, dst_size, dst)) {
                free(dst);
                goto cleanup;
        }
        free(dst);

        ret = true;

cleanup:
        free(data);

        return ret;
}
