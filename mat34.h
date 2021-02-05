#pragma once

#include "vec3.h"
#include "vec4.h"

struct mat34 {
        f32 e00;
        f32 e01;
        f32 e02;
        f32 e03;
        f32 e10;
        f32 e11;
        f32 e12;
        f32 e13;
        f32 e20;
        f32 e21;
        f32 e22;
        f32 e23;
};

struct mat34 mat34_from_angles_and_pos(struct vec3 angles, struct vec3 pos);

struct mat34 mat34_from_quat_and_pos(struct vec4 q, struct vec3 pos);

struct mat34 mat34_from_axis_angle(struct vec3 axis, f32 angle);

struct mat34 mat34_transpose(struct mat34 m);

struct mat34 mat34_mul(struct mat34 m0, struct mat34 m1);

struct vec3 mat34_mul_vec3(struct mat34 m, struct vec3 v);
