use core::{alloc::GlobalAlloc, ptr::null_mut};

use linked_list_allocator::LockedHeap;
use x86_64::{structures::paging::{mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB}, VirtAddr};

use crate::println;


pub struct Dummy;

pub const HEAP_START:usize = 0x0000_8000_0000;
pub const HEAP_SIZE:usize = 100 * 1024;
#[global_allocator]
static ALLOCATOR:LockedHeap = LockedHeap::empty();

unsafe impl GlobalAlloc for Dummy{
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        panic!("dealloc should be never called")
    }
}

pub fn init_heap(mapper:&mut impl Mapper<Size4KiB>,frame_allocator:&mut impl FrameAllocator<Size4KiB>)->Result<(),MapToError<Size4KiB>>{

    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE -1u64;
        let heap_end_page = Page::containing_address(heap_end);
        let heap_start_page = Page::containing_address(heap_start);
        Page::range_inclusive(heap_start_page,heap_end_page)
    };

    

    for page in page_range{
        let frame = frame_allocator.allocate_frame().ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush()
        };
    }
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}