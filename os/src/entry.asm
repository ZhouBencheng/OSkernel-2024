# os/src/entry.asm
    .section .text.entry
    .globl _start
_start:
    # 操作系统第一条指令——将栈帧指针设置到调用栈上
    la sp, boot_stack_top # la load address
    # 调用操作系统主函数
    call rust_main
    .section .bss.stack
    .globl boot_stack_lower_bound
    # 设置调用栈大小
boot_stack_lower_bound:
    .space 4096 * 16
    .globl boot_stack_top
boot_stack_top:
# 没有声明boot_stack_top的值但是却能正确将栈顶地址加载到栈指针寄存器中
# 是因为上述内存分配至boot_stack_lower_bound后已经分配完栈空间的大小
# 在最后这个boot_stack_top栈顶地址的值自动计算确定