//! sbi调用模块

/// 从控制台输出一个字符
pub fn console_putchar(c: usize){
    #[allow(deprecated)]
    sbi_rt::legacy::console_putchar(c);
}

/// 从控制台获取一个字符
#[allow(unused)]
pub fn console_getchar() -> usize {
    #[allow(deprecated)]
    sbi_rt::legacy::console_getchar()
}

/// 设置系统时间
pub fn set_timer(timer: usize) {
    sbi_rt::set_timer(timer as _);
}

/// sbi关机接口
pub fn shutdown(failure: bool) -> ! {
    use sbi_rt::{system_reset, NoReason, Shutdown, SystemFailure};
    if !failure {
        system_reset(Shutdown, NoReason);
    } else {
        system_reset(Shutdown, SystemFailure);
    }
    unreachable!()
}