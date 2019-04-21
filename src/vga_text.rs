
use core::sync::atomic::{AtomicBool, Ordering};
use core::slice;
use core::fmt;

static VGA_HANDLE_CREATED: AtomicBool = AtomicBool::new(false);

use vga::{
    io::{ PortIo, StandardVideoRamLocation },
    driver::text::TextMode,
};

pub struct VgaPortIo;

impl PortIo for VgaPortIo {
    fn read(&mut self, port: u16) -> u8 {
        unsafe {
            x86::io::inb(port)
        }
    }

    fn write(&mut self, port: u16, data: u8) {
        unsafe {
            x86::io::outb(port, data)
        }
    }
}

pub type VgaTextMode = TextMode<VgaPortIo, StandardVideoRamLocation>;

pub unsafe fn new_vga_text_mode_unsafe() -> VgaTextMode {
    TextMode::new(VgaPortIo, StandardVideoRamLocation::new() )
}

pub fn new_vga_text_mode() -> Option<VgaTextMode> {
    if VGA_HANDLE_CREATED.compare_and_swap(false, true, Ordering::SeqCst) {
        None
    } else {
        let text_mode = unsafe { new_vga_text_mode_unsafe() };
        Some(text_mode)
    }
}
