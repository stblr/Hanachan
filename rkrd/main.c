#include <arpa/inet.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

#define ADDRESS_COUNT 38

const char *const addresses[ADDRESS_COUNT] = {
        // intro timer
        "9bd730 1c",
        // main timer
        "9bd730 20",
        // floor_nor
        "9c18f8 20 0 10 10 44",
        "9c18f8 20 0 10 10 48",
        "9c18f8 20 0 10 10 4c",
        // dir
        "9c18f8 20 0 10 10 5c",
        "9c18f8 20 0 10 10 60",
        "9c18f8 20 0 10 10 64",
        // pos
        "9c18f8 20 0 24 90 4 68",
        "9c18f8 20 0 24 90 4 6c",
        "9c18f8 20 0 24 90 4 70",
        // vel0
        "9c18f8 20 0 24 90 4 74",
        "9c18f8 20 0 24 90 4 78",
        "9c18f8 20 0 24 90 4 7c",
        // speed1
        "9c18f8 20 0 10 10 20",
        // vel2
        "9c18f8 20 0 24 90 4 b0",
        "9c18f8 20 0 24 90 4 b4",
        "9c18f8 20 0 24 90 4 bc",
        // vel
        "9c18f8 20 0 24 90 4 d4",
        "9c18f8 20 0 24 90 4 d8",
        "9c18f8 20 0 24 90 4 dc",
        // rot_vec0
        "9c18f8 20 0 24 90 4 a4",
        "9c18f8 20 0 24 90 4 a8",
        "9c18f8 20 0 24 90 4 ac",
        // rot_vec1
        "9c18f8 20 0 24 90 4 bc",
        "9c18f8 20 0 24 90 4 c0",
        "9c18f8 20 0 24 90 4 c4",
        // rot_vec2
        "4b0",
        "4b4",
        "4b8",
        // rot
        "9c18f8 20 0 24 90 4 f0",
        "9c18f8 20 0 24 90 4 f4",
        "9c18f8 20 0 24 90 4 f8",
        "9c18f8 20 0 24 90 4 fc",
        // rot2
        "9c18f8 20 0 24 90 4 100",
        "9c18f8 20 0 24 90 4 104",
        "9c18f8 20 0 24 90 4 108",
        "9c18f8 20 0 24 90 4 10c",
};

uint32_t pack_u32(uint8_t b0, uint8_t b1, uint8_t b2, uint8_t b3) {
        return (b0 << 24) | (b1 << 16) | (b2 << 8) | b3;
}

static void write_u32(uint32_t val, FILE *output) {
        val = htonl(val);
        fwrite(&val, sizeof(uint32_t), 1, output);
}

int main(int argc, char **argv) {
        if (argc != 3) {
                printf("Usage: hanachan-rkrd <MemoryWatcher> <dump.rkrd>\n");
                return 1;
        }

        int fd = socket(AF_UNIX, SOCK_DGRAM, 0);
        if (fd < 0) {
                return 1;
        }

        if (unlink(argv[1]) != 0) {
                return 1;
        }

        struct sockaddr_un addr = {
                .sun_family = AF_UNIX,
        };
        strncpy(addr.sun_path, argv[1], sizeof(addr.sun_path) - 1);
        if (bind(fd, (struct sockaddr *)&addr, sizeof(addr)) != 0) {
                return 1;
        }

        char buf[1024];
        uint32_t vals[ADDRESS_COUNT] = { 0 };
        uint32_t last_frame = 0;

        FILE *output = fopen(argv[2], "w");
        if (!output) {
                return 1;
        }

        uint32_t fourcc = pack_u32('R', 'K', 'R', 'D');
        write_u32(fourcc, output);

        uint32_t version = 1;
        write_u32(version, output);

        while (1) {
                recv(fd, buf, sizeof(buf), 0);

                char *ptr = buf;

                while (*ptr) {
                        uint32_t i;
                        for (i = 0; i < ADDRESS_COUNT; i++) {
                                if (!strncmp(addresses[i], ptr, strlen(addresses[i]))) {
                                        break;
                                }
                        }
                        if (i == ADDRESS_COUNT) {
                                break;
                        }

                        ptr = strchr(ptr, '\n');
                        if (!ptr) {
                                break;
                        }

                        char *endptr;

                        uint32_t val = strtol(ptr, &endptr, 16);
                        vals[i] = val;

                        if (endptr == ptr || *endptr != '\n') {
                                break;
                        }

                        ptr = endptr + 1;
                }

                uint32_t frame = (vals[0] & 0xffff) + vals[1];
                if (frame > last_frame) {
                        for (uint8_t i = 2; i < ADDRESS_COUNT; i++) {
                                write_u32(vals[i], output);
                        }
                        fflush(output);
                }
                last_frame = frame;
        }

        return 0;
}
