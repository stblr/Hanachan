.text

.globl enable_ftz
.globl _enable_ftz
enable_ftz:
_enable_ftz:
    mrs x0, fpcr
    orr x0, x0, #0x1000000
    msr FPCR, x0
    ret
