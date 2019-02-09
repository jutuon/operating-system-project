
//! Keyboard and mouse support.


use pc_ps2_controller::{
    *,
    pc_keyboard::{
        Keyboard,
        layouts::Us104Key,
        ScancodeSet1,
        DecodedKey,
    },
};

pub struct PS2ControllerIO;

impl PortIO for PS2ControllerIO {
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

pub struct Input {
    keyboard_driver: KeyboardDriver<PS2ControllerIO, EnabledDevices<PS2ControllerIO, KeyboardEnabled, Disabled, InterruptsEnabled>, Us104Key, ScancodeSet1>,
}

#[derive(Debug)]
pub enum InputError {
    ControllerSelfTestError(u8),
    KeyboardConnectionError(DeviceInterfaceError),
}

pub struct NextKeyboardInterrupt(InitControllerWaitInterrupt<PS2ControllerIO>);

impl NextKeyboardInterrupt {
    pub fn poll_data(self) -> Result<Input, InputError> {
        let mut controller = self.0.poll_data();
        controller.self_test().map_err(|e| InputError::ControllerSelfTestError(e))?;
        let controller = controller.enable_keyboard_and_interrupts().map_err(|(_, e)| InputError::KeyboardConnectionError(e))?;
        let keyboard = Keyboard::new(Us104Key, ScancodeSet1);
        let keyboard_driver = KeyboardDriver::new(controller, keyboard);

        let input = Input {
            keyboard_driver
        };

        Ok(input)
    }
}

impl Input {
    pub fn start_init() -> NextKeyboardInterrupt {
        NextKeyboardInterrupt(InitController::start_init(PS2ControllerIO))
    }

    pub fn handle_keyboard_interrupt(&mut self) -> Option<DecodedKey> {
        self.keyboard_driver.handle_keyboard_interrupt()
    }

    pub fn poll_keyboard(&mut self) -> Option<DecodedKey> {
        self.keyboard_driver.poll_keyboard()
    }
}
