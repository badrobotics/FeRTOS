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
    .global ipc_unsubscribe
    .global ipc_get_message
    .global do_get_heap_remaining
    .global do_register_interrupt

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
ipc_unsubscribe:
    PUSH { LR }
    svc 0x9
    POP { PC }

    .thumb_func
ipc_get_message:
    PUSH { LR }
    svc 0xa
    POP { PC }

    .thumb_func
do_get_heap_remaining:
    PUSH { LR }
    svc 0xb
    POP { PC }

    .thumb_func
do_register_interrupt:
    PUSH { LR }
    svc 0xc
    POP { PC }
