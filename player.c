#include "player.h"

#include "mat34.h"
#include "quat.h"
#include "wii.h"

#include <float.h>
#include <math.h>
#include <stdio.h>

static struct mat34 mat34_from_diag(struct vec3 diag) {
        return (struct mat34) {
                diag.x,
                0.0f,
                0.0f,
                0.0f,
                0.0f,
                diag.y,
                0.0f,
                0.0f,
                0.0f,
                0.0f,
                diag.z,
                0.0f,
        };
}

static struct vec3 mat33_mul_vec3(struct mat34 m, struct vec3 v) {
        return (struct vec3) {
                m.e00 * v.x + m.e01 * v.y + m.e02 * v.z,
                m.e10 * v.x + m.e11 * v.y + m.e12 * v.z,
                m.e20 * v.x + m.e21 * v.y + m.e22 * v.z,
        };
}

/*static void mat34_print(struct mat34 m) {
        printf("%f %f %f %f\n", m.e00, m.e01, m.e02, m.e03);
        printf("%f %f %f %f\n", m.e10, m.e11, m.e12, m.e13);
        printf("%f %f %f %f\n", m.e20, m.e21, m.e22, m.e23);
        printf("\n");
}

static void mat34_print2(struct mat34 m) {
        printf("0x%x 0x%x 0x%x 0x%x\n", f32_to_repr(m.e00), f32_to_repr(m.e01), f32_to_repr(m.e02), f32_to_repr(m.e03));
        printf("0x%x 0x%x 0x%x 0x%x\n", f32_to_repr(m.e10), f32_to_repr(m.e11), f32_to_repr(m.e12), f32_to_repr(m.e13));
        printf("0x%x 0x%x 0x%x 0x%x\n", f32_to_repr(m.e20), f32_to_repr(m.e21), f32_to_repr(m.e22), f32_to_repr(m.e23));
        printf("\n");
}*/

static void wheel_init(struct wheel *wheel, u8 idx, struct bsp_wheel bsp_wheel, struct vec3 player_pos) {
        struct vec3 topmost_pos = vec3_add(player_pos, bsp_wheel.topmost_pos);
        // Due to a bug in the game, the initial position of the front wheel is incorrect
        struct vec3 bottom = { 0.0f, -1.0f, 0.0f };
        struct vec3 pos_rel = vec3_scale(bottom, bsp_wheel.slack_y);
        struct vec3 pos = vec3_add(topmost_pos, pos_rel);
        *wheel = (struct wheel) {
                .idx = idx,
                .bsp_wheel = bsp_wheel,
                .pos = pos,
                .down = bsp_wheel.slack_y,
                .last_pos_rel = pos_rel,
        };
}

static struct mat34 wheel_get_mat(u8 wheel_idx, struct player *player) {
        struct mat34 player_mat = mat34_from_quat_and_pos(player->rot2, player->pos);
        if (wheel_idx != 0) {
                return player_mat;
        }
        struct vec3 handle_pos = { 0.0f, 51.0f, 44.0f };
        struct vec3 handle_angles = { -25.0f, 0.0f, 0.0f };
        handle_angles = vec3_scale(handle_angles, M_PI / 180.0);
        struct mat34 handle_mat = mat34_from_angles_and_pos(handle_angles, handle_pos);
        struct mat34 wheel_mat = mat34_mul(player_mat, handle_mat);
        return wheel_mat;
}

static bool find_collision(struct vec3 pos, f32 radius, struct vec3 *nor) {
        f32 dist = 1000.0f - pos.y + radius;
        if (dist <= 0.0f) {
                return false;
        }
        *nor = vec3_scale((struct vec3) { 0.0f, 1.0f, 0.0f }, dist);
        return true;
}

