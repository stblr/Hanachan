# inject at 8058fec4 (PAL)

.set ISFS_Close, 0x8016b384
.set ptr_fd, 0x80000db0

# prepare the arguments for ISFS_Close
lis r3, ptr_fd@ha
lwz r3, ptr_fd@l (r3)

lis r12, ISFS_Close@h
ori r12, r12, ISFS_Close@l
mtctr r12
bctrl

mr r3, r29 # original instruction
