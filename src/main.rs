
#![no_std]
#![no_main]
use core::panic::PanicInfo;

use vga_buffer::WRITER;
mod vga_buffer;
#[panic_handler]
fn panic(_info:&PanicInfo)->!{
    loop {
        
    }
}
#[allow(dead_code)]
static HELLO: &[u8] = b"Hello World!";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // let vbg_buffer = 0xb8000 as *mut u8;

    // for (i, &byte) in HELLO.iter().enumerate() {
    //     unsafe {
    //         *vbg_buffer.offset(i as isize * 2) = byte;
    //         *vbg_buffer.offset(i as isize * 2 + 1) = 0xf;
    //     }
    // }
    // vga_buffer::write_something();
    println!("hello world");
    loop {}
}