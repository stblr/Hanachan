#include "wii.h"

#include "wii_tables.h"

#include <float.h>
#include <math.h>

static f64 ppc_frsqrte(f64 val) {
        u64 repr = f64_to_repr(val);
        u64 mantissa = repr & ((1ull << 52) - 1);
        u64 sign = repr & 1ull << 63;
        u64 exponent = repr & 0x7ffull << 52;

        // Special case 0
        if (mantissa == 0 && exponent == 0) {
                return sign ? -DBL_MAX : DBL_MAX;
        }

        // Special case NaN-ish numbers
        if (exponent == 0x7ffull << 52) {
                if (mantissa == 0) {
                        return sign ? nan("") : 0.0;
                }
                return 0.0 + val;
        }

        // Negative numbers return NaN
        if (sign) {
                return nan("");
        }

        if (exponent == 0) {
                // "Normalize" denormal values
                do {
                        exponent -= 1ull << 52;
                        mantissa <<= 1;
                } while (!(mantissa & 1ull << 52));
                mantissa &= (1ull << 52) - 1;
                exponent += 1ull << 52;
        }

        bool odd_exponent = !(exponent & 1ull << 52);
        exponent = ((0x3ffull << 52) - ((exponent - (0x3feull << 52)) / 2)) & (0x7ffull << 52);
        repr = sign | exponent;

        u32 i = mantissa >> 37;
        u32 idx = i / 2048 + (odd_exponent ? 16 : 0);
        repr |= (u64)(frsqrte_bases[idx] - frsqrte_decs[idx] * (i % 2048)) << 26;

        return f64_from_repr(repr);
}

static f64 f64_25_bit_mantissa(f64 val) {
        u64 repr = f64_to_repr(val);
        repr = (repr & 0xfffffffff8000000ull) + (repr & 0x8000000);
        return f64_from_repr(repr);
}

f32 wii_sqrtf(f32 val) {
        if (val <= 0.0f) {
                return 0.0f;
        }
        f64 recip_sqrt = ppc_frsqrte(val);
        f32 tmp0 = recip_sqrt * f64_25_bit_mantissa(recip_sqrt);
        f32 tmp1 = recip_sqrt * 0.5f;
        f32 tmp2 = 3.0f - (f64)tmp0 * val;
        return tmp1 * tmp2 * val;
}

f32 wii_sinf(f32 val) {
        f32 step = 256.0 / (2.0 * M_PI);
        val *= step;
        f32 f_idx = fabs(val);
        while (f_idx > 65536.0f) {
                f_idx -= 65536.0f;
        }
        u16 idx = f_idx;
        idx %= 256;
        f32 sin_f_idx = trig_table[idx][0] + (f_idx - (f32)idx) * trig_table[idx][2];
        if (val < 0.0f) {
                return -sin_f_idx;
        }
        return sin_f_idx;
}

f32 wii_cosf(f32 val) {
        f32 step = 256.0 / (2.0 * M_PI);
        val *= step;
        f32 f_idx = fabs(val);
        while (f_idx > 65536.0f) {
                f_idx -= 65536.0f;
        }
        u16 idx = f_idx;
        idx %= 256;
        return trig_table[idx][1] + (f_idx - (f32)idx) * trig_table[idx][3];
}
