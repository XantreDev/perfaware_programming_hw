bits 64

section .text
    global no_prefetching, prefetching

; (len: RDI, access_ptr: RSI, ptr: RDX) ;  RCX, R8, R9
no_prefetching:
    xor r8, r8 ; let i = 0
    xor r9, r9

    align 64
.loop:
    mov ecx, [rsi + r8]
    xor r9, [rdx + rcx]

    add r8, 4
    cmp r8, rdi
    jle .loop

    mov rax, r9
    ret

; (len: RDI, access_ptr: RSI, ptr: RDX) ;  RCX, R8, R9
prefetching:
    xor r8, r8 ; let i = 0
    xor r9, r9
    sub rdi, 4

    align 64
.loop:
    mov ecx, [rsi + r8 + 4]
    prefetcht0 [rdx + rcx]

    mov ecx, [rsi + r8]
    xor r9, [rdx + rcx]

    xor r9, rax

    add r8, 4
    cmp r8, rdi
    jle .loop

    mov ecx, [rsi + r8]
    xor r9, [rdx + rcx]

    mov rax, r9
    ret
