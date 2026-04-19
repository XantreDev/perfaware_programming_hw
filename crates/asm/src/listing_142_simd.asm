bits 64

section .text
    global read_4x3, read_8x3, read_16x3, read_32x3, read_same_32x1, read_same_32x2, read_same_32x3, read_same_32x4, write_32x1, write_32x2, write_32x3

read_4x3:
    align 64
.loop:
    mov r8d, [rsi]
    mov r8d, [rsi+4]
    mov r8d, [rsi+8]

    sub rdi, 12
    jnle .loop
    ret

read_8x3:
    align 64
.loop:
    mov r8, [rsi]
    mov r8, [rsi+8]
    mov r8, [rsi+16]

    sub rdi, 24
    jnle .loop
    ret

read_16x3:
    align 64
.loop:
    vmovdqu xmm0, [rsi]
    vmovdqu xmm1, [rsi + 16]
    vmovdqu xmm2, [rsi + 32]

    sub rdi, 48
    jnle .loop
    ret

read_32x3:
    align 64
.loop:
    vmovdqu ymm0, [rsi]
    vmovdqu ymm1, [rsi + 32]
    vmovdqu ymm2, [rsi + 64]

    sub rdi, 96
    jnle .loop
    ret


read_same_32x1:
    align 64
.loop:
    vmovdqu ymm0, [rsi]

    sub rdi, 32
    jnle .loop
    ret

read_same_32x2:
    align 64
.loop:
    vmovdqu ymm0, [rsi]
    vmovdqu ymm1, [rsi]

    sub rdi, 64
    jnle .loop
    ret


read_same_32x3:
    align 64
.loop:
    vmovdqu ymm0, [rsi]
    vmovdqu ymm1, [rsi]
    vmovdqu ymm2, [rsi]

    sub rdi, 96
    jnle .loop
    ret

read_same_32x4:
    align 64
.loop:
    vmovdqu ymm0, [rsi]
    vmovdqu ymm1, [rsi]
    vmovdqu ymm2, [rsi]
    vmovdqu ymm3, [rsi]

    sub rdi, 128
    jnle .loop
    ret

write_32x1:
    align 64
.loop:
    vmovdqu [rsi], ymm0

    sub rdi, 32
    jnle .loop
    ret

write_32x2:
    align 64
.loop:
    vmovdqu [rsi], ymm0
    vmovdqu [rsi + 32], ymm0

    sub rdi, 64
    jnle .loop
    ret

write_32x3:
    align 64
.loop:
    vmovdqu [rsi], ymm0
    vmovdqu [rsi + 32], ymm0
    vmovdqu [rsi + 64], ymm0

    sub rdi, 96
    jnle .loop
    ret
