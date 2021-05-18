# inject at 8058fdbc (PAL)

.set sprintf, 0x80011a2c
.set ISFS_CreateDir, 0x80169e74
.set ISFS_CreateFile, 0x8016ac74
.set ISFS_Open, 0x8016ae5c
.set ISFS_Write, 0x8016b2c0
.set ptr_racedata, 0x809bd728
.set ptr_fd, 0x80000db0
.set ptr_buf, 0x80000dc0
.set magic, 0x524b5244

# allocate stack memory:
# * 0x8 needed by sprintf
# * 0x40 for the formatted string
addi r1, r1, -0x48

lis r3, ptr_fd@ha
lwz r3, ptr_fd@l (r3)
cmpwi r3, 0
bne end

# prepare the arguments for ISFS_CreateDir
bl dir_path
.string "/title/00010004/524d4350/data/rkrd\0"
dir_path:
mflr r3
li r4, 0
li r5, 3
li r6, 3
li r7, 3

lis r12, ISFS_CreateDir@h
ori r12, r12, ISFS_CreateDir@l
mtctr r12
bctrl

# check ISFS_CreateDir return value
cmpwi r3, -105
beq dir_already_exists
cmpwi r3, 0
bne end
dir_already_exists:

li r30, 0 # file number for the current track

create_file:

# prepare the arguments for sprintf
addi r3, r1, 0x8
bl file_path
.string "/title/00010004/524d4350/data/rkrd/%02x-%x.rkrd"
file_path:
mflr r4
lis r5, ptr_racedata@ha
lwz r5, ptr_racedata@l (r5)
lwz r5, 0xb68 (r5) # track id
mr r6, r30
crclr 4 * cr1 + eq

lis r12, sprintf@h
ori r12, r12, sprintf@l
mtctr r12
bctrl

# prepare the arguments for ISFS_CreateFile
addi r3, r1, 0x8
li r4, 0
li r5, 3
li r6, 3
li r7, 3

lis r12, ISFS_CreateFile@h
ori r12, r12, ISFS_CreateFile@l
mtctr r12
bctrl

# increment file number
addi r30, r30, 1

# check ISFS_CreateFile return value
cmpwi r3, -105
beq create_file # try again with incremented file number
cmpwi r3, 0
bne end

# prepare the arguments for ISFS_Open
addi r3, r1, 0x8
li r4, 2

lis r12, ISFS_Open@h
ori r12, r12, ISFS_Open@l
mtctr r12
bctrl

# check ISFS_Open return value
cmpwi r3, 0
blt end

# store file descriptor to EVA
lis r4, ptr_fd@ha
stw r3, ptr_fd@l (r4)

# store the rkrd header
ori r4, r4, ptr_buf@l
lis r5, magic@h
ori r5, r5, magic@l
stw r5, 0x0 (r4)
li r5, 2 # version
stw r5, 0x4 (r4)

# prepare the arguments for ISFS_Write
li r5, 0x8

lis r12, ISFS_Write@h
ori r12, r12, ISFS_Write@l
mtctr r12
bctrl

end:
# deallocate stack memory
addi r1, r1, 0x48

mr r3, r31 # original instruction
