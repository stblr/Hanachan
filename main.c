#include "dump.h"
#include "player.h"

#include <immintrin.h>
#include <stdio.h>

static void replay(struct rkg rkg, struct dump dump) {
        struct bsp bsp;
        bsp_get(&bsp);

        struct player player;
        player_init(&player, rkg, bsp);

        u32 frame_count = rkg.frame_count + 172;
        if (frame_count > dump.frame_count) {
                frame_count = dump.frame_count;
        }

        bool desync = false;
        for (u32 frame = 0; frame < frame_count && !desync; frame++) {
                player_update(&player, frame);

                if (!vec3_equals(player.pos, dump.frames[frame].pos)) {
                        printf("POS %u\n", frame);
                        vec3_print(player.pos);
                        vec3_print(dump.frames[frame].pos);
                        desync = true;
                }
                if (!vec3_equals(player.speed0, dump.frames[frame].speed0)) {
                        printf("SPEED0 %u\n", frame);
                        vec3_print(player.speed0);
                        vec3_print(dump.frames[frame].speed0);
                        desync = true;
                }
                if (!vec3_equals(player.speed, dump.frames[frame].speed)) {
                        printf("SPEED %u\n", frame);
                        vec3_print(player.speed);
                        vec3_print(dump.frames[frame].speed);
                        desync = true;
                }
                if (!vec4_equals(player.rot, dump.frames[frame].rot)) {
                        printf("ROT %u\n", frame);
                        vec4_print(player.rot);
                        vec4_print(dump.frames[frame].rot);
                        desync = true;
                }
        }
}

int main(int argc, char **argv) {
        _MM_SET_FLUSH_ZERO_MODE(_MM_FLUSH_ZERO_ON);

        int ret = 1;

        if (argc != 3) {
                printf("Usage: hanachan <ghost.rkg> <dump.bin>\n");
                return ret;
        }

        struct rkg rkg = { 0 };
        struct dump dump = { 0 };

        if (!rkg_load(&rkg, argv[1])) {
                goto cleanup;
        }

        if (!dump_load(&dump, argv[2])) {
                goto cleanup;
        }

        replay(rkg, dump);

        ret = 0;

cleanup:
        free(dump.frames);
        free(rkg.inputs);

        return ret;
}
