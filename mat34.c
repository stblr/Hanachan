#include "mat34.h"

#include "wii.h"

struct mat34 mat34_from_angles_and_pos(struct vec3 angles, struct vec3 pos) {
        f32 s_x = wii_sinf(angles.x);
        f32 s_y = wii_sinf(angles.y);
        f32 s_z = wii_sinf(angles.z);

        f32 c_x = wii_cosf(angles.x);
        f32 c_y = wii_cosf(angles.y);
        f32 c_z = wii_cosf(angles.z);

        return (struct mat34) {
                c_y * c_z,
                s_x * s_y * c_z - s_z * c_x,
                c_x * c_z * s_y + s_x * s_z,
                pos.x,
                s_z * c_y,
                s_x * s_y * s_z + c_x * c_z,
                s_z * c_x * s_y - s_x * c_z,
                pos.y,
                -s_y,
                s_x * c_y,
                c_x * c_y,
                pos.z,
        };
}

struct mat34 mat34_from_quat_and_pos(struct vec4 q, struct vec3 pos) {
        return (struct mat34) {
                1.0f - 2.0f * q.y * q.y - 2.0f * q.z * q.z,
                2.0f * q.x * q.y - 2.0f * q.w * q.z,
                2.0f * q.x * q.z + 2.0f * q.w * q.y,
                pos.x,
                2.0f * q.x * q.y + 2.0f * q.w * q.z,
                1.0f - 2.0f * q.x * q.x - 2.0f * q.z * q.z,
                2.0f * q.y * q.z - 2.0f * q.w * q.x,
                pos.y,
                2.0f * q.x * q.z - 2.0f * q.w * q.y,
                2.0f * q.y * q.z + 2.0f * q.w * q.x,
                1.0f - 2.0f * q.x * q.x - 2.0f * q.y * q.y,
                pos.z,
        };
}

struct mat34 mat34_transpose(struct mat34 m) {
        return (struct mat34) {
                m.e00,
                m.e10,
                m.e20,
                0.0f,
                m.e01,
                m.e11,
                m.e21,
                0.0f,
                m.e02,
                m.e12,
                m.e22,
                0.0f,
        };
}

static f32 mat34_mul_entry(struct vec4 row, struct vec4 col) {
        f32 acc = col.x * row.x;
        acc = (f64)col.y * row.y + acc;
        acc = (f64)col.z * row.z + acc;
        return (f64)col.w * row.w + acc;
}

struct mat34 mat34_mul(struct mat34 m0, struct mat34 m1) {
        struct vec4 row0 = { m0.e00, m0.e01, m0.e02, m0.e03 };
        struct vec4 row1 = { m0.e10, m0.e11, m0.e12, m0.e13 };
        struct vec4 row2 = { m0.e20, m0.e21, m0.e22, m0.e23 };

        struct vec4 col0 = { m1.e00, m1.e10, m1.e20, 0.0f };
        struct vec4 col1 = { m1.e01, m1.e11, m1.e21, 0.0f };
        struct vec4 col2 = { m1.e02, m1.e12, m1.e22, 0.0f };
        struct vec4 col3 = { m1.e03, m1.e13, m1.e23, 1.0f };

        return (struct mat34) {
                mat34_mul_entry(row0, col0),
                mat34_mul_entry(row0, col1),
                mat34_mul_entry(row0, col2),
                mat34_mul_entry(row0, col3),
                mat34_mul_entry(row1, col0),
                mat34_mul_entry(row1, col1),
                mat34_mul_entry(row1, col2),
                mat34_mul_entry(row1, col3),
                mat34_mul_entry(row2, col0),
                mat34_mul_entry(row2, col1),
                mat34_mul_entry(row2, col2),
                mat34_mul_entry(row2, col3),
        };
}

static f32 mat34_mul_vec3_entry(struct vec4 row, struct vec3 v) {
        f32 tmp0 = row.x * v.x;
        tmp0 = (f64)row.z * v.z + tmp0;
        f32 tmp1 = row.y * v.y + row.w;
        return tmp0 + tmp1;
}

struct vec3 mat34_mul_vec3(struct mat34 m, struct vec3 v) {
        struct vec4 row0 = { m.e00, m.e01, m.e02, m.e03 };
        struct vec4 row1 = { m.e10, m.e11, m.e12, m.e13 };
        struct vec4 row2 = { m.e20, m.e21, m.e22, m.e23 };

        return (struct vec3) {
                mat34_mul_vec3_entry(row0, v),
                mat34_mul_vec3_entry(row1, v),
                mat34_mul_vec3_entry(row2, v),
        };
}
