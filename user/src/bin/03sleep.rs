#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{get_task_info, TaskInfo, MAX_SYSCALL_NUM, TaskStatus};

use user_lib::{get_time, yield_};

#[no_mangle]
fn main() -> i32 {
    let current_timer = get_time();
    let wait_for = current_timer + 3;
    while get_time() < wait_for {
        yield_();
    }
    let mut task_info = TaskInfo {
        status: TaskStatus::UnInit,
        call: [0; MAX_SYSCALL_NUM],
        time: 0,
    };
    get_task_info(64, &mut task_info);
    println!("TaskInfo03: status = {:?}, time = {}ms", task_info.status, task_info.time);
    println!("TaskInfo03: syscall_write = {}", task_info.call[64]);
    println!("TaskInfo03: syscall_exit = {}", task_info.call[93]);
    println!("TaskInfo03: syscall_yield = {}", task_info.call[124]);
    println!("Test sleep OK!");
    0
}
