//! app管理的系统调用


use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};
use crate::timer::get_time_ms;
use crate::task::TaskStatus;
use crate::config::MAX_SYSCALL_NUM;
use crate::task::{get_syscall_info, get_total_time};

use super::{SYSCALL_EXIT, SYSCALL_GET_TIME, SYSCALL_WRITE, SYSCALL_YIELD, SYSCALL_TASK_INFO};

/// 退出当前应用
pub fn sys_exit(exit_id: i32) -> ! {
    println!("[kernel] App exited with code {}", exit_id);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// 主动让权上系统调用
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

/// 获取当前时间
pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
#[repr(C)]
pub struct TaskInfo {
    status: TaskStatus,
    call: [usize; MAX_SYSCALL_NUM],
    time: usize,
}

pub fn sys_task_info(id: usize, ts: *mut TaskInfo) -> isize {
    if id == SYSCALL_EXIT ||
        id == SYSCALL_WRITE ||
        id == SYSCALL_YIELD ||
        id == SYSCALL_GET_TIME ||
        id == SYSCALL_TASK_INFO {
            let call = get_syscall_info();
            let time = get_total_time();
            unsafe {
                *ts = TaskInfo {
                    status: TaskStatus::Running,
                    call,
                    time,
                }; 
            }
            0
    } else {
        println!("syscall id error");
        -1
    }
}
