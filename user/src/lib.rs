#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]

#[macro_use]
pub mod console;
mod lang_items;
mod syscall;

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    clear_bss();
    exit(main());
    panic!("unreachable after sys_exit!");
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}

fn clear_bss() {
    extern "C" {
        fn start_bss();
        fn end_bss();
    }
    (start_bss as usize..end_bss as usize).for_each(|addr| unsafe {
        (addr as *mut u8).write_volatile(0);
    });
}

use syscall::*;

pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}
pub fn exit(exit_code: i32) -> isize {
    sys_exit(exit_code)
}
pub fn yield_() -> isize {
    sys_yield()
}
pub fn get_time() -> isize {
    sys_get_time()
}

pub const MAX_SYSCALL_NUM: usize = 520;

#[derive(Clone, Copy, Debug)]
pub enum TaskStatus {
    /// 未初始化
    UnInit,
    /// 就绪
    Ready,
    /// 运行
    Running,
    /// 退出
    Exited,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
#[repr(C)]
pub struct TaskInfo {
    pub status: TaskStatus,
    pub call: [usize; MAX_SYSCALL_NUM],
    pub time: usize,
}

pub fn get_task_info(id: usize, ts: *mut TaskInfo) -> isize {
    sys_task_info(id, ts as usize)
}
