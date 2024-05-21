//! 任务管理模块

use super::TaskContext;
use crate::config::{kernel_stack_position, TRAP_CONTEXT};
use crate::mm::{MapPermission, MemorySet, PhysPageNum, VirtAddr, KERNEL_SPACE};
use crate::trap::{trap_handler, TrapContext};

/// 任务控制块
pub struct TaskControlBlock {
    /// 任务状态
    pub task_status: TaskStatus,
    /// 任务上下文
    pub task_cx: TaskContext,
    /// 用户地址空间
    pub memory_set: MemorySet,
    /// Trap上下文物理页号
    pub trap_cx_ppn: PhysPageNum,
    /// 应用数据大小，从0x0到用户栈结束地址的大小
    pub base_size: usize,
    /// 堆底地址
    pub heap_bottom: usize,
    /// 
    pub program_brk: usize,
}

impl TaskControlBlock {
    /// 获取当前正在运行的应用程序地址空间中的TrapContext指针
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    /// 获取当前正在运行的应用程序地址空间的token
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
    /// 创建一个新的任务控制块
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        // 根据传入的elf数据构造应用的地址空间，包括跳板页、Trap上下文页、用户栈
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        // 通过多级页表找到应用地址空间中的Trap上下文实际的物理页号
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        let task_status = TaskStatus::Ready;
        // 获取应用的内核栈预计在内核空间中的位置
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(app_id);
        // 在内核地址空间中插入该逻辑段
        KERNEL_SPACE.exclusive_access().insert_framed_area(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            MapPermission::R | MapPermission::W,
        );
        let task_control_block = Self {
            task_status,
            task_cx: TaskContext::goto_trap_return(kernel_stack_top),
            memory_set,
            trap_cx_ppn,
            base_size: user_sp,
            heap_bottom: user_sp,
            program_brk: user_sp,
        };
        // 获取指向当前应用TrapContext的可变引用
        let trap_cx = task_control_block.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        task_control_block
    }
    /// change the location of the program break. return None if failed.
    pub fn change_program_brk(&mut self, size: i32) -> Option<usize> {
        let old_break = self.program_brk;
        let new_brk = self.program_brk as isize + size as isize;
        if new_brk < self.heap_bottom as isize {
            return None;
        }
        let result = if size < 0 {
            self.memory_set
                .shrink_to(VirtAddr(self.heap_bottom), VirtAddr(new_brk as usize))
        } else {
            self.memory_set
                .append_to(VirtAddr(self.heap_bottom), VirtAddr(new_brk as usize))
        };
        if result {
            self.program_brk = new_brk as usize;
            Some(old_break)
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
/// task status:Ready, Running, Exited
pub enum TaskStatus {
    /// 就绪态
    Ready,
    /// 运行态
    Running,
    /// 退出态
    Exited,
}
