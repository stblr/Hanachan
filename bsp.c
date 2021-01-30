#include "bsp.h"

// TODO load from Common.szs instead of hardcoding Flame Runner values
void bsp_get(struct bsp *bsp) {
        *bsp = (struct bsp) {
                .initial_pos_y = 62.0f,
                .cuboids = {
                        { 90.0f, 80.0f, 140.0f },
                        { 0.0f, -10.0f, -40.0f },
                },
                .rot_speed = 0.12f,
                .wheels = {
                        {
                                .distance_suspension = 0.16f,
                                .speed_suspension = 0.18f,
                                .slack_y = 55.0f,
                                .topmost_pos = { 0.0f, -40.0f, 0.0f },
                                .wheel_radius = 29.5f,
                                .sphere_radius = 43.0f,
                        },
                        {
                                .distance_suspension = 0.17f,
                                .speed_suspension = 0.2f,
                                .slack_y = 30.0f,
                                .topmost_pos = { 0.0f, 7.0f, -75.0f },
                                .wheel_radius = 41.0f,
                                .sphere_radius = 43.0f,
                        },
                },
        };
}
