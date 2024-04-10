//! 实现UPSafeCell<T>结构体，用于实现单处理器上的安全的互斥访问

use core::cell::{RefCell, RefMut};

pub struct UPSafeCell<T> {
    inner: RefCell<T>,
}

/* sync解释
 * sync是一种标记特质（marker trait），表示这个类型是线程安全的（thread-safe）
 * 由于我们要声明static性质的AppManager，所以需要保证AppManager是线程安全的
 * 因此在这里实现空的sync<T>特征来避开编译器线程安全检查
 * */
unsafe impl<T> Sync for UPSafeCell<T> {}

impl<T> UPSafeCell<T> {
    // UPSafeCell构造器，用户需要自己确定内部结构体运行在单处理器上，否则会出现数据竞争
    pub unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }
    // 互斥获取<T> -> AppManager 不可变引用的方法
    pub fn exclusice_access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}
