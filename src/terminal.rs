

use crate::input::KeyPress;

use arrayvec::ArrayString;

use crate::vga_text::{VgaTextBuffer, VGA_TEXT_WIDTH, VGA_TEXT_HEIGHT, white_text};

const COMMAND_HISTORY_LAST_LINE_INDEX: usize = VGA_TEXT_HEIGHT - 2;
const COMMAND_HISTORY_LINE_COUNT: usize = VGA_TEXT_HEIGHT - 1;
const COMMAND_LINE_INDEX_Y: usize = VGA_TEXT_HEIGHT - 1;
const COMMAND_LENGHT: usize = VGA_TEXT_WIDTH - 1;

fn write_char_to_vga_text_buffer(text_buffer: &mut VgaTextBuffer, x: usize, y: usize, c: char) {
    let vga_char_code = white_text(vga_framebuffer::Char::map_char(c).to_byte());
    text_buffer.write(x, y, vga_char_code);
}


#[derive(Debug)]
pub struct Terminal {
    text_buffer: VgaTextBuffer,
    history: CommandHistory,
    command_line: CommandLine,
}


impl Terminal {
    pub fn new(text_buffer: VgaTextBuffer) -> Self {
        let mut terminal = Self {
            text_buffer,
            history: CommandHistory::new(),
            command_line: CommandLine::new(),
        };

        terminal.command_line.write_char(&mut terminal.text_buffer, '>');

        terminal
    }

    pub fn update_command_line(&mut self, key: KeyPress) {
        self.command_line.update_command_line(key, &mut self.text_buffer, &mut self.history);
    }

    pub fn new_command_line(&mut self) {
        self.command_line.new_command_line(&mut self.text_buffer, &mut self.history);
    }
}

impl core::fmt::Write for Terminal {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.history.add_text(&mut self.text_buffer, s);
        Ok(())
    }
}

#[derive(Debug)]
pub struct CommandHistory {
    position: usize,
    scroll_next: bool,
}

impl CommandHistory {
    fn new() -> Self {
        Self {
            position: 0,
            scroll_next: false,
        }
    }

    fn write_char(&mut self, text_buffer: &mut VgaTextBuffer, c: char) {
        if self.scroll_next {
            self.scroll_next = false;

            text_buffer.scroll_area(0, COMMAND_HISTORY_LINE_COUNT);
            self.position = 0;
        }

        if c == '\n' {
            self.scroll_next = true;
            return;
        }

        write_char_to_vga_text_buffer(text_buffer, self.position, COMMAND_HISTORY_LAST_LINE_INDEX, c);

        self.position += 1;

        if self.position >= VGA_TEXT_WIDTH {
            self.position = 0;
            text_buffer.scroll_area(0, COMMAND_HISTORY_LINE_COUNT);
        }
    }

    fn add_text(&mut self, text_buffer: &mut VgaTextBuffer, text: &str) {
        for character in text.chars() {
            self.write_char(text_buffer, character);
        }
    }
}

#[derive(Debug)]
pub struct CommandLine {
    command: ArrayString<[u8; COMMAND_LENGHT]>,
    position: usize,
}

impl CommandLine {
    fn new() -> Self {
        Self {
            command: ArrayString::new(),
            position: 0,
        }
    }

    fn write_char(&mut self, text_buffer: &mut VgaTextBuffer, c: char) {
        if self.position < VGA_TEXT_WIDTH - 1 {
            write_char_to_vga_text_buffer(text_buffer, self.position, COMMAND_LINE_INDEX_Y, c);
            self.position += 1;
        }
    }

    pub fn update_command_line(&mut self, key: KeyPress, text_buffer: &mut VgaTextBuffer, command_history: &mut CommandHistory) {
        match key {
            KeyPress::Unicode('n') | KeyPress::Enter => self.new_command_line(text_buffer, command_history),
            KeyPress::Unicode(c) => {
                if c.is_ascii() && self.position < VGA_TEXT_WIDTH - 1 {
                    self.write_char(text_buffer, c);
                    self.command.push(c);
                }
            },
            KeyPress::Left => (),
            KeyPress::Right => (),
            KeyPress::Backspace => (),
            _ => (),
        }
    }

    pub fn new_command_line(&mut self, text_buffer: &mut VgaTextBuffer, command_history: &mut CommandHistory) {
        command_history.add_text(text_buffer, ">");
        command_history.add_text(text_buffer, &self.command);
        command_history.add_text(text_buffer, "\n");

        self.command.clear();
        self.position = 0;

        text_buffer.clear_line(COMMAND_LINE_INDEX_Y);

        self.write_char(text_buffer, '>');
    }
}
