#include "yaz.h"

#include <stdlib.h>

bool yaz_decompress(u32 *dst_size, u8 **dst_start, u32 src_size, const u8 *src) {
        u32 header_size = 0x10;
        if (src_size < header_size) {
                return false;
        }
        const u8 *src_end = src + src_size;

        u32 fourcc = get_u32(src + 0x0);
        u32 yaz0 = pack_u32('Y', 'a', 'z', '0');
        u32 yaz1 = pack_u32('Y', 'a', 'z', '1');
        if (fourcc != yaz0 && fourcc != yaz1) {
                return false;
        }

        *dst_size = get_u32(src + 0x4);
        *dst_start = alloc(*dst_size, sizeof(u8));
        u8 *dst = *dst_start;
        u8 *dst_end = *dst_start + *dst_size;

        src += header_size;

        u8 group_header;
        for (u8 i = 0; src < src_end && dst < dst_end; i = (i + 1) % 8) {
                if (i == 0) {
                        group_header = next_u8(&src);
                        if (src == src_end) {
                                goto fail;
                        }
                }
                if (group_header >> (7 - i) & 1) {
                        *dst++ = *src++;
                } else {
                        if (src + 2 > src_end) {
                                goto fail;
                        }
                        u16 val = next_u16(&src);
                        u8 *ref = dst - (val & 0xfff) - 1;
                        if (ref < *dst_start || ref >= dst) {
                                goto fail;
                        }
                        u16 ref_size = (val >> 12) + 2;
                        if (ref_size == 2) {
                                if (src + 1 > src_end) {
                                        goto fail;
                                }
                                ref_size = next_u8(&src) + 0x12;
                        }
                        if (dst + ref_size > dst_end) {
                                goto fail;
                        }
                        for (u16 j = 0; j < ref_size; j++) {
                                *dst++ = *ref++;
                        }
                }
        }

        if (dst == dst_end) {
                return true;
        }

fail:
        free(*dst_start);
        return false;
}
