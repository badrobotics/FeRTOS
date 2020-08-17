.option push
.option norelax
.section .text.asm
.align 2
.local context_switch
.local sys_exit
.local sys_sleep
.local sys_alloc
.local sys_dealloc
.local sys_block
.local sys_task_spawn
.local sys_yield
.local sys_ipc_publish
.local sys_ipc_subscribe
.local sys_ipc_get_message
.local sys_get_heap_remaining
.local sys_interrupt_register
.global syscall_handler

.equ context_switch_number, 0xFF
.equ max_syscall, 12

syscall_handler:
    #The syscall number is in a7

    #First, since mepc points to the ecall instruction, we need to increment mepc
    csrr t0, mepc
    addi t0, t0, 4
    csrw mepc, t0

    #Check if this is a context switch
    li t1, context_switch_number
    beq a7, t1, context_switch

    #Make sure a7 has a valid syscall number
    li t1, max_syscall
    bgt a7, t1, syscall_handler_end

    #Set RA to the previous pc
    sw t0, 31*4(sp)

    ##############################
    #Set the new PC to the syscall
    ##############################
    la t0, syscall_addr_table
    #Calculate the offset in the address table
    sll t1, a7, 2
    add t0, t0, t1
    lw t0, (t0)

    csrw mepc, t0

syscall_handler_end:
    ret

.align 2
syscall_addr_table:
    .word sys_exit               # 0
    .word sys_sleep              # 1
    .word sys_alloc              # 2
    .word sys_dealloc            # 3
    .word sys_block              # 4
    .word sys_task_spawn         # 5
    .word sys_yield              # 6
    .word sys_ipc_publish        # 7
    .word sys_ipc_subscribe      # 8
    .word sys_ipc_unsubscribe    # 9
    .word sys_ipc_get_message    # 10
    .word sys_get_heap_remaining # 11
    .word sys_interrupt_register # 12

.option pop
