
use core::sync::atomic::{AtomicBool, Ordering};
use core::slice;
use core::ops;

use vga_framebuffer::Colour;

static VGA_HANDLE_CREATED: AtomicBool = AtomicBool::new(false);

const VGA_TEXT_BUFFER_START: *mut u16 = 0xB8000 as *mut u16;
pub const VGA_TEXT_WIDTH: usize = 80;
pub const VGA_TEXT_HEIGHT: usize = 25;

#[derive(Debug)]
pub struct VgaTextBuffer {
    pub vga_text_buffer: &'static mut [u16],
}

impl VgaTextBuffer {
    pub unsafe fn new_unsafe() -> Self {
        let vga_text_buffer = slice::from_raw_parts_mut(VGA_TEXT_BUFFER_START, VGA_TEXT_WIDTH*VGA_TEXT_HEIGHT);

        Self {
            vga_text_buffer
        }
    }

    pub fn new() -> Option<Self> {
        if VGA_HANDLE_CREATED.compare_and_swap(false, true, Ordering::SeqCst) {
            None
        } else {
            let vga_text_buffer = unsafe { Self::new_unsafe() };
            Some(vga_text_buffer)
        }
    }

    pub fn clear(&mut self) {
        for x in self.vga_text_buffer.iter_mut() {
            *x = 0;
        }
    }

    pub fn write_to_start(&mut self, data: &[u16]) {
        for (target, data) in self.vga_text_buffer.iter_mut().zip(data.iter()) {
            *target = *data;
        }
    }

    pub fn write(&mut self, x: usize, y: usize, value: u16) {
        self.vga_text_buffer[x+y*VGA_TEXT_WIDTH] = value;
    }

    pub fn line(&mut self, y: usize) -> &[u16] {
        &self.vga_text_buffer[y*VGA_TEXT_WIDTH .. (y+1)*VGA_TEXT_WIDTH]
    }

    pub fn line_mut(&mut self, y: usize) -> &mut [u16] {
        &mut self.vga_text_buffer[y*VGA_TEXT_WIDTH .. (y+1)*VGA_TEXT_WIDTH]
    }

    pub fn two_lines_mut(&mut self, y: usize) -> &mut [u16] {
        &mut self.vga_text_buffer[y*VGA_TEXT_WIDTH .. (y+2)*VGA_TEXT_WIDTH]
    }

    pub fn scroll_line(&mut self) {
        for x in 0..(VGA_TEXT_HEIGHT-1) {
            let (line1, line2) = self.two_lines_mut(x).split_at_mut(VGA_TEXT_WIDTH);
            for (target, data) in line1.iter_mut().zip(line2.iter()) {
                *target = *data;
            }
        }

        for x in self.line_mut(VGA_TEXT_HEIGHT-1).iter_mut() {
            *x = 0;
        }
    }
}

pub fn white_text(text_code: u8) -> u16 {
    vga_text_value(false, Colour::Black, false, Colour::White, text_code)
}

pub fn vga_text_value(blink: bool, background_color: Colour, foreground_intensity: bool, foreground_color: Colour, text_code: u8) -> u16 {
    let mut left = foreground_color as u8;

    if foreground_intensity {
        left |= 0b0000_1000;
    }

    left |= (background_color as u8) << 4;

    if blink {
        left |= 0b1000_0000;
    }


    ((left as u16) << 8) | text_code as u16
}
