//! 实现TrapContext数据结构

use riscv::register::sstatus::{self, Sstatus, SPP};

// repr(C)属性告诉编译器按照C语言的结构体布局来组织内存
#[repr(C)]
/// 定义Trap上下文结构体
pub struct TrapContext {
    /* 需要保存的寄存器上下文
     * 32个通用寄存器x[0] ~ x[31] 
     * Smode的状态寄存器 
     * trap返回地址寄存器sepc 
     * 注意这里除了将整个结构体声明为pub外
     * 还将结构体内部的字段声明为pub，这是保证其他模块也能访问这个结构体的所有字段
     * 否则其他模块就只能访问结构体本身，而不能访问结构体内部的字段
     * */
    /// 通用寄存器 [0..31]
    pub x: [usize; 32],
    /// CSR sstatus
    pub sstatus: Sstatus,
    /// CSR sepc
    pub sepc: usize,
    /// 页表地址
    pub kernel_satp: usize,
    /// 内核栈地址
    pub kernel_sp: usize,
    /// trap处理函数地址
    pub trap_handler: usize,
}

impl TrapContext {
    /// 设置当前上下文的栈指针
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
    ///获取一个app执行环境的初始上下文（有所有权）
    pub fn app_init_context(
        entry: usize, 
        sp: usize,
        kernel_satp: usize,
        kernel_sp: usize,
        trap_handler: usize,
    ) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User); // 设置sstatus寄存器的SPP字段为User模式
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry, // 设置app执行入口
            kernel_satp,  
            kernel_sp,    
            trap_handler,
        };
        cx.set_sp(sp); // 设置当前上下文用户栈指针
        cx
    }
}
