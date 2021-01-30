#pragma once

#include "util.h"

bool yaz_decompress(u32 *dst_size, u8 **dst, u32 src_size, const u8 *src);
