.option push
.option norelax
.section .text.asm
.align 2
.local interrupt_switch
.local Reset
.local syscall_handler
.local get_cur_task
.local set_cur_task
.local get_next_task
.global context_switch
.global disable_interrupts
.global enable_interrupts
.global set_mtie
.global get_mepc
.global set_mepc
.global setup_interrupts
.global _start
.global trigger_context_switch

.equ regsize, 0x4
.equ mie_init, 0xB0B
.equ mie_mtie, 0xB0
.equ mstatus_mie, 0xB
.equ mcause_uecall, 0x8
.equ mcause_mecall, 0xB

context_switch:
    # a0 - Current Task
    # S1 - Next Task

    #Save the current mepc
    csrr t0, mepc
    sw t0, (sp)

    call get_next_task
    mv s1, a0
    call get_cur_task

    #Switch the stack pointers
    #Save the current stack pointer
    sw sp, (a0)
    #Load the new stack pointer
    lw sp, (s1)

    # Grab the new mepc
    lw t0, (sp)
    csrw mepc, t0

    #Set CUR_TASK = NEXT_TASK
    mv a0, s1
    call set_cur_task

    j interrupt_handler_end

.align 2
interrupt_handler:
    #Push everything onto the stack
    sw x1, -1*regsize(sp)
    # No need to save the stack pointer
    sw x3, -3*regsize(sp)
    sw x4, -4*regsize(sp)
    sw x5, -5*regsize(sp)
    sw x6, -6*regsize(sp)
    sw x7, -7*regsize(sp)
    sw x8, -8*regsize(sp)
    sw x9, -9*regsize(sp)
    sw x10, -10*regsize(sp)
    sw x11, -11*regsize(sp)
    sw x12, -12*regsize(sp)
    sw x13, -13*regsize(sp)
    sw x14, -14*regsize(sp)
    sw x15, -15*regsize(sp)
    sw x16, -16*regsize(sp)
    sw x17, -17*regsize(sp)
    sw x18, -18*regsize(sp)
    sw x19, -19*regsize(sp)
    sw x20, -20*regsize(sp)
    sw x21, -21*regsize(sp)
    sw x22, -22*regsize(sp)
    sw x23, -23*regsize(sp)
    sw x24, -24*regsize(sp)
    sw x25, -25*regsize(sp)
    sw x26, -26*regsize(sp)
    sw x27, -27*regsize(sp)
    sw x28, -28*regsize(sp)
    sw x29, -29*regsize(sp)
    sw x30, -30*regsize(sp)
    sw x31, -31*regsize(sp)
    addi sp, sp, -32*regsize

   #Grab the cause of the interrupt
   csrr t0, mcause
   #If the interrupt is for a syscall, handle that separately
   li t1, mcause_mecall
   beq t0, t1, interrupt_handler_syscall
   li t1, mcause_uecall
   beq t0, t1, interrupt_handler_syscall

   mv a0, t0
   call interrupt_switch
   j interrupt_handler_end

interrupt_handler_syscall:
   call syscall_handler

interrupt_handler_end:
    #Restore the registers
    addi sp, sp, 32*regsize
    lw x1, -1*regsize(sp)
    # No need to load the stack pointer
    lw x3, -3*regsize(sp)
    lw x4, -4*regsize(sp)
    lw x5, -5*regsize(sp)
    lw x6, -6*regsize(sp)
    lw x7, -7*regsize(sp)
    lw x8, -8*regsize(sp)
    lw x9, -9*regsize(sp)
    lw x10, -10*regsize(sp)
    lw x11, -11*regsize(sp)
    lw x12, -12*regsize(sp)
    lw x13, -13*regsize(sp)
    lw x14, -14*regsize(sp)
    lw x15, -15*regsize(sp)
    lw x16, -16*regsize(sp)
    lw x17, -17*regsize(sp)
    lw x18, -18*regsize(sp)
    lw x19, -19*regsize(sp)
    lw x20, -20*regsize(sp)
    lw x21, -21*regsize(sp)
    lw x22, -22*regsize(sp)
    lw x23, -23*regsize(sp)
    lw x24, -24*regsize(sp)
    lw x25, -25*regsize(sp)
    lw x26, -26*regsize(sp)
    lw x27, -27*regsize(sp)
    lw x28, -28*regsize(sp)
    lw x29, -29*regsize(sp)
    lw x30, -30*regsize(sp)
    lw x31, -31*regsize(sp)
    mret

setup_interrupts:
    #Set the interrupt handler
    la t0, interrupt_handler
    csrw mtvec, t0
    #Enable all of the interrupts except for the timer.
    #The timer will be enabled when the scheduling starts.
    li t0, mie_init
    csrs mie, t0
    csrsi mstatus, mstatus_mie
    ret

set_mtie:
    li t0, mie_mtie
    csrs mie, t0
    ret

get_mepc:
    csrr a0, mepc
    ret

set_mepc:
    csrw mepc, a0
    ret

disable_interrupts:
    csrci mstatus, mstatus_mie
    ret

enable_interrupts:
    csrsi mstatus, mstatus_mie
    ret

trigger_context_switch:
    addi sp, sp, -16
    sw ra, 12(sp)

    li a7, 0xff
    ecall

    lw ra, 12(sp)
    addi sp, sp, 16
    ret

.section .init, "ax"
_start:
    la sp, __stack_top
    call Reset

.option pop
