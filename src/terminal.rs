

use crate::input::KeyPress;

use arrayvec::{ArrayString, ArrayVec};

use crate::vga_text::{VgaTextMode};

use vga::driver::text::{ VGA_TEXT_WIDTH, VGA_TEXT_HEIGHT, VgaChar, Colour };

const COMMAND_HISTORY_LAST_LINE_INDEX: usize = VGA_TEXT_HEIGHT - 2;
const COMMAND_HISTORY_LINE_COUNT: usize = VGA_TEXT_HEIGHT - 1;
const COMMAND_LINE_INDEX_Y: usize = VGA_TEXT_HEIGHT - 1;
const COMMAND_LENGTH: usize = VGA_TEXT_WIDTH - 1;

fn write_char_to_vga_text_buffer(text_mode: &mut VgaTextMode, x: usize, y: usize, c: char, blink: bool) {
    let vga_char = VgaChar::new(c).blink(blink).foreground_color(Colour::White);
    text_mode.lines_mut().nth(y).unwrap().iter_mut().nth(x).unwrap().write(vga_char);
}

pub struct DebugLine<'a> {
    text_mode: &'a mut VgaTextMode,
    position: usize,
}

impl <'a> DebugLine<'a> {
    pub fn new(text_mode: &'a mut VgaTextMode) -> Self {
        Self { text_mode, position: 0 }
    }
}

impl core::fmt::Write for DebugLine<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let mut line = self.text_mode.lines_mut().nth(0).expect("DebugLine error");
        for (c, mut vga) in s.chars().zip(line.iter_mut().skip(self.position)) {
            let vga_char = VgaChar::new(c).foreground_color(Colour::White);
            vga.write(vga_char);
            self.position += 1;
        }

        Ok(())
    }
}

pub struct Terminal {
    text_mode: VgaTextMode,
    history: CommandHistory,
    command_line: CommandLine,
}


impl Terminal {
    pub fn new(text_mode: VgaTextMode, init_cmd: bool) -> Self {
        let mut terminal = Self {
            text_mode,
            history: CommandHistory::new(),
            command_line: CommandLine::new(),
        };

        if init_cmd {
            terminal.command_line.clear_line(&mut terminal.text_mode);
            terminal.command_line.add_char(&mut terminal.text_mode, '>');
            terminal.text_mode.set_cursor_height(13, 14);
            terminal.text_mode.set_cursor_visibility(true);
        }

        terminal
    }

    pub fn update_command_line<'a>(&mut self, key: KeyPress, cmd_store: &'a mut CommandStore) -> Option<ParsedCommand<'a>> {
        self.command_line.update_command_line(key, &mut self.text_mode, &mut self.history, cmd_store)
    }

    pub fn new_command_line(&mut self, cmd_store: &mut CommandStore) {
        self.command_line.new_command_line(&mut self.text_mode, &mut self.history, cmd_store);
    }
}

impl core::fmt::Write for Terminal {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.history.add_text(&mut self.text_mode, s.chars());
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

    fn add_text(&mut self, text_mode: &mut VgaTextMode, chars: impl Iterator<Item=char>) {
        for character in chars {
            self.write_char(text_mode, character);
        }
    }
}

pub struct CommandLine {
    editable_command: ArrayVec<[char; COMMAND_LENGTH]>,
    position: usize,
}

impl CommandLine {
    fn new() -> Self {
        Self {
            editable_command: ArrayVec::new(),
            position: 0,
        }
    }

    fn draw_command_line(&mut self, text_mode: &mut VgaTextMode) {
        for (i, &c) in self.editable_command.iter().enumerate() {
            write_char_to_vga_text_buffer(text_mode, i, COMMAND_LINE_INDEX_Y, c, false);
        }

        let cmd_len = self.editable_command.iter().count();

        // Clear the end of the command line. Character deleting support requires this.
        for mut c in text_mode.lines_mut().nth(COMMAND_LINE_INDEX_Y).unwrap().iter_mut().skip(cmd_len) {
            c.write(Self::whitespace_character());
        }

        self.update_cursor_position(text_mode);
    }

