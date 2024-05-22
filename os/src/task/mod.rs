//! TaskManager任务管理器实现
//! 
//! 任务管理工作：1. 任务的首次创建 2. 任务的切换 


mod context;
mod switch;

#[allow(clippy::module_inception)]
mod task;

use crate::loader::{get_app_data, get_num_app};
use crate::sbi::shutdown;
use crate::sync::UPSafeCell;
use lazy_static::*;
use crate::trap::TrapContext;
use self::switch::__switch;
use alloc::vec::Vec;

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
    /// TCB列表
    tasks: Vec<TaskControlBlock>,
    /// 当前运行的任务id
    current_task: usize,
}

lazy_static! {
    /// 全局变量：任务管理器
    pub static ref TASK_MANAGER: TaskManager = {
        println!("init TASK_MANAGER");
        let num_app = get_num_app();
        println!("num_app = {}", num_app);
        let mut tasks: Vec<TaskControlBlock> = Vec::new();
        for i in 0..num_app {
            println!("Begin load TCB{}", i);
            tasks.push(TaskControlBlock::new(get_app_data(i), i));
            println!("End load TCB{}", i);
        }
        println!("Successfully initialize the TrakControlBlock Vector.");
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
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
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        println!("Begin to run the first app.");
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    /// 挂起当前任务
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current_task_id = inner.current_task;
        // println!("task {} suspended", current_task_id);
        inner.tasks[current_task_id].task_status = TaskStatus::Ready;
    }

    /// 退出当前任务
    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current_task_id = inner.current_task;
        // println!("task {} exited", current_task_id);
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
    
    /// 获取当前正在运行的应用程序地址空间的token
    fn get_current_token(&self) -> usize {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].get_user_token()
    }

    /// 获取当前正在运行的应用程序的TrapContext可变引用
    fn get_current_trap_cx(&self) -> &'static mut TrapContext {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].get_trap_cx()
    }

    /// 改变当前正在运行应用程序的program break
    pub fn change_current_program_brk(&self, size: i32) -> Option<usize> {
        let mut inner = self.inner.exclusive_access();
        let cur = inner.current_task;
        inner.tasks[cur].change_program_brk(size)
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
/// 获取当前正在运行的应用程序地址空间的token
pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}

/// 获取当前正在运行的应用程序的TrapContext可变引用
pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}

/// 改变当前正在运行应用程序的program break
pub fn change_program_brk(size: i32) -> Option<usize> {
    TASK_MANAGER.change_current_program_brk(size)
}
