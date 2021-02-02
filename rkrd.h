#pragma once

#include "vec3.h"
#include "vec4.h"

struct rkrd_frame {
        struct vec3 dir;
        struct vec3 pos;
        struct vec3 speed0;
        f32 speed1_norm;
        struct vec3 speed;
        struct vec3 rot_vec0;
        struct vec3 rot_vec2;
        struct vec4 rot;
        struct vec4 rot2;
};

struct rkrd {
        u32 frame_count;
        struct rkrd_frame *frames;
};

bool rkrd_load(struct rkrd *rkrd, const char *path);
