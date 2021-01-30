#include "dump.h"

#include <stdlib.h>

bool dump_load(struct dump *dump, const char *path) {
        u32 size;
        u8 *data;
        if (!read_file(path, &size, &data)) {
                return false;
        }

        dump->frame_count = size / (13 * sizeof(u32));
        dump->frames = alloc(dump->frame_count, sizeof(struct dump_frame));

        u8 *ptr = data;
        for (u32 i = 0; i < dump->frame_count; i++) {
                dump->frames[i].pos = (struct vec3) {
                        get_f32(ptr + 0x0),
                        get_f32(ptr + 0x4),
                        get_f32(ptr + 0x8),
                };
                ptr += 3 * sizeof(u32);

                dump->frames[i].speed = (struct vec3) {
                        get_f32(ptr + 0x0),
                        get_f32(ptr + 0x4),
                        get_f32(ptr + 0x8),
                };
                ptr += 3 * sizeof(u32);

                dump->frames[i].speed0 = (struct vec3) {
                        get_f32(ptr + 0x0),
                        get_f32(ptr + 0x4),
                        get_f32(ptr + 0x8),
                };
                ptr += 3 * sizeof(u32);

                dump->frames[i].rot = (struct vec4) {
                        get_f32(ptr + 0x0),
                        get_f32(ptr + 0x4),
                        get_f32(ptr + 0x8),
                        get_f32(ptr + 0xc),
                };
                ptr += 4 * sizeof(u32);
        }

        free(data);

        return true;
}
