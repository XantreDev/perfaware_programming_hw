bits 64

section .text
    global test_cache

; (iterations: rdi, memory_mask: rsi, buf: rdx)
test_cache:
    xor rax, rax
    mov rcx, rdx
    align 64
.loop
    vmovdqa ymm0, [rcx]
    vmovdqa ymm0, [rcx + 32]
    vmovdqa ymm0, [rcx + 64]
    vmovdqa ymm0, [rcx + 96]

    add rax, 128
    and rax, rsi
    lea rcx, [rdx + rax]

    sub rdi, 128
    jnle .loop
    ret
