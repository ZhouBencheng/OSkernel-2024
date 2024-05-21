//! 本文件用于配置qemu模拟器的硬件参数

pub const CLOCK_FREQ:usize = 12500000; //时钟频率
pub const MEMORY_END: usize = 0x8800_0000;

pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
];