#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kevin_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
mod serial;
use core::{panic::PanicInfo};

use bootloader::{entry_point, BootInfo};
use kevin_os::memory::{self, active_level_4_table, translate_addr};
use x86_64::{structures::paging::{Page, PageTable, Translate}, VirtAddr};

// use vga_buffer::{BUFFER_HEIGHT, WRITER};
// use x86_64::structures::idt::Entry;
mod vga_buffer;

#[panic_handler]
#[cfg(not(test))]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    kevin_os::hlt_loop();
}
#[allow(dead_code)]
static HELLO: &[u8] = b"Hello World!";

entry_point!(kernel_main);

fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    // let vbg_buffer = 0xb8000 as *mut u8;

    // for (i, &byte) in HELLO.iter().enumerate() {
    //     unsafe {
    //         *vbg_buffer.offset(i as isize * 2) = byte;
    //         *vbg_buffer.offset(i as isize * 2 + 1) = 0xf;
    //     }
    // }
    // vga_buffer::write_something();
    use kevin_os::memory::BootInfoFrameAllocator;
    use kevin_os::memory::translate_addr;
    println!("hello world!!");
    kevin_os::init();

    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&_boot_info.memory_map)
    };
    
    let phy_mem_offset = VirtAddr::new(_boot_info.physical_memory_offset);
    let mut mapper = unsafe {
        memory::init(phy_mem_offset)
    };
    let page = Page::containing_address(VirtAddr::new(0xbeaf000));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();

    unsafe {page_ptr.offset(300).write_volatile(0xf021_f077_f065_f04e);}

    // pub fn stack_overflow(){
    //     stack_overflow();
    // }
    // trigger a stack overflow
    // stack_overflow();
    // x86_64::instructions::interrupts::int3();
    println!("It did not crash!");
    // loop {
    //     use kevin_os::print;
    //     print!("-");
    // }
    #[cfg(test)]
    test_main();
    kevin_os::hlt_loop();
}

// #[no_mangle]
// pub extern "C" fn _start(_boot_info:&'static BootInfo) -> ! {
//     // let vbg_buffer = 0xb8000 as *mut u8;

//     // for (i, &byte) in HELLO.iter().enumerate() {
//     //     unsafe {
//     //         *vbg_buffer.offset(i as isize * 2) = byte;
//     //         *vbg_buffer.offset(i as isize * 2 + 1) = 0xf;
//     //     }
//     // }
//     // vga_buffer::write_something();
//     use x86_64::registers::control::Cr3;
//     println!("hello world!!");
//     kevin_os::init();

//     let phy_mem_offset = VirtAddr::new(_boot_info.physical_memory_offset);
//     let l4_table = unsafe {
//         active_level_4_table(phy_mem_offset)
//     };
//     for(i,entry) in l4_table.iter().enumerate(){
//         if !entry.is_unused(){
//             println!("L4 Entry:{}:{:?}",i,entry);
//         }
//     }

//     fn stack_overflow(){
//         stack_overflow();
//     }
//     // trigger a stack overflow
//     // stack_overflow();
//     // x86_64::instructions::interrupts::int3();
//     println!("It did not crash!");
//     // loop {
//     //     use kevin_os::print;
//     //     print!("-");
//     // }
//     #[cfg(test)]
//     test_main();
//     kevin_os::hlt_loop();
// }

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use kevin_os::test_panic_handler;

    test_panic_handler(info);
}

#[test_case]
pub fn test_main() {
    serial_print!("main test");
}
