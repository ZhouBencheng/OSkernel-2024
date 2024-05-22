//! 批处理子系统
//! 批处理系统能够轮流加载执行多个应用程序的原理在于：
//! 1. 能够成功运行的应用程序， 比如app_0，在最后main函数返回0时自动调用sys_exit()系统调用，并执行run_next_app()函数
//! 2. 应用程序运行错误，特权级切换并被trap_handler()捕获，输出相应错误信息后，调用run_next_app()函数

use log::*;

use crate::sbi::shutdown;
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use core::arch::asm;

use lazy_static::*;

const USER_STACK_SIZE: usize = 4096 * 2;
const KERNEL_STACK_SIZE: usize = 4096 * 2;
const MAX_APP_NUM: usize = 16;
const APP_BASE_ADDRESS:usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;

//定义用户栈
#[repr(align(4096))]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

//定义内核栈
#[repr(align(4096))]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

//声明内核栈并初始化
static KERNEL_STACK: KernelStack = KernelStack {
    data: [0; KERNEL_STACK_SIZE],
};

//声明用户栈并初始化
static USER_STACK: UserStack = UserStack {
    data: [0; USER_STACK_SIZE],
};

impl KernelStack {
    //获取内核栈栈顶指针
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    /* core::mem::size_of::<TrapContext>()解释
     * core::mem::size_of::<TrapContext>()这行代码使用了Rust的size_of函数
     * 这个函数返回一个类型在内存中占用的字节数
     * ::<TrapContext>()是一个类型参数 （类型参数需要使用::和函数进行联系）
     * 表示我们想要获取TrapContext类型的大小 */
    //将TrapContextTrap上下文压入内核栈
    pub fn push_context(&self, cx: TrapContext) -> &'static mut TrapContext {
        let cx_ptr: *mut TrapContext = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *cx_ptr = cx;
        }
        unsafe { cx_ptr.as_mut().unwrap() }
    }
}

impl UserStack {
    //获取用户栈栈顶指针
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

//? 以上代码定义内核栈和用户栈，以及其方法； 以下代码实现AppManager结构体

struct AppManager {
    num_app: usize, //app数量
    current_app: usize, //当前app编号
    app_start: [usize; MAX_APP_NUM + 1], // 每一个app的起始地址
    // 根据trap.S文件中app数组的定义，最后一个元素存放最后一个app的结束地址
}

impl AppManager {
    // 打印所app数量信息和每一个app的起止地址
    pub fn print_app_info(&self) {
        println!("[kernel] num_app = {}", self.num_app);
        for i in 0..self.num_app {
            println!(
                "[kernel] app_{} [ {:#x}, {:#x} )",
                i,
                self.app_start[i],
                self.app_start[i + 1] // 根据objdump显示，下一个app的start地址就是上一个app的end地址
            );
        }
    }
    // 加载app到约定app运行地址，准备执行
    unsafe fn load_app(&self, app_id: usize) {
        if app_id >= self.num_app {
            println!("[kernel] All applications completed!");
            shutdown(false);
        }
        println!("[kernel] Loading app_{}...", app_id);
        // 获取APP_BASE_ADDRESS地址为起始，APP_SIZE_LIMIT为字节长度的可变切片，并填充0
        core::slice::from_raw_parts_mut(
            APP_BASE_ADDRESS as *mut u8,
             APP_SIZE_LIMIT
        ).fill(0);
        // 获取app_id对应的app的所在内存的字节切片
        let app_src = core::slice::from_raw_parts(
            self.app_start[app_id] as *const u8,
            self.app_start[app_id + 1] - self.app_start[app_id],
        );
        // 获取APP_BASE_ADDRESS地址为起始的可变切片，该切片长度为app_src，但此时切片内容为归零字节
        let app_dst = core::slice::from_raw_parts_mut(
            APP_BASE_ADDRESS as *mut u8, 
            app_src.len()
        );
        // 将app_src的内容复制到app_dst
        app_dst.copy_from_slice(app_src);
        trace!("load app_{} done", app_id);
        asm!("fence.i");
    }

    pub fn get_current_app(&self) -> usize {
        self.current_app
    }

    pub fn move_to_next_app(&mut self) {
        self.current_app += 1;
    }
}

/* lazy_static宏的解释
 * 该宏用于创建只在首次访问时初始化的静态变量
 * 由于我们的app数量和各app起止地址需要在运行时，根据trap.S代码确定，因此推迟APP_MANAGER的初始化
 * 推迟的初始化中，我们调用UPSafeCell类型的关联函数::new(value: T)进行初始化 */
lazy_static! {
    static ref APP_MANAGER: UPSafeCell<AppManager> = unsafe {
        UPSafeCell::new({
            extern "C" {
                fn _num_app();
            }
            let num_app_ptr = _num_app as usize as *const usize;
            let num_app = num_app_ptr.read_volatile();
            let mut app_start: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
            let app_start_raw: &[usize] = core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1);
            app_start[..=num_app].copy_from_slice(app_start_raw);
            AppManager {
                num_app,
                current_app: 0,
                app_start,
            }
        })
    };
}

/// 批处理系统初始化，也是APP_MANAGER的初始化，应验了lazy_static!宏的首次访问初始化特性
pub fn init() {
    print_app_info();
}

/// 被init()调用的打印函数
pub fn print_app_info() {
    APP_MANAGER.exclusice_access().print_app_info();
}

/* APP_MANAGER工作流程
 * 1. 获取AppManager实例
 * 2. 获取当前app编号
 * 3. 加载当前app
 * 4. 更新当前app编号（准备下一次加载）
 * 5. 释放AppManager */
/// 加载并运行下一个app
 pub fn run_next_app() -> ! {
    let mut app_manager = APP_MANAGER.exclusice_access();
    let current_app = app_manager.get_current_app();
    unsafe {
        app_manager.load_app(current_app);
    }
    app_manager.move_to_next_app();
    drop(app_manager);

    extern "C" {
        fn __restore(cx_addr: usize);
    }
    unsafe {
        __restore(KERNEL_STACK.push_context(TrapContext::app_init_context(
            APP_BASE_ADDRESS,
            USER_STACK.get_sp(),
        )) as *const _ as usize);
    }
    panic!("Unreachable in batch::run_current_app!");
}
