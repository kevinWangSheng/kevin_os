#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]
pub mod interrupts;
pub mod serial;
pub mod vga_buffer;
pub mod memory;
pub mod alloctor;
use core::panic::PanicInfo;
pub mod gdt;
#[cfg(test)]
use bootloader::{entry_point, BootInfo};
use vga_buffer::{BUFFER_HEIGHT, WRITER};
extern crate alloc;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum QemuExitCode {
    Succeed = 0x010,
    Failed = 0x11,
}
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    interrupts::init_pic();
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Succeed);
}

#[cfg(test)]
entry_point!(test_kernel_main);

#[cfg(test)]
pub fn test_kernel_main(_boot_info:&'static BootInfo)->!{
    init();
    test_main();
    hlt_loop();
}
// #[cfg(test)]
// #[no_mangle]
// pub extern "C" fn _start() -> ! {
//     // init the breakpoint exception


//     init();
//     test_main();
//     hlt_loop();
// }

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_print!("[Faild]\n");
    serial_println!("Error:{}", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

pub fn hlt_loop() ->!{
    loop {
        x86_64::instructions::hlt();
    }
}

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_println!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[OK]");
    }
}
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[test_case]
fn test_simple_print() {
    println!("print simple !!");
}

#[test_case]
fn test_second() {
    println! {"test second"};
}

#[test_case]
fn println_many() {
    // for _ in 0..200{
    //     println!("test manyn");
    // }
}

#[test_case]
fn test_println_output() {
    let s = "Some test string that fits on a single line";
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).unwrap();
        for (i, c) in s.chars().enumerate() {
            let screen_character = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_character.ascii_character), c);
        }
    });
}
#[test_case]
fn trivial_assertion() {
    serial_println!("trivial assertion test");
    assert_eq!(1, 1);
    serial_print!("[OK]");
}
