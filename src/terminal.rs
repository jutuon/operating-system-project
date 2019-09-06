

use crate::input::KeyPress;

use arrayvec::ArrayString;

use crate::vga_text::{self, VgaTextMode};

use vga::driver::text::{ VGA_TEXT_WIDTH, VGA_TEXT_HEIGHT, VgaChar, Colour, AttributeBit7 };

const COMMAND_HISTORY_LAST_LINE_INDEX: usize = VGA_TEXT_HEIGHT - 2;
const COMMAND_HISTORY_LINE_COUNT: usize = VGA_TEXT_HEIGHT - 1;
const COMMAND_LINE_INDEX_Y: usize = VGA_TEXT_HEIGHT - 1;
const COMMAND_LENGTH: usize = VGA_TEXT_WIDTH - 1;


fn write_char_to_vga_text_buffer(text_mode: &mut VgaTextMode, x: usize, y: usize, c: char, blink: bool) {
    let vga_char = VgaChar::new(c).blink(blink).foreground_color(Colour::White);
    text_mode.lines_mut().nth(y).unwrap().iter_mut().nth(x).unwrap().write(vga_char);
}


pub struct Terminal {
    text_mode: VgaTextMode,
    history: CommandHistory,
    command_line: CommandLine,
}


impl Terminal {
    pub fn new(mut text_mode: VgaTextMode, init_cmd: bool) -> Self {
        //text_mode.attribute_bit_7(AttributeBit7::Blink);
        let mut terminal = Self {
            text_mode,
            history: CommandHistory::new(),
            command_line: CommandLine::new(),
        };

        if init_cmd {
            terminal.command_line.add_char(&mut terminal.text_mode, '>');
        }

        terminal
    }

    pub fn update_command_line(&mut self, key: KeyPress) {
        self.command_line.update_command_line(key, &mut self.text_mode, &mut self.history);
    }

    pub fn new_command_line(&mut self) {
        self.command_line.new_command_line(&mut self.text_mode, &mut self.history);
    }
}

impl core::fmt::Write for Terminal {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.history.add_text(&mut self.text_mode, s);
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

    fn write_char(&mut self, text_mode: &mut VgaTextMode, c: char) {
        if self.scroll_next {
            self.scroll_next = false;

            text_mode.scroll_range(0..COMMAND_HISTORY_LINE_COUNT);
            text_mode.lines_mut().nth(COMMAND_HISTORY_LINE_COUNT - 1).unwrap().clear_with(VgaChar::empty());
            self.position = 0;
        }

        if c == '\n' {
            self.scroll_next = true;
            return;
        }

        write_char_to_vga_text_buffer(text_mode, self.position, COMMAND_HISTORY_LAST_LINE_INDEX, c, false);

        self.position += 1;

        if self.position >= VGA_TEXT_WIDTH {
            self.position = 0;
            text_mode.scroll_range(0..COMMAND_HISTORY_LINE_COUNT);
            text_mode.lines_mut().nth(COMMAND_HISTORY_LINE_COUNT - 1).unwrap().clear_with(VgaChar::empty());
        }
    }

    fn add_text(&mut self, text_mode: &mut VgaTextMode, text: &str) {
        for character in text.chars() {
            self.write_char(text_mode, character);
        }
    }
}

#[derive(Debug)]
pub struct CommandLine {
    command: ArrayString<[u8; COMMAND_LENGTH]>,
    position: usize,
}

impl CommandLine {
    fn new() -> Self {
        Self {
            command: ArrayString::new(),
            position: 0,
        }
    }

    fn draw_command_line(&mut self, text_mode: &mut VgaTextMode) {
        let cmd_len = self.command.chars().count();

        for (i, c) in self.command.chars().enumerate() {
            write_char_to_vga_text_buffer(text_mode, i, COMMAND_LINE_INDEX_Y, c, false);

            if i == cmd_len - 1 {
                write_char_to_vga_text_buffer(text_mode, cmd_len, COMMAND_LINE_INDEX_Y, '_', true);
            }
        }
    }

    fn add_char(&mut self, text_mode: &mut VgaTextMode, c: char) {
        if self.position < VGA_TEXT_WIDTH - 1 {
            self.command.push(c);
            self.position += 1;
            self.draw_command_line(text_mode);
        }
    }

    pub fn update_command_line(&mut self, key: KeyPress, text_mode: &mut VgaTextMode, command_history: &mut CommandHistory) {
        match key {
            KeyPress::Unicode('n') | KeyPress::Enter => self.new_command_line(text_mode, command_history),
            KeyPress::Unicode(c) => {
                if c.is_ascii() {
                    self.add_char(text_mode, c);
                }
            },
            KeyPress::Left => (),
            KeyPress::Right => (),
            KeyPress::Backspace => (),
            _ => (),
        }
    }

    pub fn new_command_line(&mut self, text_mode: &mut VgaTextMode, command_history: &mut CommandHistory) {

        command_history.add_text(text_mode, &self.command);
        command_history.add_text(text_mode, "\n");

        self.command.clear();
        self.position = 0;

        self.clear_line(text_mode);

        self.add_char(text_mode, '>');
    }

    pub fn clear_line(&self, text_mode: &mut VgaTextMode) {
        text_mode.lines_mut().nth(COMMAND_LINE_INDEX_Y).expect("new_command_line, line not found").clear_with(VgaChar::empty());
    }
}
