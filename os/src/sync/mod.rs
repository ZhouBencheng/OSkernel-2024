//! 提供UPSafeCell类型，用于提供一个全局共享并且线程安全的变量

mod up;
pub use self::up::UPSafeCell;