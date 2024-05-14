//! 实现TaskContext数据结构

/// 任务上下文
#[derive(Clone, Copy)]
#[repr(C)]
pub struct TaskContext {
    ra: usize,
    sp: usize,
    s: [usize; 12],
}

impl TaskContext {
    /// 创建一个初始为0的TaskContext
    pub fn zero_init() -> Self {
        TaskContext {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }
    /// 设置app的运行初态
    pub fn goto_restore(kernel_stack_ptr: usize) ->Self{
        extern "C" {
            fn __restore();
        }
        TaskContext {
            ra: __restore as usize,
            sp: kernel_stack_ptr,
            s: [0; 12],
        }
    } 
}
