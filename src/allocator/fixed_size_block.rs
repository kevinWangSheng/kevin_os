use core::{alloc::{GlobalAlloc, Layout}, ptr::{self, NonNull}};

use alloc::alloc;
use linked_list_allocator::Heap;

use crate::bump::Locked;



const BLOCK_SIZE:&[usize] = &[8,16,32,63,128,256,512,1024,2048];
struct ListNode{
    next:Option<& 'static mut ListNode> 
}


struct  Node<'a>{
    next: &'a Node<'a>
}

pub struct FixedSizeBlockAllocator{
    // the different size of the meomory to allocate 
    list_head:[Option<&'static mut ListNode>;BLOCK_SIZE.len()],
    // the fallback allocator to allocate the memory that more than the largest size of list_head
    fallback_allocator: linked_list_allocator::Heap
}

impl FixedSizeBlockAllocator {
    pub const fn new()->Self{
        // create a empty FixedSizeBlockAllocator
        const EMPTY:Option<&'static mut ListNode> = None;
        FixedSizeBlockAllocator{
            list_head:[EMPTY;BLOCK_SIZE.len()],
            // create a empty linked_list_allocator
            fallback_allocator: linked_list_allocator::Heap::empty()
        }
    }

    /// Initialize the allocator with the given heap bounds.
    /// 
    /// This function is unsfae because the caller must guarantee that the given heap bounds are vaild
    pub unsafe fn init(&mut self,heap_start:usize,heap_size:usize){
        self.fallback_allocator.init(heap_start, heap_size);
    }

    fn fallback_alloc(&mut self,layout:Layout)-> *mut u8{
        match self.fallback_allocator.allocate_first_fit(layout){
            Ok(ptr)=>ptr.as_ptr(),
            Err(_)=>ptr::null_mut()
        }
    }
}

/// choose an appropriate block size for the given layout
/// 
/// Retruns an index into the `BLOCK_SIZES` array
fn list_index(layout:&Layout)->Option<usize>{
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZE.iter().position(|&s| s>=required_block_size)
}

unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator>{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();
        match list_index(&layout){
            Some(index)=>{
                match allocator.list_head[index].take() {
                    Some(head)=>{
                        allocator.list_head[index] = head.next.take();
                        head as * mut ListNode as *mut u8
                    },
                    None=>{
                        let block_size = BLOCK_SIZE[index];
                        let block_align = block_size;
                        let layout = Layout::from_size_align(block_size, block_align).unwrap();
                        allocator.fallback_alloc(layout)
                    }
                }
            },
            None=>
                allocator.fallback_alloc(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();
        match list_index(&layout) {
            Some(index)=>{
                let new_node = ListNode{
                    next:allocator.list_head[index].take()
                };

                assert!(core::mem::size_of::<ListNode>() <= BLOCK_SIZE[index]);
                assert!(core::mem::align_of::<ListNode>() <= BLOCK_SIZE[index]);

                let new_node_ptr = ptr as * mut ListNode;
                new_node_ptr.write(new_node);
                allocator.list_head[index] = Some(&mut *new_node_ptr);
            },
            None=>{
                let ptr = NonNull::new(ptr).unwrap();
                allocator.fallback_allocator.deallocate(ptr, layout);
            }
        }
    }
}



