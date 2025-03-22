#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kevin_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use alloc::{boxed::Box, vec::Vec};
use bootloader::{entry_point, BootInfo};
use kevin_os::{
    allocator::{self, HEAP_SIZE},
    memory::{self, BootInfoFrameAllocator},
};
use x86_64::VirtAddr;

extern crate alloc;

entry_point!(main);
fn main(_boot_info: &'static BootInfo) -> ! {
    kevin_os::init();

    let phys_mem_offset = VirtAddr::new(_boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&_boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initailization failed");

    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kevin_os::test_panic_handler(info)
}


#[test_case]
fn simple_allocation(){
    let heap_value1 = Box::new(12);
    let heap_value2 = Box::new(23);
    assert_eq!(*heap_value1,12);
    assert_eq!(*heap_value2,23);
}

#[test_case]
fn large_vec(){
    let n = 100;
    let mut vec = Vec::new();
    for i in 0..n{
        vec.push(i);
    }

    assert_eq!(vec.iter().sum::<u64>(),(n-1)*n/2)
}

#[test_case]
fn many_boxes(){
    for i in 0.. 10{
        let x = Box::new(i);
        assert_eq!(*x,i);
    }
}

#[test_case]
fn many_boxes_long_lived(){
    for i in 0.. HEAP_SIZE{
        let x = Box::new(i);
        assert_eq!(*x,i);
    }
}