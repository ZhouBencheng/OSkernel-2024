//! 任务管理模块

use crate::config::MAX_SYSCALL_NUM;

use super::TaskContext;

/// 任务控制块
#[derive(Clone, Copy)]
pub struct TaskControlBlock {
    /// 任务状态
    pub task_status: TaskStatus,
    /// 任务上下文
    pub task_cx: TaskContext,
    /// 内核态运行时间
    pub kernel_time:usize,
    /// 用户态运行时间
    pub user_time:usize,
    /// 系统调用信息
    pub syscall_info: [usize; MAX_SYSCALL_NUM],
}

/// 任务状态
#[derive(Clone, Copy, PartialEq)]
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
