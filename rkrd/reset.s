# inject at 805e3b78 (PAL)

.set ptr_fd, 0x80000db0

li r3, 0x0
lis r4, ptr_fd@ha
stw r3, ptr_fd@l (r4)

lwz r0, 0x24 (r1) # original instruction
