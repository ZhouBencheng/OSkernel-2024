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
pub use self::context::TrapContext;

use crate::syscall::syscall;
use crate::timer::set_next_trigger;
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};
use core::arch::global_asm;
use core::ptr::addr_of_mut;

use log::trace;
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    stval, stvec, sie,
    sstatus::{self, FS},
};

static mut KERNEL_INTERRUPT_TRIGGERED: bool = false;

global_asm!(include_str!("trap.S"));

///将stvec寄存器的异常处理入口设置为`__alltraps`
pub fn init() {
    extern "C" {
        fn __alltraps();
    }
    unsafe {stvec::write(__alltraps as usize, TrapMode::Direct);}
}

/// 计时器模块使能
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

/// 使能浮点运算
pub fn enable_fpu() {
    unsafe {
        sstatus::set_fs(FS::Initial);
    }
}

/// 检查内核中断是否被触发
pub fn check_kernel_interrupt() -> bool {
    unsafe {(addr_of_mut!(KERNEL_INTERRUPT_TRIGGERED) as *mut bool).read_volatile()}
}


/// 标记内核中断已触发
pub fn trigger_kernel_interrupt() {
    unsafe {(addr_of_mut!(KERNEL_INTERRUPT_TRIGGERED) as *mut bool).write_volatile(true);}
}

/// Trap处理入口
#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    match sstatus::read().spp() {
        sstatus::SPP::Supervisor => kernel_trap_handler(cx),
        sstatus::SPP::User => user_trap_handler(cx),
    }
}

/// Kernel Trap处理入口
pub fn kernel_trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read(); // 获取异常原因
    let stval = stval::read();
    match scause.cause() {
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            // 内核中断来自一个时钟中断
            println!("kernel interrupt: from timer");
            // 标记一下触发了中断
            trigger_kernel_interrupt();
            set_next_trigger();
        }
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            panic!("[kernel] PageFault in kernel, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.", stval, cx.sepc);
        }
        _ => {
            // 其他的内核异常/中断
            panic!("unknown kernel exception or interrupt");
        }
    }
    cx
}

/// User Trap处理入口
pub fn user_trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    crate::task::user_time_end();
    let scause = scause::read(); // 获取异常原因
    trace!("Begin to handle trap");
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            println!("[kernel] PageFault in application, kernel killed it.");
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
    crate::task::user_time_start();
    cx
}
