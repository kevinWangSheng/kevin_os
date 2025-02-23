use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

lazy_static! {
    pub static ref SERIAL: Mutex<uart_16550::SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

pub fn print_(args: core::fmt::Arguments) {
    use fmt::Write;
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        SERIAL
            .lock()
            .write_fmt(args)
            .expect("Print to serial faild");
    });
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::print_(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
        ()=>($crate::serial_print!("\n"));
        ($fmt:expr)=>($crate::serial_print!(concat!($fmt,"\n")));
        ($fmt:expr,$($arg:tt)*)=>($crate::serial_print!(concat!($fmt,"\n"),$($arg)*));
}
