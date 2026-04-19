bits 64

section .text
    global read_1, read_2, read_3, read_4, write_1, write_2, write_3, write_4, read_1x2, read_8x2

read_1:
    align 64
.loop:
    mov rax, [rsi]
    sub rdi, 1
    jnle .loop
    ret


read_2:
    align 64
.loop:
    mov rax, [rsi]
    mov rax, [rsi]

    sub rdi, 2
    jnle .loop
    ret

read_3:
    align 64
.loop:
    mov rax, [rsi]
    mov rax, [rsi]
    mov rax, [rsi]

    sub rdi, 3
    jnle .loop
    ret


read_4:
    align 64
.loop:
    mov rax, [rsi]
    mov rax, [rsi]
    mov rax, [rsi]
    mov rax, [rsi]

    sub rdi, 4
    jnle .loop
    ret


write_1:
    xor rax, rax
    align 64
.loop:
    mov [rsi], rax

    sub rdi, 1
    jnle .loop
    ret

write_2:
    xor rax, rax
    align 64
.loop:
    mov [rsi], rax
    mov [rdx], rax

    sub rdi, 2
    jnle .loop
    ret

write_3:
    xor rax, rax
    align 64
.loop:
    mov [rsi], rax
    mov [rdx], rax
    mov [rcx], rax

    sub rdi, 3
    jnle .loop
    ret

write_4:
    xor rax, rax
    align 64
.loop:
    mov [rsi], rax
    mov [rdx], rax
    mov [rcx], rax
    mov [r8], rax

    sub rdi, 4
    jnle .loop
    ret

read_1x2:
    align 64
.loop:
    mov al, [rsi]
    mov al, [rsi]

    sub rdi, 2
    jnle .loop
    ret


read_8x2:
    align 64
.loop:
    mov rax, [rsi]
    mov rax, [rsi]

    sub rdi, 2
    jnle .loop
    ret
