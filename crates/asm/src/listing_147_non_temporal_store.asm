bits 64

section .text
    global baseline_fill, non_temporal_fill

section .data
    msg db "Assertion failed: must be dividable by 128", 10
    msg_len equ $ - msg

.assert_failed:
    mov eax, 4 ; sys_write sys call ID
    mov ebx, 2 ; fd - stderr
    mov ecx, msg
    mov edx, msg_len
    int 0x00 ; kernel interupt

    mov eax, 1 ; sys_exit
    mov ebx, 1 ; exit code
    int 0x80

; (len: rbx, src_ptr: rsi, repeats: rdx, dst_ptr: rcx)
baseline_fill:
    xor r8, r8

    mov r9, rbx
    or  r9, 0b0111_1111
    jz .assert_failed
    mov len

    align 64
.loop:
    ; mov rbx, rbx
    movdqa ymm0, [rsi]
    movdqa ymm1, [rsi + 32]
    movdqa ymm2, [rsi + 64]
    movdqa ymm3, [rsi + 96]

    mov r12, rcx
    mov r13, rdx ; let counter = repeates
    .inner:
        movdqa [r12], ymm0
        movdqa [r12 + 32], ymm1
        movdqa [r12 + 64], ymm2
        movdqa [r12 + 96], ymm3
        add r12, rbx
        sub r13, 128
        jnz .inner

    add rsi, 128
    sub r9, 128
    jnz .loop
