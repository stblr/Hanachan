#pragma once

#include "util.h"

struct vec3 {
        f32 x;
        f32 y;
        f32 z;
};

struct vec3 vec3_add(struct vec3 v0, struct vec3 v1);

struct vec3 vec3_sub(struct vec3 v0, struct vec3 v1);

struct vec3 vec3_mul(struct vec3 v0, struct vec3 v1);

struct vec3 vec3_scale(struct vec3 v, f32 s);

f32 vec3_dot(struct vec3 v0, struct vec3 v1);

struct vec3 vec3_cross(struct vec3 v0, struct vec3 v1);

struct vec3 vec3_proj_unit(struct vec3 v0, struct vec3 v1);

struct vec3 vec3_rej_unit(struct vec3 v0, struct vec3 v1);

f32 vec3_sq_norm(struct vec3 v);

f32 vec3_norm(struct vec3 v);

struct vec3 vec3_normalize(struct vec3 v);

bool vec3_equals(struct vec3 v0, struct vec3 v1);

void vec3_print(struct vec3 v);
