#pragma once

#include "util.h"

struct vec4 {
        f32 x;
        f32 y;
        f32 z;
        f32 w;
};

struct vec4 vec4_add(struct vec4 v0, struct vec4 v1);

struct vec4 vec4_scale(struct vec4 v, f32 s);

struct vec4 vec4_normalize(struct vec4 v);

bool vec4_equals(struct vec4 v0, struct vec4 v1);

void vec4_print(struct vec4 v);
