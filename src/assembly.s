
.code32

.global _start
.extern kernel_main

# Code
.text

_start:
    # init stack
    mov $stack_start, %esp
    mov $0, %ebp

    # jump to rust
    jmp kernel_main

# Stack area
.bss
.balign 4

stack_end:
    .space 1024*1024 # 1 MiB
stack_start:


# Interrupt handlers
.text

.global interrupt_hander
.extern rust_interrupt_handler

interrupt_hander:
    call rust_interrupt_handler
    # TODO: pop error codes if they exists (specific interrupt numbers)
    iret







