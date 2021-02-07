#include "rkrd.h"
#include "player.h"

#include <immintrin.h>
#include <stdio.h>

static void replay(struct rkg rkg, struct rkrd rkrd) {
        struct stats stats;
        stats_get(&stats);

        struct bsp bsp;
        bsp_get(&bsp);

        struct player player;
        player_init(&player, rkg, stats, bsp);

        u32 frame_count = rkg.frame_count + 172;
        if (frame_count > rkrd.frame_count) {
                frame_count = rkrd.frame_count;
        }

        bool desync = false;
        for (u32 frame = 0; frame < frame_count && !desync; frame++) {
                player_update(&player, frame);

                if (!vec3_equals(player.dir, rkrd.frames[frame].dir)) {
                        printf("DIR %u\n", frame);
                        vec3_print(player.dir);
                        vec3_print(rkrd.frames[frame].dir);
                        desync = true;
                }
                if (!vec3_equals(player.pos, rkrd.frames[frame].pos)) {
                        printf("POS %u\n", frame);
                        vec3_print(player.pos);
                        vec3_print(rkrd.frames[frame].pos);
                        desync = true;
                }
                if (!vec3_equals(player.speed0, rkrd.frames[frame].speed0)) {
                        printf("SPEED0 %u\n", frame);
                        vec3_print(player.speed0);
                        vec3_print(rkrd.frames[frame].speed0);
                        desync = true;
                }
                if (player.speed1_norm != rkrd.frames[frame].speed1_norm) {
                        printf("SPEED1_NORM %u\n", frame);
                        printf("%f ", player.speed1_norm);
                        printf("0x%x\n", f32_to_repr(player.speed1_norm));
                        printf("%f ", rkrd.frames[frame].speed1_norm);
                        printf("0x%x\n", f32_to_repr(rkrd.frames[frame].speed1_norm));
                }
                if (!vec3_equals(player.speed, rkrd.frames[frame].speed)) {
                        printf("SPEED %u\n", frame);
                        vec3_print(player.speed);
                        vec3_print(rkrd.frames[frame].speed);
                        desync = true;
                }
                if (!vec3_equals(player.rot_vec0, rkrd.frames[frame].rot_vec0)) {
                        printf("ROT_VEC0 %u\n", frame);
                        vec3_print(player.rot_vec0);
                        vec3_print(rkrd.frames[frame].rot_vec0);
                        desync = true;
                }
                if (!vec4_equals(player.rot, rkrd.frames[frame].rot)) {
                        printf("ROT %u\n", frame);
                        vec4_print(player.rot);
                        vec4_print(rkrd.frames[frame].rot);
                        desync = true;
                }
                if (!vec4_equals(player.rot2, rkrd.frames[frame].rot2)) {
                        printf("ROT2 %u\n", frame);
                        vec4_print(player.rot2);
                        vec4_print(rkrd.frames[frame].rot2);
                        desync = true;
                }
        }
}

int main(int argc, char **argv) {
        _MM_SET_FLUSH_ZERO_MODE(_MM_FLUSH_ZERO_ON);

        int ret = 1;

        if (argc != 3) {
                printf("Usage: hanachan <ghost.rkg> <dump.rkrd>\n");
                return ret;
        }

        struct rkg rkg = { 0 };
        struct rkrd rkrd = { 0 };

        if (!rkg_load(&rkg, argv[1])) {
                goto cleanup;
        }

        if (!rkrd_load(&rkrd, argv[2])) {
                goto cleanup;
        }

        replay(rkg, rkrd);

        ret = 0;

cleanup:
        free(rkrd.frames);
        free(rkg.inputs);

        return ret;
}