static void wheel_update(struct wheel *wheel, struct player *player, struct vec4 last_q, u32 frame) {
        struct mat34 wheel_mat = wheel_get_mat(wheel->idx, player);
        struct vec3 topmost_pos = mat34_mul_vec3(wheel_mat, wheel->bsp_wheel.topmost_pos);

        struct vec3 bottom = { 0.0f, -1.0f, 0.0f };
        bottom = mat33_mul_vec3(wheel_mat, bottom);

        wheel->down += 5.0f;
        if (wheel->down > wheel->bsp_wheel.slack_y) {
                wheel->down = wheel->bsp_wheel.slack_y;
        }

        struct vec3 last_pos = wheel->pos;
        wheel->pos = vec3_add(topmost_pos, vec3_scale(bottom, wheel->down));

        f32 radius_diff = wheel->bsp_wheel.wheel_radius - wheel->bsp_wheel.sphere_radius;
        struct vec3 sphere_pos_rel = vec3_scale(bottom, radius_diff);
        struct vec3 sphere_pos = vec3_add(wheel->pos, sphere_pos_rel);
        struct mat34 player_mat = mat34_from_quat_and_pos(last_q, player->pos);
        struct vec3 col0 = { player_mat.e00, player_mat.e10, player_mat.e20 };
        col0 = vec3_scale(col0, player->turn_rot_z * wheel->bsp_wheel.sphere_radius * 0.3f);
        sphere_pos = vec3_add(sphere_pos, col0);
        f32 radius = wheel->bsp_wheel.sphere_radius;
        if (frame == 0) {
                radius = 10.0f;
        }
        struct vec3 nor;
        bool collision = find_collision(sphere_pos, radius, &nor);
        if (collision) {
                player->ground = true;
                struct vec3 floor_dir = { 0.0f, 1.0f, 0.0f };
                player->next_top = vec3_add(player->next_top, floor_dir);
                wheel->pos = vec3_add(wheel->pos, nor);
        }
        wheel->down = vec3_dot(bottom, vec3_sub(wheel->pos, topmost_pos));
        wheel->pos = vec3_add(topmost_pos, vec3_scale(bottom, wheel->down));

        if (collision) {
                struct vec3 speed = vec3_sub(wheel->pos, last_pos);
                speed = vec3_sub(speed, player->speed1);
                struct vec3 unk0 = { 0.0f, 10.0f * -1.3f, 0.0f };
                struct vec3 speed2 = vec3_add(speed, unk0);
                struct vec3 nor = { 0.0f, 1.0f, 0.0f };
                f32 dot = vec3_dot(speed2, nor);
                if (dot < 0.0f) {
                        struct vec3 zero = { 0.0f, 0.0f, 0.0f };
                        struct mat34 rot_mat = mat34_from_quat_and_pos(player->rot, zero);
                        struct mat34 inv_inertia_tensor = mat34_from_diag(player->inv_inertia_tensor);
                        struct mat34 tmp = mat34_mul(rot_mat, inv_inertia_tensor);
                        struct mat34 rot_mat_t = mat34_transpose(rot_mat);
                        rot_mat = mat34_mul(tmp, rot_mat_t);
                        struct vec3 sphere_pos_rel = vec3_sub(sphere_pos, player->pos);
                        struct vec3 cross = vec3_cross(sphere_pos_rel, nor);
                        cross = mat33_mul_vec3(rot_mat, cross);
                        struct vec3 cross2 = vec3_cross(cross, sphere_pos_rel);
                        f32 val = -dot / (1.0f + vec3_dot(nor, cross2));
                        struct vec3 cross3 = vec3_cross(nor, vec3_scale(speed, -1.0f));
                        struct vec3 cross4 = vec3_cross(cross3, nor);
                        if (vec3_sq_norm(cross4) > FLT_EPSILON) {
                                struct vec3 cross4_n = vec3_normalize(cross4);
                                f32 dot2 = vec3_dot(speed, cross4_n);
                                if (dot2 > 0.0f) {
                                        dot2 = 0.0f;
                                }
                                struct vec3 cross4_ns = vec3_scale(cross4_n, val * dot2 / dot);
                                struct vec3 forward = { 0.0f, 0.0f, 1.0f };
                                forward = quat_rotate_vec3(player->rot2, forward);
                                struct vec3 proj = vec3_proj_unit(cross4_ns, forward);
                                struct vec3 rej = vec3_sub(cross4_ns, proj);
                                f32 proj_norm = wii_sqrtf(vec3_sq_norm(proj));
                                f32 rej_norm = wii_sqrtf(vec3_sq_norm(rej));
                                f32 tmp = 0.1f * fabs(val);
                                if (fabs(proj_norm) > tmp) {
                                        if (proj_norm < 0.0f) {
                                                proj_norm = -tmp;
                                        } else {
                                                proj_norm = tmp;
                                        }
                                }
                                proj = vec3_normalize(proj);
                                proj = vec3_scale(proj, proj_norm);
                                tmp = wheel->down * fabs(val); // FIXME down
                                if (fabs(rej_norm) > tmp) {
                                        if (rej_norm < 0.0f) {
                                                rej_norm = -tmp;
                                        } else {
                                                rej_norm = tmp;
                                        }
                                }
                                rej = vec3_normalize(rej);
                                rej = vec3_scale(rej, rej_norm);
                                struct vec3 sum = vec3_add(proj, rej);
                                rej = vec3_rej_unit(sum, player->dir);
                                player->speed0 = vec3_add(player->speed0, rej);
                                if (!player->wheelie && player->wheelie_rot == 0.0f) {
                                        struct vec3 cross5 = vec3_cross(sphere_pos_rel, sum);
                                        struct vec3 cross5_r = mat33_mul_vec3(rot_mat, cross5);
                                        struct vec3 cross5_rr = quat_inv_rotate_vec3(player->rot, cross5_r);
                                        cross5_rr.y = 0;
                                        player->rot_vec0 = vec3_add(player->rot_vec0, cross5_rr);
                                }
                        }
                }
        }

        struct vec3 last_pos_rel = wheel->last_pos_rel;
        struct vec3 pos_rel = vec3_sub(wheel->pos, topmost_pos);
        wheel->last_pos_rel = pos_rel;
        // TODO move this up
        if (collision) {
                f32 down = vec3_dot(bottom, pos_rel);
                f32 speed = vec3_dot(bottom, vec3_sub(last_pos_rel, pos_rel));
                struct vec3 acceleration = vec3_scale(bottom, -(wheel->bsp_wheel.distance_suspension * (wheel->bsp_wheel.slack_y - down) + wheel->bsp_wheel.speed_suspension * speed));
                if (player->speed0.y < 5.0f) {
                        player->normal_acceleration += acceleration.y;
                }
                acceleration = quat_inv_rotate_vec3(player->rot2, acceleration);
                struct vec3 topmost_pos_rel = vec3_sub(topmost_pos, player->pos);
                topmost_pos_rel = quat_inv_rotate_vec3(player->rot2, topmost_pos_rel);
                struct vec3 cross = vec3_cross(topmost_pos_rel, acceleration);
                cross.y = 0.0f;
                if (player->wheelie_rot != 0.0f) {
                        cross.x = 0.0f;
                }
                if (!player->wheelie) {
                        player->normal_rot_vec = vec3_add(player->normal_rot_vec, cross);
                }
        }
}

