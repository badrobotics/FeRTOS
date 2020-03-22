    .syntax unified
    .section .text.asm
    .global do_exit
    .global do_sleep
    .global do_alloc
    .global do_dealloc
    .global do_block
    .global do_task_spawn
    .global do_yield
    .global ipc_publish
    .global ipc_subscribe
    .global ipc_get_message

    .thumb_func
do_exit:
    PUSH { LR }
    svc 0x0
    POP { PC }

    .thumb_func
do_sleep:
    PUSH { LR }
    svc 0x1
    POP { PC }

    .thumb_func
do_alloc:
    PUSH { LR }
    svc 0x2
    POP { PC }

    .thumb_func
do_dealloc:
    PUSH { LR }
    svc 0x3
    POP { PC }

    .thumb_func
do_block:
    PUSH { LR }
    svc 0x4
    POP { PC }

    .thumb_func
do_task_spawn:
    PUSH { LR }
    svc 0x5
    POP { PC }

    .thumb_func
do_yield:
    PUSH { LR }
    svc 0x6
    POP { PC }

    .thumb_func
ipc_publish:
    PUSH { LR }
    svc 0x7
    POP { PC }

    .thumb_func
ipc_subscribe:
    PUSH { LR }
    svc 0x8
    POP { PC }

    .thumb_func
ipc_get_message:
    PUSH { LR }
    svc 0x9
    POP { PC }
