
.code32

.global _start
.extern kernel_main

# System V ABI - Intel386 specification
# http://www.sco.com/developers/devspecs/abi386-4.pdf

# Code
.text

_start:
    # init stack
    mov $stack_start_plus_4_bytes, %esp
    sub $4, %esp
    mov $0, %ebp

    # System V ABI - Intel386 requirement: clear direction flag from EFLAGS.
    cld

    # extern "C" fn kernel_main(eax: u32, ebx: u32)
    push %ebx
    push %eax
    call kernel_main

# Stack area
.bss 0
.balign 1024*1024*2

.global READ_WRITE_PAGE_START_LOCATION
READ_WRITE_PAGE_START_LOCATION:
    .space 1024*1024*2 # 2 MiB
stack_start_plus_4_bytes:

# Interrupt handler macros

.macro interrupt number:req
.text
.global interrupt_\number
interrupt_\number:
    # System V ABI - Intel386 requirement: clear direction flag from EFLAGS.
    cld

    # System V ABI - Intel386 requirement: store general-purpose registers which
    # function call doesn't preserve.
    push %eax
    push %ecx
    push %edx

    # Call function.

    push $\number
    # extern "C" fn rust_interrupt_handler(interrupt_number: u32)
    call rust_interrupt_handler
    add $4, %esp

    # Restore general-purpose registers.
    mov (%esp), %edx
    mov 4(%esp), %ecx
    mov 8(%esp), %eax

    # Remove general-purpose registers from the stack.
    add $12, %esp

    # Stack pointer currently points to return EIP
    # so lets return from the interrupt handler.
    iret
.endm

.macro interrupt_with_error number:req
.text
.global interrupt_with_error_\number
interrupt_with_error_\number:
    # System V ABI - Intel386 requirement: clear direction flag from EFLAGS.
    cld

    # System V ABI - Intel386 requirement: store general-purpose registers which
    # function call doesn't preserve.
    push %eax
    push %ecx
    push %edx

    # Call function

    mov 12(%esp), %eax
    push %eax
    push $\number
    # extern "C" fn rust_interrupt_handler_with_error(
    #     interrupt_number: u32,
    #     error_code: u32)
    call rust_interrupt_handler_with_error
    add $8, %esp

    # Restore general-purpose registers.
    mov (%esp), %edx
    mov 4(%esp), %ecx
    mov 8(%esp), %eax

    # Remove general-purpose registers from the stack.
    add $12, %esp

    # Remove error code from the stack.
    add $4, %esp

    # Stack pointer currently points to return EIP
    # so lets return from the interrupt handler.
    iret
.endm

# Interrupt handlers

interrupt 0
interrupt 1
interrupt 2
interrupt 3
interrupt 4
interrupt 5
interrupt 6
interrupt 7
interrupt_with_error 8 # Double-Fault Exception
interrupt 9
interrupt_with_error 10 # Invalid-TSS Exception
interrupt_with_error 11 # Segment-Not-Present Exception
interrupt_with_error 12 # Stack Exception
interrupt_with_error 13 # General-Protection Exception
interrupt_with_error 14 # Page-Fault Exception
interrupt 15
interrupt 16
interrupt_with_error 17 # Alignment-Check Exception
interrupt 18
interrupt 19
interrupt 20
interrupt 21
interrupt 22
interrupt 23
interrupt 24
interrupt 25
interrupt 26
interrupt 27
interrupt 28
interrupt 29
interrupt 30
interrupt 31
interrupt 32
interrupt 33
interrupt 34
interrupt 35
interrupt 36
interrupt 37
interrupt 38
interrupt 39
interrupt 40
interrupt 41
interrupt 42
interrupt 43
interrupt 44
interrupt 45
interrupt 46
interrupt 47
interrupt 48
interrupt 49
interrupt 50
interrupt 51
interrupt 52
interrupt 53
interrupt 54
interrupt 55
interrupt 56
interrupt 57
interrupt 58
interrupt 59
interrupt 60
interrupt 61
interrupt 62
interrupt 63
interrupt 64
interrupt 65
interrupt 66
interrupt 67
interrupt 68
interrupt 69
interrupt 70
interrupt 71
interrupt 72
interrupt 73
interrupt 74
interrupt 75
interrupt 76
interrupt 77
interrupt 78
interrupt 79
interrupt 80
interrupt 81
interrupt 82
interrupt 83
interrupt 84
interrupt 85
interrupt 86
interrupt 87
interrupt 88
interrupt 89
interrupt 90
interrupt 91
interrupt 92
interrupt 93
interrupt 94
interrupt 95
interrupt 96
interrupt 97
interrupt 98
interrupt 99
interrupt 100
interrupt 101
interrupt 102
interrupt 103
interrupt 104
interrupt 105
interrupt 106
interrupt 107
interrupt 108
interrupt 109
interrupt 110
interrupt 111
interrupt 112
interrupt 113
interrupt 114
interrupt 115
interrupt 116
interrupt 117
interrupt 118
interrupt 119
interrupt 120
interrupt 121
interrupt 122
interrupt 123
interrupt 124
interrupt 125
interrupt 126
interrupt 127
interrupt 128
interrupt 129
interrupt 130
interrupt 131
interrupt 132
interrupt 133
interrupt 134
interrupt 135
interrupt 136
interrupt 137
interrupt 138
interrupt 139
interrupt 140
interrupt 141
interrupt 142
interrupt 143
interrupt 144
interrupt 145
interrupt 146
interrupt 147
interrupt 148
interrupt 149
interrupt 150
interrupt 151
interrupt 152
interrupt 153
interrupt 154
interrupt 155
interrupt 156
interrupt 157
interrupt 158
interrupt 159
interrupt 160
interrupt 161
interrupt 162
interrupt 163
interrupt 164
interrupt 165
interrupt 166
interrupt 167
interrupt 168
interrupt 169
interrupt 170
interrupt 171
interrupt 172
interrupt 173
interrupt 174
interrupt 175
interrupt 176
interrupt 177
interrupt 178
interrupt 179
interrupt 180
interrupt 181
interrupt 182
interrupt 183
interrupt 184
interrupt 185
interrupt 186
interrupt 187
interrupt 188
interrupt 189
interrupt 190
interrupt 191
interrupt 192
interrupt 193
interrupt 194
interrupt 195
interrupt 196
interrupt 197
interrupt 198
interrupt 199
interrupt 200
interrupt 201
interrupt 202
interrupt 203
interrupt 204
interrupt 205
interrupt 206
interrupt 207
interrupt 208
interrupt 209
interrupt 210
interrupt 211
interrupt 212
interrupt 213
interrupt 214
interrupt 215
interrupt 216
interrupt 217
interrupt 218
interrupt 219
interrupt 220
interrupt 221
interrupt 222
interrupt 223
interrupt 224
interrupt 225
interrupt 226
interrupt 227
interrupt 228
interrupt 229
interrupt 230
interrupt 231
interrupt 232
interrupt 233
interrupt 234
interrupt 235
interrupt 236
interrupt 237
interrupt 238
interrupt 239
interrupt 240
interrupt 241
interrupt 242
interrupt 243
interrupt 244
interrupt 245
interrupt 246
interrupt 247
interrupt 248
interrupt 249
interrupt 250
interrupt 251
interrupt 252
interrupt 253
interrupt 254
interrupt 255
