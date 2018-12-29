
use x86::dtables::*;
use x86::segmentation::*;

use seq_macro::seq;

seq!(N in 18..=255 {
    extern "C" {
        fn interrupt_0();
        fn interrupt_1();
        fn interrupt_2();
        fn interrupt_3();
        fn interrupt_4();
        fn interrupt_5();
        fn interrupt_6();
        fn interrupt_7();
        fn interrupt_with_error_8();
        fn interrupt_9();
        fn interrupt_with_error_10();
        fn interrupt_with_error_11();
        fn interrupt_with_error_12();
        fn interrupt_with_error_13();
        fn interrupt_with_error_14();
        fn interrupt_15();
        fn interrupt_16();
        fn interrupt_with_error_17();
        #(
            fn interrupt_#N();
        )*
    }
});

seq!(N in 18..=255 {
    const INTERRUPT_HANDLERS: [unsafe extern "C" fn (); 256] = [
        interrupt_0,
        interrupt_1,
        interrupt_2,
        interrupt_3,
        interrupt_4,
        interrupt_5,
        interrupt_6,
        interrupt_7,
        interrupt_with_error_8,
        interrupt_9,
        interrupt_with_error_10,
        interrupt_with_error_11,
        interrupt_with_error_12,
        interrupt_with_error_13,
        interrupt_with_error_14,
        interrupt_15,
        interrupt_16,
        interrupt_with_error_17,
        #(
            interrupt_#N,
        )*
    ];
});

pub struct IDT {
    entries: [Descriptor; 256],
}

static mut IDT_DATA: IDT = IDT {
    entries: [Descriptor::NULL; 256],
};

impl IDT {
    pub fn load_idt() {
        unsafe {
            for (i, entry) in IDT_DATA.entries.iter_mut().enumerate() {
                let function_position = INTERRUPT_HANDLERS[i] as u32;

                let descriptor = DescriptorBuilder::interrupt_descriptor(SegmentSelector::new(1, x86::Ring::Ring0), function_position)
                    .present()
                    .finish();

                *entry = descriptor;
            }
            let idt_pointer = DescriptorTablePointer::new(&IDT_DATA);

            lidt(&idt_pointer);
        }
    }

    pub fn enable_interrupts() {
        unsafe {
            x86::irq::enable();
        }
    }
}

#[derive(Debug)]
pub enum Exception {
    DivideByZero = 0,
    Debug,
    NonMaskableInterrupt,
    Breakpoint,
    Overflow,
    BoundRange,
    InvalidOpcode,
    DeviceNotAvailable,
    DoubleFault,
    CoprosessorSegmentOverrun,
    InvalidTSS,
    SegementNotPresent,
    Stack,
    GeneralProtection,
    PageFault = 14,
    X87FloatingPoint = 16,
    AlignmentCheck,
    MachineCheck,
    SMIDFloatingPoint = 19,
    VMMCommunication = 29,
    Security,
}

#[derive(Debug)]
struct UnknownInterrupt;

impl Exception {
    fn from_interrupt_number(interrupt_number: u32) -> Result<Self, UnknownInterrupt> {
        use self::Exception::*;
        let exception = match interrupt_number {
            0 => DivideByZero,
            1 => Debug,
            2 => NonMaskableInterrupt,
            3 => Breakpoint,
            4 => Overflow,
            5 => BoundRange,
            6 => InvalidOpcode,
            7 => DeviceNotAvailable,
            8 => DoubleFault,
            9 => CoprosessorSegmentOverrun,
            10 => InvalidTSS,
            11 => SegementNotPresent,
            12 => Stack,
            13 => GeneralProtection,
            14 => PageFault,
            16 => X87FloatingPoint,
            17 => AlignmentCheck,
            18 => MachineCheck,
            19 => SMIDFloatingPoint,
            29 => VMMCommunication,
            30 => Security,
            _ => return Err(UnknownInterrupt),
        };
        Ok(exception)
    }
}

#[derive(Debug)]
pub enum HardwareInterrupt {
    Timer,
    Keyboard,
    COM2,
    COM1,
    LPT2,
    FloppyDisk,
    LPT1,
    RealTimeClock,
    Mouse,
    FPU,
    PrimaryHardDisk,
    SecondaryHardDisk,
}

impl HardwareInterrupt {
    fn from_interrupt_number(interrupt_number: u32) -> Result<Self, UnknownInterrupt> {
        use self::HardwareInterrupt::*;
        let interrupt = match interrupt_number {
            32 => Timer,
            33 => Keyboard,
            34 => COM2,
            35 => COM1,
            36 => LPT2,
            37 => FloppyDisk,
            38 => LPT1,
            39 => RealTimeClock,
            40 => Mouse,
            42 => FPU,
            43 => PrimaryHardDisk,
            44 => SecondaryHardDisk,
            _ => return Err(UnknownInterrupt),
        };
        Ok(interrupt)
    }
}

#[no_mangle]
extern "C" fn rust_interrupt_handler(interrupt_number: u32) {
    use core::fmt::Write;

    let text_buffer = unsafe {
        crate::vga_text::VgaTextBuffer::new_unsafe()
    };

    let mut terminal = crate::terminal::Terminal::new(text_buffer);

    let exception = Exception::from_interrupt_number(interrupt_number);
    if exception.is_ok() {
        writeln!(terminal, "Interrupt {:?}, number: {}", exception, interrupt_number);
    } else {
        let hardware_interrupt = HardwareInterrupt::from_interrupt_number(interrupt_number);
        writeln!(terminal, "Interrupt {:?}, number: {}", hardware_interrupt, interrupt_number);
    }
}

#[no_mangle]
extern "C" fn rust_interrupt_handler_with_error(
    interrupt_number: u32,
    error_code: u32
) {
    let exception = Exception::from_interrupt_number(interrupt_number);
    panic!("Interrupt {:?}, number: {}, error: {:#08x}",
        exception, interrupt_number, error_code);
}
