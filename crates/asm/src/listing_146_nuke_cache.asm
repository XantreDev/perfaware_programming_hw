bits 64

section .text
    global nuke_l1

; (iterations: rdi, ptr: rsi)
nuke_l1:
    align 64
.loop:
    xor rcx, rcx
    mov rdx, rsi
    mov r8, 2251799813685248

    .inner:
        add rcx, 1

        vmovdqu ymm0, [rdx + 0]
        vmovdqu ymm0, [rdx + 4096]
        vmovdqu ymm0, [rdx + 8192]
        vmovdqu ymm0, [rdx + 12288]

        ;add rdx, 16384

        cmp rcx, 4
        shl r8, 1
        jne .inner

    sub rdi, 1
    jnle .loop
    ret