    fn add_char(&mut self, text_mode: &mut VgaTextMode, c: char) {
        if let Ok(()) = self.editable_command.try_insert(self.position, c) {
            self.position += 1;
            self.draw_command_line(text_mode);
        }
    }

    pub fn update_command_line<'a>(&mut self, key: KeyPress, text_mode: &mut VgaTextMode, command_history: &mut CommandHistory, cmd_store: &'a mut CommandStore) -> Option<ParsedCommand<'a>> {
        match key {
            KeyPress::Enter => {
                self.new_command_line(text_mode, command_history, cmd_store);
                return Some(ParsedCommand::parse(&cmd_store.cmd));
            }
            KeyPress::Unicode(c) => {
                if c.is_ascii() {
                    self.add_char(text_mode, c);
                }
            },
            KeyPress::Left => {
                if self.position > 1 {
                    self.position -= 1;
                }
                self.update_cursor_position(text_mode);
            },
            KeyPress::Right => {
                if self.position < self.editable_command.len() {
                    self.position += 1;
                }
                self.update_cursor_position(text_mode);
            },
            KeyPress::Backspace => {
                if self.editable_command.len() > 1 && self.position > 1 {
                    self.editable_command.remove(self.position - 1);
                    self.position -= 1;
                    self.draw_command_line(text_mode);
                }
            },
            KeyPress::Delete => {
                if self.editable_command.len() > self.position && self.position > 0 {
                    self.editable_command.remove(self.position);
                    self.draw_command_line(text_mode);
                }
            }
            KeyPress::Home => {
                self.position = 1;
                self.update_cursor_position(text_mode);
            }
            KeyPress::End => {
                self.position = self.editable_command.len();
                self.update_cursor_position(text_mode);
            }
            _ => (),
        }
        None
    }

    pub fn new_command_line(&mut self, text_mode: &mut VgaTextMode, command_history: &mut CommandHistory, cmd_store: &mut CommandStore) {
        command_history.add_text(text_mode, self.editable_command.iter().copied());
        command_history.add_text(text_mode, "\n".chars());
        cmd_store.replace_cmd(self.editable_command.iter().copied());

        self.editable_command.clear();
        self.position = 0;

        self.clear_line(text_mode);

        self.add_char(text_mode, '>');
    }

    pub fn clear_line(&self, text_mode: &mut VgaTextMode) {
        text_mode.lines_mut().nth(COMMAND_LINE_INDEX_Y).expect("new_command_line, line not found").clear_with(Self::whitespace_character());
    }

    pub fn whitespace_character() -> VgaChar {
        VgaChar::new(' ').foreground_color(Colour::White)
    }

    pub fn update_cursor_position(&self, text_mode: &mut VgaTextMode) {
        text_mode.set_cursor_character_index(self.position + COMMAND_LINE_INDEX_Y * VGA_TEXT_WIDTH);
    }
}

pub struct CommandStore {
    pub cmd: ArrayString<[u8; COMMAND_LENGTH]>,
}

impl CommandStore {
    pub fn new() -> Self {
        Self {
            cmd: ArrayString::new(),
        }
    }

    fn replace_cmd(&mut self, chars: impl Iterator<Item=char>) {
        self.cmd.clear();
        for character in chars {
            self.cmd.try_push(character).expect("Multibyte characters are not supported currently.");
        }
    }
}


pub struct ParsedCommand<'a> {
    pub name: &'a str,
    pub arguments: core::str::SplitWhitespace<'a>,
}

impl <'a> ParsedCommand<'a> {
    pub fn parse(cmd_line: &'a str) -> Self {
        let cmd = cmd_line[1..].trim();

        let mut iter = cmd.split_whitespace();
        let name = iter.next().unwrap_or_default();

        Self {
            name,
            arguments: iter,
        }
    }
}
