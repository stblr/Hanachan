#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef uint8_t u8;
typedef uint16_t u16;
typedef uint32_t u32;
typedef uint64_t u64;
typedef int8_t i8;
typedef float f32;
typedef double f64;

#define M_PI (3.14159265358979323846)

void *alloc(size_t count, size_t size);

bool read_file(const char *path, u32 *size, u8 **data);

f32 f32_from_repr(u32 repr);

u32 f32_to_repr(f32 f);

f64 f64_from_repr(u64 repr);

u64 f64_to_repr(f64 val);

u8 get_u8(const u8 *data);

u16 get_u16(const u8 *data);

u32 get_u32(const u8 *data);

f32 get_f32(const u8 *data);

u8 next_u8(const u8 **data);

u16 next_u16(const u8 **data);

u32 pack_u32(u8 b0, u8 b1, u8 b2, u8 b3);
