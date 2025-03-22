use core::{alloc::GlobalAlloc, ptr};

use linked_list_allocator::align_up;



pub struct BumpAllocator{
    heap_start:usize,
    heap_end:usize,
    next:usize,
    allocations:usize
}


impl BumpAllocator{
    pub const fn new()->Self{
        BumpAllocator{
            heap_start:0,
            heap_end:0,
            next:0,
            allocations:0
        }
    }

    pub fn init(&mut self,heap_start:usize,heap_size:usize){
        self.heap_start = heap_start;
        self.heap_end = self.heap_start+heap_size;
        self.next = heap_start;
    }
}

pub struct Locked<A>{
    inner : spin::Mutex<A>
}

impl <A> Locked<A>{
    pub const fn new(inner:A)->Self{
        Locked{
            inner:spin::Mutex::new(inner)
        }
    }

    pub fn lock(&self)->spin::MutexGuard<A>{
        self.inner.lock()
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    // allocate the memory with the wrapper lock
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut dump = self.lock();

        let alloc_start =  align_up(dump.next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()){
            Some(end)=>end,
            None=>return ptr::null_mut(),
        };

        if alloc_end > dump.heap_end{
            ptr::null_mut()
        }else{
            dump.next = alloc_end;
            dump.allocations+=1;
            alloc_start as * mut u8
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let mut bump = self.lock();
        bump.allocations-=1;
        if bump.allocations == 0{
            bump.next = bump.heap_start;
        }
    }
}