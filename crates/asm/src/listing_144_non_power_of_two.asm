bits 64

section .text
    global test_cache_non_bin


; (iterations: rdi, inner_reads: rsi, ptr: rdx)
test_cache_non_bin:
    align 64
.outer
    mov rax, rsi
    mov rcx, rdx

    ; 128b
    .inner
        vmovdqu ymm0, [rcx + 0x00]
        vmovdqu ymm0, [rcx + 0x20]
        vmovdqu ymm0, [rcx + 0x40]
        vmovdqu ymm0, [rcx + 0x60]

        add rcx, 0x80
        sub rax, 1
        jnle .inner

    sub rdi, 1
    jnle .outer
    ret
