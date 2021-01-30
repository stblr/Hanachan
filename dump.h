#pragma once

#include "vec3.h"
#include "vec4.h"

struct dump_frame {
        struct vec3 pos;
        struct vec3 speed0;
        struct vec3 speed;
        struct vec4 rot;
};

struct dump {
        u32 frame_count;
        struct dump_frame *frames;
};

bool dump_load(struct dump *dump, const char *path);
