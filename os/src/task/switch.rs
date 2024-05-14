//! 将汇编语言的__switch函数转化为Rust接口
//! 此文件仅是Switch.S的包裹

use super::TaskContext;
use core::arch::global_asm;

global_asm!(include_str!("switch.S"));

extern "C" {
    /// 保存当前上下文至其PCB，切换到下一个上下文
    pub fn __switch(current_task: *mut TaskContext, next_task: *const TaskContext);
}
