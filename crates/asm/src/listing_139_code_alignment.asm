bits 64

section .text
    global align_64, misalign_1, misalign_8, misalign_16, misalign_32, misalign_48, misalign_62, misalign_63

align_64:
    xor rax, rax
    align 64
.loop:
    inc rax
    cmp rax, rdi
    jb .loop
    ret

misalign_1:
    xor rax, rax
    align 64
    %rep 1
    nop
    %endrep
.loop:
    inc rax
    cmp rax, rdi
    jb .loop
    ret


misalign_8:
    xor rax, rax
    align 64
    %rep 8
    nop
    %endrep
.loop:
    inc rax
    cmp rax, rdi
    jb .loop
    ret


misalign_16:
    xor rax, rax
    align 64
    %rep 16
    nop
    %endrep
.loop:
    inc rax
    cmp rax, rdi
    jb .loop
    ret


misalign_32:
    xor rax, rax
    align 64
    %rep 32
    nop
    %endrep
.loop:
    inc rax
    cmp rax, rdi
    jb .loop
    ret


misalign_48:
    xor rax, rax
    align 64
    %rep 48
    nop
    %endrep
.loop:
    inc rax
    cmp rax, rdi
    jb .loop
    ret
misalign_62:
    xor rax, rax
    align 64
    %rep 62
    nop
    %endrep
.loop:
    inc rax
    cmp rax, rdi
    jb .loop
    ret

misalign_63:
    xor rax, rax
    align 64
    %rep 63
    nop
    %endrep
.loop:
    inc rax
    cmp rax, rdi
    jb .loop
    ret
