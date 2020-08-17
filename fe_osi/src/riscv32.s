.option push
.option norelax
.section .text.asm
.align 2
.globl do_exit
.globl do_sleep
.globl do_alloc
.globl do_dealloc
.globl do_block
.globl do_task_spawn
.globl do_yield
.globl ipc_publish
.globl ipc_subscribe
.globl ipc_unsubscribe
.globl ipc_get_message
.globl do_get_heap_remaining
.globl do_register_interrupt

do_exit:
    addi sp, sp, -16
    sw ra, 12(sp)
    sw fp, 8(sp)
    addi fp, sp, 16

    li a7, 0x0
    ecall

    lw fp, 8(sp)
    lw ra, 12(sp)
    addi sp, sp, 16
    ret

do_sleep:
    addi sp, sp, -16
    sw ra, 12(sp)
    sw fp, 8(sp)
    addi fp, sp, 16

    li a7, 0x1
    ecall

    lw fp, 8(sp)
    lw ra, 12(sp)
    addi sp, sp, 16
    ret

do_alloc:
    addi sp, sp, -16
    sw ra, 12(sp)
    sw fp, 8(sp)
    addi fp, sp, 16

    li a7, 0x2
    ecall

    lw fp, 8(sp)
    lw ra, 12(sp)
    addi sp, sp, 16
    ret

do_dealloc:
    addi sp, sp, -16
    sw ra, 12(sp)
    sw fp, 8(sp)
    addi fp, sp, 16

    li a7, 0x3
    ecall

    lw fp, 8(sp)
    lw ra, 12(sp)
    addi sp, sp, 16
    ret

do_block:
    addi sp, sp, -16
    sw ra, 12(sp)
    sw fp, 8(sp)
    addi fp, sp, 16

    li a7, 0x4
    ecall

    lw fp, 8(sp)
    lw ra, 12(sp)
    addi sp, sp, 16
    ret

do_task_spawn:
    addi sp, sp, -16
    sw ra, 12(sp)
    sw fp, 8(sp)
    addi fp, sp, 16

    li a7, 0x5
    ecall

    lw fp, 8(sp)
    lw ra, 12(sp)
    addi sp, sp, 16
    ret

do_yield:
    addi sp, sp, -16
    sw ra, 12(sp)
    sw fp, 8(sp)
    addi fp, sp, 16

    li a7, 0x6
    ecall

    lw fp, 8(sp)
    lw ra, 12(sp)
    addi sp, sp, 16
    ret

ipc_publish:
    addi sp, sp, -16
    sw ra, 12(sp)
    sw fp, 8(sp)
    addi fp, sp, 16

    li a7, 0x7
    ecall

    lw fp, 8(sp)
    lw ra, 12(sp)
    addi sp, sp, 16
    ret

ipc_subscribe:
    addi sp, sp, -16
    sw ra, 12(sp)
    sw fp, 8(sp)
    addi fp, sp, 16

    li a7, 0x8
    ecall

    lw fp, 8(sp)
    lw ra, 12(sp)
    addi sp, sp, 16
    ret

ipc_unsubscribe:
    addi sp, sp, -16
    sw ra, 12(sp)
    sw fp, 8(sp)
    addi fp, sp, 16

    li a7, 0x9
    ecall

    lw fp, 8(sp)
    lw ra, 12(sp)
    addi sp, sp, 16
    ret

ipc_get_message:
    addi sp, sp, -16
    sw ra, 12(sp)
    sw fp, 8(sp)
    addi fp, sp, 16

    li a7, 0xa
    ecall

    lw fp, 8(sp)
    lw ra, 12(sp)
    addi sp, sp, 16
    ret

do_get_heap_remaining:
    addi sp, sp, -16
    sw ra, 12(sp)
    sw fp, 8(sp)
    addi fp, sp, 16

    li a7, 0xb
    ecall

    lw fp, 8(sp)
    lw ra, 12(sp)
    addi sp, sp, 16
    ret

do_register_interrupt:
    addi sp, sp, -16
    sw ra, 12(sp)
    sw fp, 8(sp)
    addi fp, sp, 16

    li a7, 0xc
    ecall

    lw fp, 8(sp)
    lw ra, 12(sp)
    addi sp, sp, 16
    ret

.option pop
