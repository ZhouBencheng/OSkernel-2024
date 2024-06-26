use core::{arch::asm, ptr};

pub unsafe fn print_stack_trace() -> (){ // 当前仅在panic发生时调用函数栈帧打印
    let mut fp: *const usize;
    asm!("mv {}, fp", out(reg) fp); // 将fp寄存器(汇编代码内)的值赋给fp变量（汇编代码外的高级语言参数）
    println!("=== Begin stack trace ===");
    while fp != ptr::null() {
        let saved_ra = *fp.sub(1); // 注意，这里 *const T类型的sub()方法的优先级高于 * 解引用的优先级，因此是先降低地址再解引用
        let saved_fp = *fp.sub(2);
        println!("ra = {:16x}, fp = {:16x}", saved_ra, saved_fp);
        fp = saved_fp as *const usize;
    }
    println!("=== End stack trace ===");
}

/*
 * 根据函数调用栈帧图显示
 * fp栈帧寄存器指向当前函数的栈帧起始地址
 * 由于栈帧向低地址生长，所以获取当前栈帧的返回地址和父栈帧地址是fp - 1和fp - 2
 * |-------------------|
 * |      Father       |
 * |    StackFrame     |
 * |-------------------| <- fp
 * |        ra         |
 * |-------------------|
 * |      prev fp      |
 * |-------------------|
 * |    Callee-saved   |
 * |-------------------|
 * |  Local Variable   |
 * |-------------------| <- sp
 * |                   |
 * */