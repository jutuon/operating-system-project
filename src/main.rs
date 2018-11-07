#![feature(global_asm)]

#![no_std]
#![no_main]

global_asm!(include_str!("assembly.s"));

pub mod vga_text;
pub mod terminal;

use self::terminal::{Terminal};

use core::ptr;
use core::fmt::Write;

#[no_mangle]
extern "C" fn kernel_main() -> ! {
    let mut vga_handle = vga_text::VgaTextBuffer::new().unwrap();
    let mut terminal = Terminal::new(vga_handle);

    writeln!(terminal, "Hello world");

    for _ in 0..10 {
        writeln!(terminal, "Hello World");
    }

    panic!("Hello panic");

    loop {
        unsafe {
            x86::halt()
        }
    }
}

use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut text_buffer = unsafe {
        vga_text::VgaTextBuffer::new_unsafe()
    };

    let mut terminal = Terminal::new(text_buffer);

    writeln!(terminal, "{:#?}", info);

    loop {
        unsafe {
            x86::halt()
        }
    }
}
