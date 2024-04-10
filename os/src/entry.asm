# os/src/entry.asm
    .section .text.entry
    .globl _start
_start:
    la sp, boot_stack_top # la load address
    call rust_main
    .section .bss.stack
    .globl boot_stack_lower_bound
boot_stack_lower_bound:
    .space 4096 * 16
    .globl boot_stack_top
boot_stack_top:
# 没有声明boot_stack_top的值但是却能正确将栈顶地址加载到栈指针寄存器中
# 是因为上述内存分配至boot_stack_lower_bound后已经分配完栈空间的大小
# 在最后这个boot_stack_top栈顶地址的值自动计算确定