//! app管理的系统调用

use crate::batch::run_next_app;

/// 系统调用：退出当前应用
pub fn sys_exit(exit_id: i32) -> isize {
    println!("[kernel] App exited with code {}", exit_id);
    run_next_app();
}