#pragma once

#include "vec3.h"
#include "vec4.h"

struct vec4 quat_mul_from_vec3(struct vec4 q, struct vec3 v);

struct vec3 quat_rotate_vec3(struct vec4 q, struct vec3 v);

struct vec3 quat_inv_rotate_vec3(struct vec4 q, struct vec3 v);
