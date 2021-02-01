#include "quat.h"

#include "wii.h"

#include <float.h>
#include <math.h>

struct vec4 quat_from_vectors(struct vec3 from, struct vec3 to) {
        f32 s = 2.0f * (vec3_dot(from, to) + 1.0f);
        s = wii_sqrtf(s);
        if (s <= FLT_EPSILON) {
                return (struct vec4) { 0.0f, 0.0f, 0.0f, 1.0f };
        }
        f32 recip = 1.0f / s;
        struct vec3 cross = vec3_cross(from, to);
        return (struct vec4) {
                recip * cross.x,
                recip * cross.y,
                recip * cross.z,
                0.5f * s,
        };
}

static struct vec4 quat_invert(struct vec4 q) {
        return (struct vec4) {
                -q.x,
                -q.y,
                -q.z,
                q.w,
        };
}

struct vec4 quat_mul(struct vec4 q0, struct vec4 q1) {
        return (struct vec4) {
                q0.w * q1.x + q0.x * q1.w + q0.y * q1.z - q0.z * q1.y,
                q0.w * q1.y + q0.y * q1.w + q0.z * q1.x - q0.x * q1.z,
                q0.w * q1.z + q0.z * q1.w + q0.x * q1.y - q0.y * q1.x,
                q0.w * q1.w - q0.x * q1.x - q0.y * q1.y - q0.z * q1.z,
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

struct vec4 quat_slerp(struct vec4 q0, struct vec4 q1, f32 t) {
        f32 dot = vec4_dot(q0, q1);
        if (dot < -1.0f) {
                dot = -1.0f;
        } else if (dot > 1.0f) {
                dot = 1.0f;
        }

        // TODO do we need a wii_acosf function?
        f32 angle = dot < 0.0f ? acosf(-dot) : acosf(dot);
        f32 sin = wii_sinf(angle);

        f32 s;
        if (sin <= -1e-5 || sin >= 1e-5) {
                f32 recip = 1.0f / sin;
                s = recip * wii_sinf(angle - t * angle);
                t = recip * wii_sinf(t * angle);
        } else {
                s = 1.0f - t;
        }

        if (dot < 0.0f) {
                t = -t;
        }

        return vec4_add(vec4_scale(q0, s), vec4_scale(q1, t));
}
