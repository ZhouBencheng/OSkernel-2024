//! Memory management implementation
//!
//! SV39 page-based virtual-memory architecture for RV64 systems, and
//! everything about memory management, like frame allocator, page table,
//! map area and memory set, is implemented here.
//!
//! Every task or process has a memory_set to control its virtual memory.

mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

pub use self::address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use self::address::{StepByOne, VPNRange};
pub use self::frame_allocator::{frame_alloc, FrameTracker};
pub use self::memory_set::remap_test;
pub use self::memory_set::{MapPermission, MemorySet, KERNEL_SPACE};
pub use self::page_table::{translated_byte_buffer, PageTableEntry};
use self::page_table::{PTEFlags, PageTable};

/// initiate heap allocator, frame allocator and kernel space
pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive_access().activate();
}
