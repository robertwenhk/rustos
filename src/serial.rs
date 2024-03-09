use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;
use x86_64::instructions::interrupts::without_interrupts;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        // 0x3f8: standard port number for the first serial interface
        let mut serial_port = unsafe { SerialPort::new(0x3f8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    without_interrupts(|| {
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    });
}

// Print to the host through the serial interface
#[macro_export]
macro_rules!  serial_print {
    ($($args:tt)*) => {
        $crate::serial::_print(format_args!($($args)*))
    };
}

// Print to the host through the serial interface and append a newline
#[macro_export]
macro_rules! serial_println {
    () => {$crate::serial_print!("\n")};
    ($fmt:expr) => {$crate::serial_print!(concat!($fmt, "\n"))};
    ($fmt:expr, $($arg:tt)*) => {$crate::serial_print!(concat!($fmt, "\n"), $($arg)*)};
}