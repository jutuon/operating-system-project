

use ps2_controller_x86::pc_keyboard::{DecodedKey, KeyCode};

use arrayvec::ArrayString;

use crate::vga_text::{VgaTextBuffer, VGA_TEXT_WIDTH, VGA_TEXT_HEIGHT, white_text};

const COMMAND_LENGHT: usize = VGA_TEXT_WIDTH - 1;

#[derive(Debug)]
pub struct Terminal {
    text_buffer: VgaTextBuffer,
    position: usize,
    command: ArrayString<[u8; COMMAND_LENGHT]>
}


impl Terminal {
    pub fn new(text_buffer: VgaTextBuffer) -> Self {
        Self {
            text_buffer,
            position: 0,
            command: ArrayString::new(),
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

    pub fn update_command_line(&mut self, key: DecodedKey) {
        match key {
            DecodedKey::Unicode(c) => {
                if c == '\n' {
                    self.new_command_line();
                } else if c.is_ascii() && self.position < VGA_TEXT_WIDTH - 1 {
                    self.write_char(c);
                    self.command.push(c);
                }
            },
            DecodedKey::RawKey(key_code) => {
                match key_code {
                    KeyCode::ArrowLeft => self.write_char('q'),
                    KeyCode::ArrowRight => (),
                    KeyCode::Backspace =>  self.write_char('e'),
                    KeyCode::Enter => self.write_char('w'),
                    _ => (),
                }
            }
        }
    }

    pub fn new_command_line(&mut self) {
        self.command.clear();
        self.position = 0;
        self.text_buffer.scroll_line();

        self.write_char('>');
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
