
//! Keyboard and mouse support.


use pc_ps2_controller::{
    controller::{
        io::{PortIO, DATA_PORT_RAW, STATUS_REGISTER_RAW, COMMAND_REGISTER_RAW},
        driver::{EnabledDevices, InitController, DeviceData, ReadData, marker::InterruptsEnabled, InterfaceError, Testing, EnableDevice, ResetCPU},
    },
    device::keyboard::driver::{Keyboard, KeyboardError, KeyboardEvent},
    device::command_queue::Command,
    device::io::SendToDevice,
    pc_keyboard::{
        layouts::Us104Key,
        ScancodeSet2,
        DecodedKey,
        KeyEvent,
        KeyCode,
        KeyState,
        HandleControl,
    },
    pc_keyboard,
};

pub struct PS2ControllerIO;
#[derive(Copy, Clone)]
pub struct PortID(u16);

impl PortIO for PS2ControllerIO {
    type PortID = PortID;

    const DATA_PORT: Self::PortID = PortID(DATA_PORT_RAW);
    const STATUS_REGISTER: Self::PortID = PortID(STATUS_REGISTER_RAW);
    const COMMAND_REGISTER: Self::PortID = PortID(COMMAND_REGISTER_RAW);

    fn read(&mut self, port: Self::PortID) -> u8 {
        unsafe {
            x86::io::inb(port.0)
        }
    }

    fn write(&mut self, port: Self::PortID, data: u8) {
        unsafe {
            x86::io::outb(port.0, data)
        }
    }
}

pub struct ToKeyboard<'a>(&'a mut EnabledDevices<PS2ControllerIO, InterruptsEnabled>);

impl SendToDevice for ToKeyboard<'_> {
    fn send(&mut self, data: u8) {
        self.0.send_to_keyboard(data).unwrap();
    }
}

pub struct Input {
    ps2_controller: EnabledDevices<PS2ControllerIO, InterruptsEnabled>,
    keyevent_decoder: pc_keyboard::Keyboard<Us104Key, ScancodeSet2>,
    keyboard_driver: Keyboard<[Command; 8]>,
}

#[derive(Debug)]
pub enum InputError {
    ControllerSelfTestError(u8),
    KeyboardConnectionError(InterfaceError),
}

impl Input {
    // Disable interrupts before calling this function.
    pub fn init() -> Result<Self, InputError> {
        let mut controller = InitController::start_init(PS2ControllerIO);
        controller.self_test().map_err(|e| InputError::ControllerSelfTestError(e))?;
        controller.scancode_translation(false);
        let mut controller = controller.enable_devices_and_interrupts(EnableDevice::Keyboard).map_err(|(_, e)| InputError::KeyboardConnectionError(e))?;
        let keyevent_decoder = pc_keyboard::Keyboard::new(Us104Key, ScancodeSet2, HandleControl::Ignore);
        let mut keyboard_driver = Keyboard::new(&mut ToKeyboard(&mut controller)).unwrap();
        keyboard_driver.enable(&mut ToKeyboard(&mut controller)).unwrap();

        let input = Input {
            ps2_controller: controller,
            keyevent_decoder,
            keyboard_driver,
        };

        Ok(input)
    }

    pub fn handle_keyboard_interrupt(&mut self) -> Result<Option<KeyPress>, KeyboardError> {
        if let Some(DeviceData::Keyboard(data)) = self.ps2_controller.read_data() {
            self.keyboard_driver.receive_data(data, &mut ToKeyboard(&mut self.ps2_controller)).map(|event| {
                match event {
                    Some(KeyboardEvent::Key(event)) => self.keyboard_event_to_key_press(event),
                    Some(KeyboardEvent::BATCompleted) |
                    Some(KeyboardEvent::ID {..}) |
                    Some(KeyboardEvent::ScancodeSet(_)) |
                    Some(KeyboardEvent::Echo) |
                    None => None,
                }
            })
        } else {
            Ok(None)
        }
    }

    fn keyboard_event_to_key_press(&mut self, key_event: KeyEvent) -> Option<KeyPress> {
        let converted = match key_event.code {
            KeyCode::ArrowUp => KeyPress::Up,
            KeyCode::ArrowDown => KeyPress::Down,
            KeyCode::ArrowLeft => KeyPress::Left,
            KeyCode::ArrowRight => KeyPress::Right,
            KeyCode::Enter => KeyPress::Enter,
            KeyCode::Backspace => KeyPress::Backspace,
            KeyCode::Escape => KeyPress::Escape,
            KeyCode::Delete => KeyPress::Delete,
            KeyCode::Home => KeyPress::Home,
            KeyCode::End => KeyPress::End,
            _ => {
                return match self.keyevent_decoder.process_keyevent(key_event) {
                    Some(DecodedKey::Unicode(c)) => Some(KeyPress::Unicode(c)),
                    _ => None,
                };
            }
        };

        if let KeyState::Down = key_event.state {
            Some(converted)
        } else {
            None
        }
    }

    pub fn reboot_computer(&mut self) {
        self.ps2_controller.reset_cpu();
    }
}

pub enum KeyPress {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Backspace,
    Escape,
    Delete,
    Home,
    End,
    Unicode(char)
}
