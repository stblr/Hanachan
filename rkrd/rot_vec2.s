# inject at 805b5ae4 (PAL)

.set ptr_buf, 0x80000dc0

lis r3, ptr_buf@h
ori r3, r3, ptr_buf@l

addi r4, r30, 0xe4
lswi r5, r4, 0xc
stswi r5, r3, 0xc

stfs f0, 0xec (r30) # original instruction
