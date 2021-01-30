#include "vec4.h"

#include "wii.h"

#include <float.h>
#include <stdio.h>

struct vec4 vec4_add(struct vec4 v0, struct vec4 v1) {
        return (struct vec4) {
                v0.x + v1.x,
                v0.y + v1.y,
                v0.z + v1.z,
                v0.w + v1.w,
        };
}

struct vec4 vec4_scale(struct vec4 v, f32 s) {
        return (struct vec4) {
                s * v.x,
                s * v.y,
                s * v.z,
                s * v.w,
        };
}

struct vec4 vec4_normalize(struct vec4 v) {
        f32 sq_norm = v.w * v.w + v.x * v.x + v.y * v.y + v.z * v.z;
        if (sq_norm <= FLT_EPSILON) {
                return v;
        }
        f32 norm = wii_sqrtf(sq_norm);
        return vec4_scale(v, 1.0f / norm);
}

bool vec4_equals(struct vec4 v0, struct vec4 v1) {
        return v0.x == v1.x && v0.y == v1.y && v0.z == v1.z && v0.w == v1.w;
}

void vec4_print(struct vec4 v) {
        printf("%f %f %f %f ", v.x, v.y, v.z, v.w);
        printf("0x%x 0x%x 0x%x 0x%x\n", f32_to_repr(v.x), f32_to_repr(v.y), f32_to_repr(v.z), f32_to_repr(v.w));
}
