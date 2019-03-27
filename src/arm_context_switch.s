    .syntax unified
    .section .text.context_switch
    .weak get_cur_task
    .weak set_cur_task
    .weak get_next_task
    .weak get_task_table
    .weak get_task_size
    .global context_switch
    .global ret_from_isr
    .thumb_func
context_switch:
    //R0 - Current Task
    //R1 - Next Task
    //R2 - Pointer to Task table

    //Push registers R4-R11 of the old task
    PUSH {R4 - R11}

    //The size of the Task struct
    BL get_task_size
    MOV R4, R0

    //Load the variables
    //The return value is in R0
    BL get_next_task
    MOV R1, R0
    BL get_task_table
    MOV R2, R0
    BL get_cur_task

    //Switch stack pointers
    //Save the current stack pointer
    MUL R3, R0, R4
    STR SP, [R2, R3]

    //Load the new stack pointer
    MUL R3, R1, R4
    LDR SP, [R2, R3]

    //Set the CUR_TASK = NEXT_TASK
    MOV R0, R1
    BL set_cur_task

    //Clear the CPU pipeline
    ISB

    //Pop registers R4-R11
    POP {R4 - R11}

    MOV LR, 0xFFFFFFF9
    BX LR

ret_from_isr:
    MOV LR, #0xFFFFFFF9
    BX LR
