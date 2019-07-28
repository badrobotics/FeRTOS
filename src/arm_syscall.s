    .syntax unified
    .section .text.asm
    .weak sys_exit
    .weak sys_sleep
    .global svc_handler

.equ max_svc, 1

    .thumb_func
//Do not overwrite R0-R3 because they hold the syscall params
svc_handler:
    PUSH {R4, R5, LR}

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

    .thumb_func
svc0:
    BL sys_exit
    B svc_handler_end
    .thumb_func
svc1:
    BL sys_sleep
    B svc_handler_end
    .thumb_func
svc_handler_end:
    POP {R4, R5, PC}
