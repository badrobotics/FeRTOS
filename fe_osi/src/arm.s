    .syntax unified
    .section .text.asm
    .global do_exit
    .global do_sleep
    .global do_alloc
    .global do_dealloc
    .global do_block
    .global do_task_spawn

    .thumb_func
do_exit:
    svc 0x0
    BX LR

    .thumb_func
do_sleep:
    svc 0x1
    BX LR

    .thumb_func
do_alloc:
    svc 0x2
    BX LR

    .thumb_func
do_dealloc:
    svc 0x3
    BX LR

    .thumb_func
do_block:
    svc 0x4
    BX LR

    .thumb_func
do_task_spawn:
    svc 0x5
    BX LR
