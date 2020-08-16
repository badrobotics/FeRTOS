    .syntax unified
    .section .text.asm
    .weak sys_exit
    .weak sys_sleep
    .weak sys_alloc
    .weak sys_dealloc
    .weak sys_block
    .weak sys_task_spawn
    .weak sys_yield
    .weak sys_ipc_publish
    .weak sys_ipc_subscribe
    .weak sys_ipc_unsubscribe
    .weak sys_ipc_get_message
    .weak sys_get_heap_remaining
    .weak sys_interrupt_register
    .global svc_handler
.equ max_svc, 12

///////////////////////////////////////////////////////////////////////////////
// Arm Cortex-M interrupt stack frame order:
// high address: xPSR
//      |         PC
//      |         LR
//      |         R12
//      |         R3
//      |         R2
//      |         R1
//  low address:  R0
///////////////////////////////////////////////////////////////////////////////

    .thumb_func
//Do not overwrite R0-R3 because they hold the syscall params
svc_handler:
    //////////////////////////////
    //Retrieve the syscall number
    //////////////////////////////

    //Find the stacked PC
    //Offest is 24(decimal) + (registers pushed * 4)
    LDR R0, [SP, 0x18]
    //Get the svc instruction
    LDRH R0, [R0, -2]
    //Get the syscall number from the instruction
    BIC R0, R0, 0xFF00

    //////////////////////////////
    //Get the syscall address
    //////////////////////////////

    //Determine if the syscall number is valid
    CMP R0, max_svc
    BGT svc_handler_end

    //If it is valid, grab the address of the function
    ADR R1, svc_addr_table
    LDR R0, [R1, R0, LSL 2]

    //////////////////////////////
    //Fix the stack
    //////////////////////////////
    //The old PC in the ISR stackframe is the new LR in the ISR stackframe
    LDR R1, [SP, 0x18]
    //Setting the LSB of LR to 1 indicates that we are in thumb mode
    ORR R1, R1, 1
    STR R1, [SP, 0x14]

    //The new PC in the ISR stackframe is where we're jumping to
    STR R0, [SP, 0x18]

    .thumb_func
svc_handler_end:
    BX LR

.align 4
svc_addr_table:
    .word sys_exit               // 0
    .word sys_sleep              // 1
    .word sys_alloc              // 2
    .word sys_dealloc            // 3
    .word sys_block              // 4
    .word sys_task_spawn         // 5
    .word sys_yield              // 6
    .word sys_ipc_publish        // 7
    .word sys_ipc_subscribe      // 8
    .word sys_ipc_unsubscribe    // 9
    .word sys_ipc_get_message    // 10
    .word sys_get_heap_remaining // 11
    .word sys_interrupt_register // 12
