//! 快速幂运算测试

#![no_std]
#![no_main]

use user_lib::{get_task_info, TaskInfo, MAX_SYSCALL_NUM, TaskStatus};

#[macro_use]
extern crate user_lib;

const LEN: usize = 100;

#[no_mangle]
fn main() -> i32 {
    let p = 7u64;
    let m = 998244353u64;
    let iter: usize = 160000;
    let mut s = [0u64; LEN];
    let mut cur = 0usize;
    s[cur] = 1;
    for i in 1..=iter {
        let next = if cur + 1 == LEN { 0 } else { cur + 1 };
        s[next] = s[cur] * p % m;
        cur = next;
        if i % 10000 == 0 {
            println!("power_7 [{}/{}]", i, iter);
        }
    }
    println!("{}^{} = {}(MOD {})", p, iter, s[cur], m);
    println!("Test power_7 OK!");
    let mut task_info = TaskInfo {
        status: TaskStatus::UnInit,
        call: [0; MAX_SYSCALL_NUM],
        time: 0,
    };
    get_task_info(64, &mut task_info);
    let mut f: f64 = 0.1;
    println!("Float number f = {}", f);
    f += 0.1;
    println!("Float number f = {}", f);
    println!("TaskInfo02: status = {:?}, time = {}ms", task_info.status, task_info.time);
    println!("TaskInfo02: syscall_write = {}", task_info.call[64]);
    println!("TaskInfo02: syscall_exit = {}", task_info.call[93]);
    0
}
