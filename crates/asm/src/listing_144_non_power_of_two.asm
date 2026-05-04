bits 64

section .text:
    global test_cache

; (iterations: rdi, inner_iterations: rsi, ptr: rdx)
test_cache:
    xor rax, rax

    align 64
.outer
    xor rbx, rbx

    ; 256b
    .inner
        vmovqa ymm0, [rdx + 0x00]
        vmovqa ymm0, [rdx + 0x20]
        vmovqa ymm0, [rdx + 0x40]
        vmovqa ymm0, [rdx + 0x60]
        vmovqa ymm0, [rdx + 0x80]
        vmovqa ymm0, [rdx + 0xa0]
        vmovqa ymm0, [rdx + 0xc0]
        vmovqa ymm0, [rdx + 0xe0]

        add rdx, 0x100
        add rbx, 1
        cmp rsi, rbx
        jnz .inner

   add rax, 1
   cmp rdi, rax
   jnz .outer
   ret