void player_init(struct player *player, struct rkg rkg, struct bsp bsp) {
        f32 masses[2] = { 1.0f / 12.0f, 1.0f };
        struct vec3 inertia_tensors[2];
        for (u8 i = 0; i < 2; i++) {
                struct vec3 dims = bsp.cuboids[i];
                inertia_tensors[i] = (struct vec3) {
                        masses[i] * (dims.y * dims.y + dims.z * dims.z),
                        masses[i] * (dims.x * dims.x + dims.z * dims.z),
                        masses[i] * (dims.x * dims.x + dims.y * dims.y),
                };
        }
        struct vec3 inertia_tensor = vec3_add(inertia_tensors[0], inertia_tensors[1]);
        f32 det = inertia_tensor.x * inertia_tensor.y * inertia_tensor.z;
        f32 recip = 1.0f / det;
        struct vec3 inv_inertia_tensor = {
                recip * (inertia_tensor.y * inertia_tensor.z),
                recip * (inertia_tensor.z * inertia_tensor.x),
                recip * (inertia_tensor.x * inertia_tensor.y),
        };

        *player = (struct player) {
                .rkg = rkg,
                .bsp = bsp,
                .turn = 0.0f,
                .wheelie = false,
                .wheelie_frame = 0,
                .wheelie_rot = 0.0f,
                .wheelie_rot_dec = 0.0f,
                .ground = false,
                .next_top = { 0.0f, 0.0f, 0.0f },
                .top = { 0.0f, 0.0f, 0.0f },
                .dir = { 0.0f, 0.0f, -1.0f },
                .dir_diff = { 0.0f, 0.0f, 0.0f },
                .start_boost_charge = 0.0f,
                .standstill_boost_rot = 0.0f,
                .mt_boost = 0,
                .inv_inertia_tensor = inv_inertia_tensor,
                .pos = { -14720.0f, 1000.0f + bsp.initial_pos_y, -2954.655f },
                .normal_acceleration = 0.0f,
                .speed0 = { 0.0f, 0.0f, 0.0f },
                .soft_speed_limit = 0.0f,
                .speed1_norm = 0.0f,
                .speed1 = { 0.0f, 0.0f, 0.0f },
                .speed = { 0.0f, 0.0f, 0.0f },
                .normal_rot_vec = { 0.0f, 0.0f, 0.0f },
                .rot_vec0 = { 0.0f, 0.0f, 0.0f },
                .turn_rot_z = 0.0f,
                .rot = { 0.0f, 1.0f, 0.0f, 0.0f },
                .rot2 = { 0.0f, 1.0f, 0.0f, 0.0f },
        };

        for (u8 i = 0; i < 2; i++) {
                wheel_init(player->wheels + i, i, bsp.wheels[i], player->pos);
        }
}

