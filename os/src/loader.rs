//! # loader模块职责
//! - 我每个应用程序创建独立的内核栈和用户栈
//! - 已知应用程序在编译时静态加载在内核程序数据段中，loader模块负责将所有应用程序一次性加载到‘约定’的地址中
//! - 提供‘应用程序trap上下文初始化函数’`init_app_cx`的接口
//! - 提供’应用程序总数查询函数’`get_app_num`接口

use crate::config::*;
use crate::trap::TrapContext;
use core::arch::asm;

#[repr(align(4096))]
#[derive(Copy, Clone)]
struct KernelStack {
    data:[u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
#[derive(Copy, Clone)]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

/// 为每个用户程序分配一个内核栈
static KERNEL_STACK: [KernelStack; MAX_APP_NUM] = [KernelStack {
    data: [0; KERNEL_STACK_SIZE],
}; MAX_APP_NUM];

/// 为每个用户程序分配一个用户栈
static USER_STACK: [UserStack; MAX_APP_NUM] = [UserStack {
    data: [0; USER_STACK_SIZE],
}; MAX_APP_NUM];

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    fn push_context(&self, trap_cx: TrapContext) -> usize {
        let trap_cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *trap_cx_ptr = trap_cx;
        }
        trap_cx_ptr as usize
    }
}

impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

/// 获取当前app的起始运行地址
fn get_base_i(app_id: usize) -> usize {
    APP_BASE_ADDRESS + app_id * APP_SIZE_LIMIT
}

/// 获得当前app总数
pub fn get_app_num() -> usize {
    extern "C" {
        fn _num_app();
    }
    unsafe { (_num_app as usize as *const usize).read_volatile() }
}

/// 从内核数据段加载所有静态链接的用户程序
pub fn load_app() {
    extern "C" {
        fn _num_app();
    }
    let num_app = get_app_num();
    let num_app_ptr = _num_app as usize as *const usize;
    // 获取静态链接app的起始地址数组切片
    let app_start = unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) };
    for i in 0..num_app {
        let base_i = get_base_i(i);
        // 强行清空app的预运行空间
        (base_i..base_i + APP_SIZE_LIMIT).for_each(|addr| {
            unsafe {
                (addr as *mut u8).write_volatile(0);
            }
        });
        // 获取静态链接app的字节数组切片
        let src = unsafe { core::slice::from_raw_parts(app_start[i] as *const u8, app_start[i + 1] - app_start[i]) };
        // 获取app预运行地址的字节切片
        let dst = unsafe { core::slice::from_raw_parts_mut(base_i as *mut u8, src.len()) };
        dst.copy_from_slice(src);
    }
    unsafe { asm!("fence.i"); }
}

/// 初始化app的trap上下文
pub fn init_app_cx(app_id: usize) -> usize {
    KERNEL_STACK[app_id].push_context(TrapContext::app_init_context(
        get_base_i(app_id),
        USER_STACK[app_id].get_sp(),
    ))
}
