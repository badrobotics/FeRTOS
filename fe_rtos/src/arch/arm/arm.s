    .syntax unified
    .section .text.asm
    .weak get_cur_task
    .weak set_cur_task
    .weak get_next_task
    .global context_switch
    .global disable_interrupts
    .global enable_interrupts
    .thumb_func
context_switch:
    //R0 - Current Task
    //R4 - Next Task

    //Push registers R4-R11 of the old task
    PUSH {R4 - R11}

    //Load the variables
    //The return value is in R0
    BL get_next_task
    MOV R4, R0
    BL get_cur_task

    //Switch stack pointers
    //Save the current stack pointer
    STR SP, [R0]

    //Load the new stack pointer
    LDR SP, [R4]

    //Set CUR_TASK = NEXT_TASK
    MOV R0, R4
    BL set_cur_task

    //Clear the CPU pipeline
    ISB

    //Clear the exclusion monitors used in the LDREX and STREX instructions
    CLREX

    //Pop registers R4-R11
    POP {R4 - R11}

    MOV LR, 0xFFFFFFF9
    BX LR

    .thumb_func
disable_interrupts:
    cpsid if
    BX LR

    .thumb_func
enable_interrupts:
    cpsie if
    BX LR
