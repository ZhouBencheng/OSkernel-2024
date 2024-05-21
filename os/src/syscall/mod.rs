//! 实现系统调用
//!
//! 这个单独的系统调用抽象入口适用于实现所有类型的系统调用，只要用户空间
//! 想要用`ecall`指令执行系统调用。在这种情况下，处理器会引发一个“来自U模式的环境调用”
//! 异常，这个异常会被作为[`crate::trap::trap_handler`]中的一个case来处理。
//! 
//! 为了清晰起见，每个单独的系统调用都被实现为自己的函数，命名为`sys_`然后是系统调用的名称。
//! 你可以在子模块中找到这样的函数，你也应该用这种方式实现系统调用。

const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_SBRK: usize = 214;

mod fs;
mod process;

use self::fs::*;
use self::process::*;

/// 处理通用的所有系统调用，这里是所有系统调用的最高抽象入口
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {

    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(),
        SYSCALL_SBRK => sys_sbrk(args[0] as i32),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
} 