static bool should_cancel_wheelie(struct player *player) {
        if (player->wheelie_frame < 15) {
                return false;
        }

        if (player->wheelie_frame > 180) {
                return true;
        }

        f32 base_speed = 82.95f + 1.06f; // TODO stop hardcoding fr + fk
        f32 speed_ratio = player->speed1_norm / base_speed;
        return player->speed1_norm < 0.0f || speed_ratio < 0.3f;
}

void player_update(struct player *player, u32 frame) {
        if (frame >= 172) {
                bool accelerate = player->rkg.inputs[frame - 172] & 1;
                if (accelerate) {
                        player->start_boost_charge += 0.02f - (0.02f - 0.002f) * player->start_boost_charge;
                } else {
                        player->start_boost_charge *= 0.96f;
                }

                if (frame == 411) {
                        player->mt_boost = 70;
                }

                if (player->rkg.inputs[frame - 172] >> 5 & 1) {
                        player->wheelie = true;
                }
                if (player->wheelie) {
                        player->wheelie_frame++;
                        if (should_cancel_wheelie(player)) {
                                player->wheelie = false;
                                player->wheelie_frame = 0;
                        } else {
                                player->wheelie_rot += 0.01f;
                                if (player->wheelie_rot > 0.07f) {
                                        player->wheelie_rot = 0.07f;
                                }
                        }
                } else if (player->wheelie_rot > 0.0f) {
                        player->wheelie_rot_dec -= 0.001f;
                        player->wheelie_rot += player->wheelie_rot_dec;
                        if (player->wheelie_rot < 0.0f) {
                                player->wheelie_rot = 0.0f;
                        }
                }

                i8 discrete_stick_x = (player->rkg.inputs[frame - 172] >> 12);
                f32 stick_x = (discrete_stick_x - 7.0f) / 7.0f;
                f32 s;
                if (stick_x < -0.2f && !player->wheelie) {
                        player->turn_rot_z -= 0.08f;
                        s = 1.0f;
                } else if (stick_x <= 0.2f || player->wheelie) {
                        player->turn_rot_z *= 0.9f;
                        s = 0.0f;
                } else {
                        player->turn_rot_z += 0.08f;
                        s = -1.0f;
                }
                if (player->turn_rot_z < -0.6f) {
                        player->turn_rot_z = -0.6f;
                } else if (player->turn_rot_z > 0.6f) {
                        player->turn_rot_z = 0.6f;
                } else {
                        struct mat34 player_mat = mat34_from_quat_and_pos(player->rot2, player->pos);
                        struct vec3 col0 = { player_mat.e00, player_mat.e10, player_mat.e20 };
                        col0 = vec3_scale(col0, s);
                        player->speed0 = vec3_add(player->speed0, col0);
                }
        }

        struct vec3 right = { 1.0f, 0.0f, 0.0f };
        right = quat_rotate_vec3(player->rot, right);
        struct vec3 next_dir = vec3_cross(right, player->top);
        next_dir = vec3_normalize(next_dir);
        next_dir = vec3_perp_in_plane(next_dir, player->top);
        struct vec3 next_dir_diff = vec3_sub(next_dir, player->dir);
        if (vec3_sq_norm(next_dir_diff) <= FLT_EPSILON) {
                player->dir = next_dir;
                player->dir_diff = (struct vec3) { 0.0f, 0.0f, 0.0f };
        } else {
                next_dir_diff = vec3_add(player->dir_diff, vec3_scale(next_dir_diff, 0.7f));
                player->dir = vec3_add(player->dir, next_dir_diff);
                player->dir = vec3_normalize(player->dir);
                player->dir_diff = vec3_scale(next_dir_diff, 0.1f);
        }

        if (frame >= 411) {
                i8 discrete_stick_x = (player->rkg.inputs[frame - 172] >> 12);
                f32 stick_x = (discrete_stick_x - 7.0f) / 7.0f;
                f32 reactivity = 0.88f;
                player->turn = reactivity * -stick_x + (1.0f - reactivity) * player->turn;
        }

        if (player->ground) {
                player->top = vec3_normalize(player->next_top);
        } else {
                player->top = (struct vec3) { 0.0f, 1.0f, 0.0f };
        }

        if (frame < 411) {
                player->speed0 = vec3_rej_unit(player->speed0, player->top);
        }

        player->speed0.y += player->normal_acceleration - 1.3f;
        player->normal_acceleration = 0.0f;

        player->speed0 = vec3_scale(player->speed0, 0.998f);

        struct vec3 forward = { 0.0f, 0.0f, 1.0f };
        forward = quat_rotate_vec3(player->rot, forward);
        forward.y = 0.0f;
        if (vec3_sq_norm(forward) > FLT_EPSILON) {
                player->speed0 = vec3_rej_unit(player->speed0, vec3_normalize(forward));
        }

        if (!player->mt_boost) {
                player->speed1_norm *= 0.9924f + (1.0f - 0.9924f) * (1.0f - fabsf(player->turn));
        }
        f32 last_speed1_norm = player->speed1_norm;
        f32 next_soft_speed_limit = 1.0f;
        if (player->mt_boost) {
                player->speed1_norm += 3.0f;
                next_soft_speed_limit = 1.2f;
                player->mt_boost--;
        }
        if (player->wheelie) {
                next_soft_speed_limit += 0.15f;
        }
        f32 base_speed = 82.95f + 1.06f; // TODO stop hardcoding fr + fk
        next_soft_speed_limit *= base_speed;
        player->soft_speed_limit -= 3.0f;
        if (next_soft_speed_limit > player->soft_speed_limit) {
                player->soft_speed_limit = next_soft_speed_limit;
        }
        if (player->speed1_norm > player->soft_speed_limit) {
                player->speed1_norm = player->soft_speed_limit;
        }

        struct vec3 speed1_dir = vec3_perp_in_plane(player->dir, player->top);
        right = vec3_cross(player->top, player->dir);
        f32 deg_to_rad = M_PI / 180.0;
        speed1_dir = mat33_mul_vec3(mat34_from_axis_angle(right, 0.5f * deg_to_rad), speed1_dir);
        player->speed1 = vec3_scale(speed1_dir, player->speed1_norm);

        player->speed = vec3_add(player->speed0, player->speed1);
        f32 speed_norm = vec3_norm(player->speed);
        player->speed = vec3_scale(vec3_normalize(player->speed), speed_norm);
        player->pos = vec3_add(player->pos, player->speed);

        if (player->wheelie) {
                player->rot_vec0.x *= 0.9f;
        }
        player->rot_vec0 = vec3_scale(player->rot_vec0, 0.98f);
        struct vec3 tmp = vec3_mul(player->inv_inertia_tensor, player->normal_rot_vec);
        struct vec3 tmp2 = vec3_mul(player->inv_inertia_tensor, vec3_add(player->normal_rot_vec, tmp));
        player->normal_rot_vec = vec3_scale(vec3_add(tmp, tmp2), 0.5f);
        player->rot_vec0 = vec3_add(player->rot_vec0, player->normal_rot_vec);
        player->rot_vec0.z = 0.0f;
        player->normal_rot_vec = (struct vec3) { 0.0f, 0.0f, 0.0f };

        struct vec3 rot_vec2 = { 0.0f, 0.0f, 0.0f };
        struct vec3 top = { 0.0f, 1.0f, 0.0f };
        f32 dot = vec3_dot(player->dir, top);
        rot_vec2.x -= player->wheelie_rot * (1.0f - fabsf(dot));

        f32 turn = player->turn * 0.0216f;
        turn *= 0.5f;
        if (player->wheelie) {
                turn *= 0.2f;
        }
        rot_vec2.y += turn;

        if (frame < 411) {
                player->standstill_boost_rot = 0.015f * -player->start_boost_charge;
        } else {
                f32 acceleration = player->speed1_norm - last_speed1_norm;
                acceleration = fminf(fmaxf(-3.0f, acceleration), 3.0f);
                player->standstill_boost_rot += 0.2f * (-acceleration * 0.15f * 0.08f - player->standstill_boost_rot);
        }
        rot_vec2.x += player->standstill_boost_rot;
        rot_vec2.z += 0.05f * player->turn_rot_z;

        struct vec3 rot_vec = vec3_scale(player->rot_vec0, player->bsp.rot_speed);
        rot_vec = vec3_add(rot_vec, rot_vec2);
        struct vec4 last_rot = player->rot;
        if (vec3_sq_norm(rot_vec) > FLT_EPSILON) {
                struct vec4 tmp3 = quat_mul_from_vec3(player->rot, rot_vec);
                player->rot = vec4_add(player->rot, vec4_scale(tmp3, 0.5f));
                player->rot = vec4_normalize(player->rot);
        }

        forward = (struct vec3) { 0.0f, 0.0f, 1.0f };
        forward = quat_rotate_vec3(player->rot, forward);
        right = vec3_cross(player->top, forward);
        forward = vec3_cross(right, player->top);
        forward = vec3_normalize(forward);
        right = vec3_cross(player->top, forward);
        top = vec3_cross(forward, right);
        top = vec3_normalize(top);
        struct vec3 rot_top = { 0.0f, 1.0f, 0.0f };
        rot_top = quat_rotate_vec3(player->rot, rot_top);
        if (vec3_dot(top, rot_top) < 0.9999f) {
                struct vec4 rot = quat_from_vectors(rot_top, top);
                struct vec4 prod = quat_mul(rot, player->rot);
                player->rot = quat_slerp(player->rot, prod, 0.1f);
        }
        player->rot = vec4_normalize(player->rot);

        struct vec4 identity = { 0.0f, 0.0f, 0.0f, 1.0f };
        player->rot2 = quat_mul(identity, player->rot);
        player->rot2 = quat_mul(player->rot2, identity);
        player->rot2 = vec4_normalize(player->rot2);

        player->ground = false;
        player->next_top = (struct vec3) { 0.0f, 0.0f, 0.0f };
        for (u8 i = 0; i < 2; i++) {
                wheel_update(player->wheels + i, player, last_rot, frame);
        }
}
