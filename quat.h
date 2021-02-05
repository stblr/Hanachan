#pragma once

#include "vec3.h"
#include "vec4.h"

struct vec4 quat_from_vectors(struct vec3 from, struct vec3 to);

struct vec4 quat_from_axis_angle(struct vec3 axis, f32 angle);

struct vec4 quat_mul(struct vec4 q0, struct vec4 q1);

struct vec4 quat_mul_from_vec3(struct vec4 q, struct vec3 v);

struct vec3 quat_rotate_vec3(struct vec4 q, struct vec3 v);

struct vec3 quat_inv_rotate_vec3(struct vec4 q, struct vec3 v);

struct vec4 quat_slerp(struct vec4 q0, struct vec4 q1, f32 t);
