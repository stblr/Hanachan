# inject at 805900b0 (PAL)

.macro copy_val offset, size
    addi r7, r6, \offset
    lswi r8, r7, \size
    stswi r8, r3, \size
    addi r3, r3, \size
.endm

.set ISFS_Write, 0x8016b2c0
.set ptr_raceinfo, 0x809bd730
.set ptr_fd, 0x80000db0
.set ptr_buf, 0x80000dc0

lis r4, ptr_buf@h
ori r4, r4, ptr_buf@l

addi r3, r4, 0xc # rot_vec2

lwz r5, 0x0 (r31) # player
lwz r5, 0x0 (r5) # playerPointers

lwz r6, 0x28 (r5) # playerSub10

copy_val 0x18, 0x4 # speed1_soft_limit
copy_val 0x20, 0x4 # speed1
copy_val 0x44, 0xc # floor_nor
copy_val 0x5c, 0xc # dir

lwz r6, 0x8 (r5) # playerGraphics
lwz r6, 0x90 (r6) # playerPhysicsHolder
lwz r6, 0x4 (r6) # playerPhysics

copy_val 0x68, 0xc # pos
copy_val 0x74, 0xc # vel0
copy_val 0xa4, 0xc # rot_vec0
copy_val 0xb0, 0xc # vel2
copy_val 0xd4, 0xc # vel
copy_val 0xf0, 0x10 # rot0
copy_val 0x100, 0x10 # rot1

lwz r6, 0x14 (r5) # playerModel

copy_val 0xfa, 0x2 # currentAnimation

lwz r6, 0x0 (r5) # playerParams
lbz r6, 0x10 (r6) # playerIdx
lis r5, ptr_raceinfo@ha
lwz r5, ptr_raceinfo@l (r5)
lwz r5, 0xc (r5)
mulli r6, r6, 0x4
lwzx r6, r5, r6 # raceinfoPlayer

copy_val 0xa, 0x2 # checkpoint_idx

# prepare the arguments for ISFS_Write
sub r5, r3, r4
lis r3, ptr_fd@ha
lwz r3, ptr_fd@l (r3)

lis r12, ISFS_Write@h
ori r12, r12, ISFS_Write@l
mtctr r12
bctrl

addi r29, r29, 1 # original instruction 
