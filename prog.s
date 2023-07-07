	.text
	.file	"prog.ll"
	.globl	main                            # -- Begin function main
	.p2align	4, 0x90
	.type	main,@function
main:                                   # @main
	.cfi_startproc
# %bb.0:                                # %entry
	subq	$24, %rsp
	.cfi_def_cfa_offset 32
	movl	$1, 20(%rsp)
	movl	$1, 16(%rsp)
	movl	$1, 12(%rsp)
	movl	$2, 8(%rsp)
	movl	$1, 4(%rsp)
	movl	$3, (%rsp)
	movl	$.Ldigit_fmt, %edi
	movl	$3, %esi
	xorl	%eax, %eax
	callq	printf@PLT
	xorl	%eax, %eax
	addq	$24, %rsp
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end0:
	.size	main, .Lfunc_end0-main
	.cfi_endproc
                                        # -- End function
	.type	.Ldigit_fmt,@object             # @digit_fmt
	.section	.rodata,"a",@progbits
.Ldigit_fmt:
	.asciz	"%d\n"
	.size	.Ldigit_fmt, 4

	.section	".note.GNU-stack","",@progbits
