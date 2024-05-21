//! rCore os入口main函数

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

use core::arch::global_asm;

use log::*;


extern crate log;
extern crate riscv;
extern crate lazy_static;
extern crate sbi_rt;
extern crate alloc;
extern crate buddy_system_allocator;
extern crate xmas_elf;
#[macro_use]
extern crate bitflags;

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
mod mm;

#[path = "board/qemu.rs"]
mod board;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

#[no_mangle]
fn rust_main() -> ! {
    logging::init();
    clear_bss();
    trace!("Hello, world!");
    println!("[kernel] Hello, world!");
    mm::init();
    println!("[kernel] back to world!");
    mm::remap_test();
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
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
    trace!("bss segment cleared");
}
