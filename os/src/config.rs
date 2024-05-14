//! 设置全局使用的常量

pub const USER_STACK_SIZE: usize = 4096;
pub const KERNEL_STACK_SIZE: usize = 4096 *2;
pub const APP_BASE_ADDRESS: usize = 0x80400000;
pub const APP_SIZE_LIMIT: usize = 0x20000;
pub const MAX_APP_NUM: usize = 4;

// 本操作系统采用qemu模拟器运行，此处记录qemu的时钟频率
pub use crate::board::CLOCK_FREQ;
