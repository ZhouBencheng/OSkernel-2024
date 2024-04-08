#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::arch::global_asm;
use crate::sbi::shutdown;
global_asm!(include_str!("entry.asm"));

#[macro_use]
mod console;
mod sbi;
mod lang_items;
mod logging;

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
    shutdown(false);
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
