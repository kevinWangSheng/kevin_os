use core::{alloc::GlobalAlloc, future::Future, pin::Pin, ptr::null_mut, task::{Context, Poll}};

use alloc::string::String;
use fixed_size_block::FixedSizeBlockAllocator;
use linked_list::LinkedListAllocator;
use linked_list_allocator::LockedHeap;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};
pub mod fixed_size_block;
use crate::{
    bump::{self, BumpAllocator, Locked},
    println,
};

pub struct Dummy;

pub const HEAP_START: usize = 0x0000_8000_0000;
pub const HEAP_SIZE: usize = 100 * 1024;
#[global_allocator]
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(FixedSizeBlockAllocator::new());

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        panic!("dealloc should be never called")
    }
}

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_end_page = Page::containing_address(heap_end);
        let heap_start_page = Page::containing_address(heap_start);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

pub fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

mod linked_list {
    use core::{
        alloc::{GlobalAlloc, Layout},
        ptr,
    };

    use linked_list_allocator::align_down;

    use crate::bump::Locked;

    struct ListNode {
        size: usize,
        next: Option<&'static mut ListNode>,
    }

    // impl the ListNode of the base operator
    impl ListNode {
        pub const fn new(size: usize) -> Self {
            ListNode { size, next: None }
        }

        fn start_addr(&self) -> usize {
            self as *const Self as usize
        }

        fn end_addr(&self) -> usize {
            self.start_addr() + self.size
        }
    }

    pub struct LinkedListAllocator {
        head: ListNode,
    }

    impl LinkedListAllocator {
        pub const fn new() -> Self {
            LinkedListAllocator {
                head: ListNode::new(0),
            }
        }

        /// Initialize the allocator with the given heap bounds.
        ///
        /// This function is unsafe because the caller must guarantee that the given
        /// heap bounds are valid and that the heap is unused. This method must be
        /// called only once.
        pub unsafe fn init(&mut self, head_start: usize, heap_size: usize) {
            self.add_free_region(head_start, heap_size);
        }
        unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
            assert_eq!(align_down(addr, core::mem::align_of::<ListNode>()), addr);
            assert!(size >= core::mem::size_of::<ListNode>());

            let mut node = ListNode::new(size);
            node.next = self.head.next.take();
            // convert to a ListNode pointer
            let node_ptr = addr as *mut ListNode;
            node_ptr.write(node);
            self.head.next = Some(&mut *node_ptr);
        }

        fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> Result<usize, ()> {
            // check the addres is right
            let alloc_start = region.start_addr();
            let alloc_end = alloc_start.checked_add(size).ok_or(())?;

            if alloc_end > region.end_addr() {
                return Err(());
            }
            let excess_size = region.end_addr() - alloc_end;
            if excess_size > 0 && excess_size < core::mem::size_of::<ListNode>() {
                // if the alloc address is less than the region's end addr , or it remind the addr is less than one ListNode size
                // then return the err
                return Err(());
            }

            Ok(alloc_start)
        }

        fn size_align(layout: Layout) -> (usize, usize) {
            let layout = layout
                .align_to(core::mem::size_of::<ListNode>())
                .expect("adjust alignment failed")
                .pad_to_align();
            let size = layout.size().max(core::mem::size_of::<ListNode>());
            (size, layout.align())
        }
        /// Looks for a free region with the given size and alignment and removes
        /// it from the list.
        ///
        /// Returns a tuple of the list node and the start address of the allocation.
        fn find_region(
            &mut self,
            size: usize,
            align: usize,
        ) -> Option<(&'static mut ListNode, usize)> {
            // reference to current list node, update for each iteration
            let mut current = &mut self.head;

            while let Some(ref mut region) = current.next {
                if let Ok(alloc_start) = Self::alloc_from_region(&region, size, align) {
                    let next = region.next.take();
                    let ret = Some((current.next.take().unwrap(), alloc_start));
                    current.next = next;
                    return ret;
                } else {
                    current = current.next.as_mut().unwrap();
                }
            }

            None
        }
    }

    unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
        unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
            let (size, align) = LinkedListAllocator::size_align(layout);
            let mut allocator = self.lock();

            if let Some((region, alloc_start)) = allocator.find_region(size, align) {
                let alloc_end = alloc_start.checked_add(size).expect("overflow");
                let excess_size = region.end_addr() - alloc_end;
                if excess_size > 0 {
                    allocator.add_free_region(alloc_end, excess_size);
                }

                alloc_start as *mut u8
            } else {
                ptr::null_mut()
            }
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
            let (size, _) = LinkedListAllocator::size_align(layout);
            self.lock().add_free_region(ptr as usize, size);
        }
    }
}
