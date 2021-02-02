#include "util.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void *alloc(size_t count, size_t size) {
        if (size != 0 && count > (size_t)-1 / size) {
                abort();
        }
        void *buf = malloc(count * size);
        if (!buf) {
                abort();
        }
        return buf;
}

bool read_file(const char *path, u32 *size, u8 **data) {
        bool ret = false;

        FILE *input = fopen(path, "r");
        if (!input) {
                return ret;
        }

        if (fseek(input, 0, SEEK_END)) {
                goto cleanup;
        }

        long pos = ftell(input);
        if (pos < 0 || (u64)pos > UINT32_MAX) {
                goto cleanup;
        }
        *size = pos;

        if (fseek(input, 0, SEEK_SET)) {
                goto cleanup;
        }

        *data = alloc(*size, sizeof(u8));

        if (fread(*data, sizeof(u8), *size, input) != *size) {
                goto cleanup;
        }

        ret = true;

cleanup:
        fclose(input);

        return ret;
}

f32 f32_from_repr(u32 repr) {
        f32 val;
        memcpy(&val, &repr, sizeof(f32));
        return val;
}

u32 f32_to_repr(f32 f) {
        u32 repr;
        memcpy(&repr, &f, sizeof(u32));
        return repr;
}

f64 f64_from_repr(u64 repr) {
        f64 val;
        memcpy(&val, &repr, sizeof(f64));
        return val;
}

u64 f64_to_repr(f64 val) {
        u64 repr;
        memcpy(&repr, &val, sizeof(u64));
        return repr;
}

u8 get_u8(const u8 *data) {
        return data[0];
}

u16 get_u16(const u8 *data) {
        return (data[0] << 8) | data[1];
}

u32 get_u32(const u8 *data) { 
        return (data[0] << 24) | (data[1] << 16) | (data[2] << 8) | data[3];
}

f32 get_f32(const u8 *data) {
        return f32_from_repr(get_u32(data));
}

u8 next_u8(const u8 **data) {
        u8 val = get_u8(*data);
        *data += 1;
        return val;
}

u16 next_u16(const u8 **data) {
        u16 val = get_u16(*data);
        *data += 2;
        return val;
}

u32 next_u32(const u8 **data) {
        u32 val = get_u32(*data);
        *data += 4;
        return val;
}

f32 next_f32(const u8 **data) {
        f32 val = get_f32(*data);
        *data += 4;
        return val;
}

u32 pack_u32(u8 b0, u8 b1, u8 b2, u8 b3) {
        return (b0 << 24) | (b1 << 16) | (b2 << 8) | b3;
}
