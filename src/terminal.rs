

use crate::vga_text::{VgaTextBuffer, VGA_TEXT_WIDTH, VGA_TEXT_HEIGHT, white_text};

#[derive(Debug)]
pub struct Terminal {
    text_buffer: VgaTextBuffer,
    position: usize,
}


impl Terminal {
    pub fn new(text_buffer: VgaTextBuffer) -> Self {
        Self {
            text_buffer,
            position: 0,
        }
    }

    pub fn write_char(&mut self, x: char) {
        if x == '\n' {
            self.text_buffer.scroll_line();
            self.position = 0;
            return;
        }

        let vga_char_code = white_text(vga_framebuffer::Char::map_char(x).to_byte());

        self.text_buffer.write(self.position, VGA_TEXT_HEIGHT-1, vga_char_code);

        self.position += 1;

        if self.position >= VGA_TEXT_WIDTH {
            self.position = 0;
            self.text_buffer.scroll_line();
        }
    }

    pub fn write_str(&mut self, text: &str) {
        for character in text.chars() {
            self.write_char(character);
        }
    }
}

impl core::fmt::Write for Terminal {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_str(s);
        Ok(())
    }
}
