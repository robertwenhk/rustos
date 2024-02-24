#![no_std] // bare metal, don't use std library
#![no_main] // define our own entry

mod vga_buffer;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World from _start.");
    println!("i32 {} and float {}.", 4, 4.0 / 6.0);

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}