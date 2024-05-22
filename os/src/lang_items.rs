//! 程序panic处理，崩溃时打印文件名、行号、错误信息、栈回溯信息，并关机

use core::panic::PanicInfo;
use crate::sbi::shutdown;
use log::*;
use crate::stack_trace::print_stack_trace;

#[panic_handler]
/// 内核panic处理函数
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        error!(
            "Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        error!("Panicked: {}", info.message().unwrap());
    }
    unsafe { print_stack_trace(); }
    shutdown(true)
}

