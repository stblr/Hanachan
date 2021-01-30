#pragma once

#include "util.h"

struct rkg {
        u8 minutes;
        u8 seconds;
        u16 milliseconds;
        u8 track;
        u8 vehicle;
        u8 character;
        u16 year;
        u8 month;
        u8 day;
        u8 controller;
        bool compressed;
        u8 ghost_type;
        bool automatic_drift;
        u32 frame_count;
        u16 *inputs;
};

bool rkg_load(struct rkg *rkg, const char *path);
