
//! Keyboard and mouse support.


use ps2_controller_x86::{
    PortIO,
    PS2Controller,
    pc_keyboard::{
        Keyboard,
        layouts::Us104Key,
        ScancodeSet1,
        DecodedKey,
    },
};

pub struct PS2ControllerIO;

unsafe impl PortIO for PS2ControllerIO {
    fn read(&self, port: u16) -> u8 {
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

pub struct Input {
    controller: PS2Controller<PS2ControllerIO, Us104Key, ScancodeSet1>,
}

impl Input {
    pub fn new() -> Self {
        let keyboard = Keyboard::new(Us104Key, ScancodeSet1);
        let controller = PS2Controller::new(PS2ControllerIO, keyboard);

        Self {
            controller
        }
    }

    pub fn read_key(&mut self) -> Option<DecodedKey> {
        self.controller.read_keyboard()
    }
}
