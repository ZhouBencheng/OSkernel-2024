//! Trap处理模块
//! 
//! 在rCore中仅有唯一Trap处理入口，即 `__alltraps`
//! 在 [`init()`] 函数中，我们需要将stvec寄存器的异常处理入口设置为`__alltraps`
//! 
//! 所有的Trap处理首先都要先进入 `__alltraps`进行上下文保存
//! 这个函数入口能够保存trap上下文，让rust代码顺利运行，并且能够把
//! 控制权转交给[`trap_handler()`]
//! 
//! 该函数基于scause寄存器中不同的异常，调用不同的处理函数

mod context;

use crate::config::{TRAMPOLINE, TRAP_CONTEXT};
use crate::syscall::syscall;
use crate::task::{
    current_trap_cx, current_user_token, exit_current_and_run_next, suspend_current_and_run_next,
};
use crate::timer::set_next_trigger;
use core::arch::{asm, global_asm};
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec,
};

global_asm!(include_str!("trap.S"));

/// 初始化 CSR `stvec` 的入口地址为 `__alltraps`
pub fn init() {
    set_kernel_trap_entry();
}

/// 设置内核中断的处理函数
fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
    }
}

/// 设置用户中断的处理函数
fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
}

/// 使能时钟中断
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

#[no_mangle]
/// 处理中断、异常、系统调用
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    let cx = current_trap_cx();
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            println!("[kernel] PageFault in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.", stval, cx.sepc);
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            exit_current_and_run_next();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            suspend_current_and_run_next();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    trap_return();
}

#[no_mangle]
/// 在 TRAMPOLINE 页中设置__restore函数的地址
/// set the reg a0 = trap上下文指针, reg a1 = 用户页表物理地址
/// 最后跳转至 __restore 返回用户态
pub fn trap_return() -> ! {
    set_user_trap_entry();
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = current_user_token();
    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    unsafe {
        // 执行__restore的目的
        // 1. 切换到应用地址空间
        // 2. 从Trap上下文恢复通用寄存器
        // 3. sret继续执行应用
        asm!(
            "fence.i",
            "jr {restore_va}",             // jump to new addr of __restore asm function
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,      // a0 = virt addr of Trap Context
            in("a1") user_satp,        // a1 = phy addr of usr page table
            options(noreturn)
        );
    }
}

#[no_mangle]
/// 未实现内核态处理异常、中断
/// Todo: Chapter 9: I/O device
pub fn trap_from_kernel() -> ! {
    panic!("a trap from kernel!");
}

pub use self::context::TrapContext;
