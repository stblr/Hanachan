#pragma once

#include "vec3.h"

struct bsp_wheel {
        f32 distance_suspension;
        f32 speed_suspension;
        f32 slack_y;
        struct vec3 topmost_pos;
        f32 wheel_radius;
        f32 sphere_radius;
};

struct bsp {
        f32 initial_pos_y;
        struct vec3 cuboids[2];
        f32 rot_speed;
        struct bsp_wheel wheels[2];
};

void bsp_get(struct bsp *bsp);
