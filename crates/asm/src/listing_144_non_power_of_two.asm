bits 64

section .text:
    global test_cache_non_bin

; (iterations: rdi, inner_reads: rsi, ptr: rdx)
test_cache_non_bin:
    ; we SIGSEGV here for some reason xD
    ret
    xor rax, rax

    align 64
.outer
    xor rbx, rbx
    mov rcx, rdx

    ; 128b
    .inner
        ;vmovdqa ymm0, [rcx + 0x00]
        ;vmovdqa ymm0, [rcx + 0x20]
        ;vmovdqa ymm0, [rcx + 0x40]
        ;vmovdqa ymm0, [rcx + 0x60]

        add rcx, 0x80
        add rbx, 1
        cmp rsi, rbx
        jnle .inner

    add rax, 1
    cmp rdi, rax
    jnle .outer
    ret
