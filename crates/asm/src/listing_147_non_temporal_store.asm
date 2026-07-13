bits 64

section .data
    msg db "Assertion failed: must be dividable by 128", 10
    msg_len equ $ - msg

section .text
    global baseline_fill, non_temporal_fill

assert_failed:
    ; write(2, msg, msg_len)
    mov eax, 1              ; sys_write
    mov edi, 2              ; stderr
    lea rsi, [rel msg]
    mov edx, msg_len
    syscall

    ; exit(1)
    mov eax, 60             ; sys_exit
    mov edi, 1
    syscall


; (len: rdi, src_ptr: rsi, repeats: rdx, dst_ptr: rcx)
baseline_fill:
    ; let len_cp
    mov r8, rdi
    mov r9, rdi
    cmp r9, 0

    and r9, 0b0111_1111
    jnz .fail
    cmp r8, 0
    je .fail
    jmp .ok

.fail:
    call assert_failed
.ok:
    align 64
.loop:
    vmovdqa ymm0, [rsi]
    vmovdqa ymm1, [rsi + 32]
    vmovdqa ymm2, [rsi + 64]
    vmovdqa ymm3, [rsi + 96]

    mov r10, rcx ; let dst_ptr_cp = dst
    mov r11, rdx ; let counter = repeates
    .inner:
        vmovdqa [r10], ymm0
        vmovdqa [r10 + 32], ymm1
        vmovdqa [r10 + 64], ymm2
        vmovdqa [r10 + 96], ymm3
        add r10, rdi
        sub r11, 1
        jnz .inner

    add rsi, 128
    add rcx, 128
    sub r8, 128

    jnz .loop
    ret

; mostly copy
; (len: rdi, src_ptr: rsi, repeats: rdx, dst_ptr: rcx)
non_temporal_fill:
    ; let len_cp
    mov r8, rdi
    mov r9, rdi
    cmp r9, 0

    and r9, 0b0111_1111
    jnz .fail
    cmp r8, 0
    je .fail
    jmp .ok

.fail:
    call assert_failed
.ok:
    align 64
.loop:
    vmovdqa ymm0, [rsi]
    vmovdqa ymm1, [rsi + 32]
    vmovdqa ymm2, [rsi + 64]
    vmovdqa ymm3, [rsi + 96]

    mov r10, rcx ; let dst_ptr_cp = dst
    mov r11, rdx ; let counter = repeates
    .inner:
        vmovntdq [r10], ymm0
        vmovntdq [r10 + 32], ymm1
        vmovntdq [r10 + 64], ymm2
        vmovntdq [r10 + 96], ymm3
        add r10, rdi
        sub r11, 1
        jnz .inner

    add rsi, 128
    add rcx, 128
    sub r8, 128

    jnz .loop
    ret
