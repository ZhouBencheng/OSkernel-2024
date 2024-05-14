//! m模式时钟寄存器操作接口

use riscv::register::time;
use crate::config::CLOCK_FREQ;
use crate::sbi::set_timer;

/// 获取当前时间,读取mtime寄存器
pub fn get_time() -> usize {
    time::read()
}

const MSEC_PER_SEC: usize = 1000;

/// 获取当前时间到毫秒数
pub fn get_time_ms() -> usize {
    time::read() / (CLOCK_FREQ / MSEC_PER_SEC)
}

const TICKS_PER_SEC: usize = 100;

/// 设置下一次始终中断触发时间，即mtimecmp寄存器的值
pub fn set_next_trigger() {
    /* 时间片轮转 
     * 注意该函数并没有直接永久的设置一个时间片的大小 
     * 仅仅只是设置下一次中断的时间
     * 因为每次在Trap_handler中match一次时钟中断时，就调用一次set_next_trigger()来设置周期性时间片*/
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}
