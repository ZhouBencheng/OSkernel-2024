//! TaskManager任务管理器实现
//! 
//! 任务管理工作：1. 任务的首次创建 2. 任务的切换 


mod context;
mod switch;

#[allow(clippy::module_inception)]
mod task;

use crate::config::MAX_APP_NUM;
use crate::loader::{get_app_num, init_app_cx};
use crate::sbi::shutdown;
use crate::sync::UPSafeCell;
use crate::timer::get_time_ms;
use lazy_static::*;
use self::switch::__switch;
use crate::config::MAX_SYSCALL_NUM;

pub use self::task::{TaskControlBlock, TaskStatus};
pub use self::context::TaskContext;

/// 任务管理器
pub struct TaskManager {
    /// app总数
    num_app: usize,
    /// 使用互斥借用容器获取任务管理器的可变借用
    inner: UPSafeCell<TaskManagerInner>,
}

/// 任务管理器内部
pub struct TaskManagerInner {
    /// 当前运行的任务id
    current_task: usize,
    /// 停表
    stop_watch: usize,
    /// TCB列表
    tasks: [TaskControlBlock; MAX_APP_NUM],
}

impl TaskManagerInner {
    fn refresh_stop_watch(&mut self) -> usize {
        let start_time = self.stop_watch;
        self.stop_watch = get_time_ms();
        self.stop_watch - start_time
    }
}

lazy_static! {
    /// 全局变量：任务管理器
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_app_num();
        let mut tasks = [TaskControlBlock {
            task_status: TaskStatus::UnInit,
            task_cx: TaskContext::zero_init(),
            kernel_time: 0,
            user_time: 0,
            syscall_info: [0; MAX_SYSCALL_NUM],
        }; MAX_APP_NUM];
        for (i, task) in tasks.iter_mut().enumerate() {
            task.task_status = TaskStatus::Ready;
            task.task_cx = TaskContext::goto_restore(init_app_cx(i));
        }
        TaskManager {
            num_app,
            inner: unsafe{
                UPSafeCell::new(TaskManagerInner {
                    current_task: 0,
                    stop_watch: 0,
                    tasks,
                })
            }, 
        }
    };
}

impl TaskManager {
    /// 运行第一个任务
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        inner.refresh_stop_watch();
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    /// 挂起当前任务
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current_task_id = inner.current_task;
        // println!("Task {} suspended.", current_task_id);
        // 统计内核时间
        inner.tasks[current_task_id].kernel_time += inner.refresh_stop_watch();
        inner.tasks[current_task_id].task_status = TaskStatus::Ready;
    }

    /// 退出当前任务
    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current_task_id = inner.current_task;
        // println!("Task {} exited.", current_task_id);
        // 统计内核时间并输出
        inner.tasks[current_task_id].kernel_time += inner.refresh_stop_watch();
        println!("[task {} exited. user_time: {} ms, kernel_time: {} ms.", 
                    current_task_id, inner.tasks[current_task_id].user_time, 
                    inner.tasks[current_task_id].kernel_time
                );
        inner.tasks[current_task_id].task_status = TaskStatus::Exited;
    }

    /// 用Josephu环寻找下一个处于就绪态的任务
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current_task_id = inner.current_task;
        (current_task_id + 1..current_task_id + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }
    
    /// 运行下一个就绪态任务
    fn run_next_task(&self) {
        if let Some(next_task_id) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            // println!("task {} start", current);
            inner.tasks[next_task_id].task_status = TaskStatus::Running;
            inner.current_task = next_task_id;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next_task_id].task_cx as *const TaskContext;
            drop(inner);
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            println!("All applications completed, shutdown!");
            shutdown(false);
        }
    }

    /// 统计内核时间，现在开始算用户时间
    fn user_time_start(&self) {
        let mut inner = self.inner.exclusive_access();
        let current_task_id = inner.current_task;
        inner.tasks[current_task_id].kernel_time += inner.refresh_stop_watch();
    }

    /// 统计用户时间，现在开始算内核时间
    fn user_time_end(&self) {
        let mut inner = self.inner.exclusive_access();
        let current_task_id = inner.current_task;
        inner.tasks[current_task_id].user_time += inner.refresh_stop_watch();
    }

    /// 更新系统调用次数
    fn update_syscall_info(&self, syscall_id: usize) {
        let mut inner = self.inner.exclusive_access();
        let current_task_id = inner.current_task;
        inner.tasks[current_task_id].syscall_info[syscall_id] += 1;
    }

    /// 获取系统调用次数
    fn get_syscall_info(&self) -> [usize; MAX_SYSCALL_NUM] {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].syscall_info
    }

    /// 获取当前任务运行时间(内核时间+用户时间)
    fn get_total_time(&self) -> usize {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].kernel_time + inner.tasks[inner.current_task].user_time
    }
}

/// 运行第一个任务
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

/// 运行下一个任务
fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

/// 挂起当前任务
fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

/// 退出当前任务
fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

/// 挂起当前任务，并运行下一个任务
pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

/// 退出当前任务，并运行下一个任务
pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

/// 统计内核时间，现在开始算用户时间
pub fn user_time_start() {
    TASK_MANAGER.user_time_start();
}

/// 统计用户时间，现在开始算内核时间
pub fn user_time_end() {
    TASK_MANAGER.user_time_end();
}

/// 获取系统调用次数
pub fn get_syscall_info() -> [usize; MAX_SYSCALL_NUM] {
    TASK_MANAGER.get_syscall_info()
}

/// 更新系统调用次数
pub fn update_syscall_info(syscall_id: usize) {
    TASK_MANAGER.update_syscall_info(syscall_id);
}

/// 获取当前任务运行时间(内核时间+用户时间)
pub fn get_total_time() -> usize {
    TASK_MANAGER.get_total_time()
}
