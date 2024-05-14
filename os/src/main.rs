//! The main module and entrypoint
//!
//! Various facilities of the kernels are implemented as submodules. The most
//! important ones are:
//!
//! - [`trap`]: Handles all cases of switching from userspace to the kernel
//! - [`syscall`]: System call handling and implementation
//!
//! The operating system also starts in this module. Kernel code starts
//! executing from `entry.asm`, after which [`rust_main()`] is called to
//! initialize various pieces of functionality. (See its source code for
//! details.)
//!
//! We then call [`batch::run_next_app()`] and for the first time go to
//! userspace.

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
