#include "rkrd.h"

#include <stdlib.h>

static struct vec3 next_vec3(const u8 **data) {
        struct vec3 v;
        v.x = next_f32(data);
        v.y = next_f32(data);
        v.z = next_f32(data);
        return v;
}

static struct vec4 next_vec4(const u8 **data) {
        struct vec4 v;
        v.x = next_f32(data);
        v.y = next_f32(data);
        v.z = next_f32(data);
        v.w = next_f32(data);
        return v;
}

bool rkrd_load(struct rkrd *rkrd, const char *path) {
        u32 size;
        u8 *data_start;
        if (!read_file(path, &size, &data_start)) {
                return false;
        }
        const u8 *data = data_start;

        u32 fourcc = next_u32(&data);
        if (fourcc != pack_u32('R', 'K', 'R', 'D')) {
                return false;
        }

        u32 version = next_u32(&data);
        if (version != 0) {
                return false;
        }

        size -= 2 * sizeof(u32);

        rkrd->frame_count = size / (27 * sizeof(u32));
        rkrd->frames = alloc(rkrd->frame_count, sizeof(struct rkrd_frame));
        for (u32 i = 0; i < rkrd->frame_count; i++) {
                rkrd->frames[i].dir = next_vec3(&data);
                rkrd->frames[i].pos = next_vec3(&data);
                rkrd->frames[i].speed0 = next_vec3(&data);
                rkrd->frames[i].speed1_norm = next_f32(&data);
                rkrd->frames[i].speed = next_vec3(&data);
                rkrd->frames[i].rot_vec0 = next_vec3(&data);
                rkrd->frames[i].rot_vec2 = next_vec3(&data);
                rkrd->frames[i].rot = next_vec4(&data);
                rkrd->frames[i].rot2 = next_vec4(&data);
        }

        free(data_start);

        return true;
}
