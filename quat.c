#include "quat.h"

static struct vec4 quat_invert(struct vec4 q) {
        return (struct vec4) {
                -q.x,
                -q.y,
                -q.z,
                q.w,
        };
}

struct vec4 quat_mul_from_vec3(struct vec4 q, struct vec3 v) {
        return (struct vec4) {
                q.y * v.z - q.z * v.y + q.w * v.x,
                q.z * v.x - q.x * v.z + q.w * v.y,
                q.x * v.y - q.y * v.x + q.w * v.z,
                -(q.x * v.x + q.y * v.y + q.z * v.z),
        };
}

static struct vec3 quat_mul_to_vec3(struct vec4 q0, struct vec4 q1) {
        return (struct vec3) {
                q0.w * q1.x + q0.x * q1.w + q0.y * q1.z - q0.z * q1.y,
                q0.w * q1.y + q0.y * q1.w + q0.z * q1.x - q0.x * q1.z,
                q0.w * q1.z + q0.z * q1.w + q0.x * q1.y - q0.y * q1.x,
        };
}

struct vec3 quat_rotate_vec3(struct vec4 q, struct vec3 v) {
        struct vec4 tmp = quat_mul_from_vec3(q, v);
        struct vec4 q_inv = quat_invert(q);
        return quat_mul_to_vec3(tmp, q_inv);
}

struct vec3 quat_inv_rotate_vec3(struct vec4 q, struct vec3 v) {
        struct vec4 q_inv = quat_invert(q);
        struct vec4 tmp = quat_mul_from_vec3(q_inv, v);
        return quat_mul_to_vec3(tmp, q);
}
