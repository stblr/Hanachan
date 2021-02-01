#include "vec3.h"

#include "wii.h"

#include <float.h>
#include <math.h>
#include <stdio.h>

struct vec3 vec3_add(struct vec3 v0, struct vec3 v1) {
        return (struct vec3) {
                v0.x + v1.x,
                v0.y + v1.y,
                v0.z + v1.z,
        };
}

struct vec3 vec3_sub(struct vec3 v0, struct vec3 v1) {
        return (struct vec3) {
                v0.x - v1.x,
                v0.y - v1.y,
                v0.z - v1.z,
        };
}

struct vec3 vec3_mul(struct vec3 v0, struct vec3 v1) {
        return (struct vec3) {
                v0.x * v1.x,
                v0.y * v1.y,
                v0.z * v1.z,
        };
}

struct vec3 vec3_scale(struct vec3 v, f32 s) {
        return (struct vec3) {
                v.x * s,
                v.y * s,
                v.z * s
        };
}

f32 vec3_dot(struct vec3 v0, struct vec3 v1) {
        return v0.x * v1.x + v0.y * v1.y + v0.z * v1.z;
}

struct vec3 vec3_cross(struct vec3 v0, struct vec3 v1) {
        return (struct vec3) {
                v0.y * v1.z - v0.z * v1.y,
                v0.z * v1.x - v0.x * v1.z,
                v0.x * v1.y - v0.y * v1.x,
        };
}

struct vec3 vec3_proj_unit(struct vec3 v0, struct vec3 v1) {
        return vec3_scale(v1, vec3_dot(v0, v1));
}

struct vec3 vec3_rej_unit(struct vec3 v0, struct vec3 v1) {
        return vec3_sub(v0, vec3_proj_unit(v0, v1));
}

struct vec3 vec3_perp_in_plane(struct vec3 v0, struct vec3 v1) {
        bool colinear = fabsf(vec3_dot(v1, v0)) == 1.0f;
        if (colinear) {
                return (struct vec3) { 0.0f, 0.0f, 0.0f };
        }
        struct vec3 cross = vec3_cross(v1, v0);
        return vec3_normalize(vec3_cross(cross, v1));
}

f32 vec3_sq_norm(struct vec3 v) {
        return vec3_dot(v, v);
}

f32 vec3_norm(struct vec3 v) {
        f32 sq_norm = vec3_sq_norm(v);
        if (sq_norm <= FLT_EPSILON) {
                return 0.0f;
        }
        return wii_sqrtf(sq_norm);
}

struct vec3 vec3_normalize(struct vec3 v) {
        f32 norm = vec3_norm(v);
        if (norm == 0.0f) {
                return v;
        }
        return vec3_scale(v, 1.0f / norm);
}

bool vec3_equals(struct vec3 v0, struct vec3 v1) {
        return v0.x == v1.x && v0.y == v1.y && v0.z == v1.z;
}

void vec3_print(struct vec3 v) {
        printf("%f %f %f ", v.x, v.y, v.z);
        printf("0x%x 0x%x 0x%x\n", f32_to_repr(v.x), f32_to_repr(v.y), f32_to_repr(v.z));
}
