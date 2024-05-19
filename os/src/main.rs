//! rCore os入口main函数

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::arch::global_asm;

extern crate log;
extern crate riscv;
extern crate lazy_static;
extern crate sbi_rt;

#[macro_use]
mod console;
mod lang_items;
mod logging;
mod sbi;
mod sync;
pub mod syscall;
pub mod trap;
mod stack_trace;
mod timer;
mod config;
pub mod task;
mod loader;

#[path = "board/qemu.rs"]
mod board;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

#[no_mangle]
fn rust_main() -> ! {
    extern "C" {
        fn sbss();
        fn ebss();
        fn sdata();
        fn edata();
        fn srodata();
        fn erodata();
        fn boot_stack_lower_bound();
        fn boot_stack_top();
        fn stext();
        fn etext();
    }
    clear_bss();
    logging::init();
    println!("\u{1B}[31m[kernel] Hello world\x1b[0m");
    log::trace!(
        "[kernel] .text [{:#X}, {:#X})",
        stext as usize, etext as usize
    );
    log::debug!(
        "[kernel] .rodate [{:#X}, {:#X})",
        srodata as usize, erodata as usize
    );
    log::info!(
        "[kernel] .data [{:#X}, {:#X})",
        sdata as usize, edata as usize
    );
    log::warn!(
        "[kernel] .bss [{:#X}, {:#X})",
        sbss as usize, ebss as usize
    );
    log::error!(
        "[kernel] boot_stack_top = {:#X} boot_stack_lower_bound = {:#X}",
        boot_stack_top as usize, boot_stack_lower_bound as usize
    );
    trap::init();
    loader::load_app();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    trap::enable_fpu();
    
    // 开始检测内核态中断
    use riscv::register::{sstatus, sie};
    unsafe { sstatus::set_sie(); sie::set_stimer();}
    loop {
        if trap::check_kernel_interrupt() {
            println!("kernel interrupt returned.");
            break;
        }
    }
    unsafe { sstatus::clear_sie(); sie::clear_stimer(); }

    task::run_first_task();
    panic!("Unreachable in rust_main!");
}

fn clear_bss() { //bss段清零函数
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| {
        unsafe { (a as *mut u8).write_volatile(0) }
    });
}
