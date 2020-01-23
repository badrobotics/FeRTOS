    .syntax unified
    .section .text.asm
    .weak sys_exit
    .weak sys_sleep
    .weak sys_alloc
    .weak sys_dealloc
    .weak sys_block
    .weak sys_task_spawn
    .global svc_handler

.equ max_svc, 5

    .thumb_func
//Do not overwrite R0-R3 because they hold the syscall params
svc_handler:
    PUSH {R4, R5, LR}

    //////////////////////////////
    //Load the parameters
    //////////////////////////////
    LDR R0, [SP, 0xC]
    LDR R1, [SP, 0x10]
    LDR R2, [SP, 0x14]
    LDR R3, [SP, 0x18]

    //////////////////////////////
    //Retrieve the syscall number
    //////////////////////////////

    //Find the stacked PC
    //Offest is 24(decimal) + (registers pushed * 4)
    LDR R4, [SP, 0x24]
    //Get the svc instruction
    LDRH R4, [R4, -2]
    //Get the syscall number from the instruction
    BIC R4, R4, 0xFF00

    //////////////////////////////
    //Call the syscall
    //////////////////////////////

    //Determine if the syscall number is valid
    CMP R4, max_svc
    BGT svc_handler_end

    //If it is valid, jump to the right place
    ADR R5, svc_jump_table
    LDR PC, [R5, R4, LSL 2]

.align 4
svc_jump_table:
    .word svc0 //exit
    .word svc1 //sleep
    .word svc2 //alloc
    .word svc3 //dealloc
    .word svc4 //block
    .word svc5 //task spawn

    .thumb_func
svc0:
    BL sys_exit
    B svc_handler_end
    .thumb_func
svc1:
    BL sys_sleep
    B svc_handler_end
    .thumb_func
svc2:
    BL sys_alloc
    B svc_handler_end
    .thumb_func
svc3:
    BL sys_dealloc
    B svc_handler_end
    .thumb_func
svc4:
    BL sys_block
    B svc_handler_end
    .thumb_func
svc5:
    BL sys_task_spawn
    B svc_handler_end
    .thumb_func
svc_handler_end:
    //Save the return value
    //Offset to the stored R0 is 0 + (registers pushed * 4)
    STR R0, [SP, 0xC]

    POP {R4, R5, PC}
