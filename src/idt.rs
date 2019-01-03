
use x86::dtables::*;
use x86::segmentation::*;

use arraydeque::{ArrayDeque, Saturating};

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

pub struct PicPortIO;

unsafe impl pic8259::PortIO for PicPortIO {
    fn read(&self, port: u16) -> u8 {
        unsafe { x86::io::inb(port) }
    }

    fn write(&mut self, port: u16, data: u8) {
        unsafe { x86::io::outb(port, data); }
    }
}

pub struct IDTHandler {
    pic: pic8259::PicAEOI<PicPortIO>,
}

const MASTER_PIC_INTERRUPT_OFFSET: u8 = 32;
const SLAVE_PIC_INTERRUPT_OFFSET: u8 = MASTER_PIC_INTERRUPT_OFFSET + 8;

const MASTER_PIC_SPURIOUS_INTERRUPT: u8 = MASTER_PIC_INTERRUPT_OFFSET + 7;
const SLAVE_PIC_SPURIOUS_INTERRUPT: u8 = SLAVE_PIC_INTERRUPT_OFFSET + 7;

static mut RECEIVED_HARDWARE_INTERRUPT_BITFLAGS: u32 = 0;
static mut INTERRUPT_DEQUE: core::mem::MaybeUninit<ArrayDeque<[HardwareInterrupt; 32], Saturating>> = unsafe { core::mem::MaybeUninit::uninitialized() };


impl IDTHandler {
    pub fn new() -> Self {
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

        unsafe {
            INTERRUPT_DEQUE.set(ArrayDeque::new());
        }

        use pic8259::*;

        let mut pic = pic8259::PicInit::start_init(PicPortIO, InterruptTriggerMode::EdgeTriggered)
            .interrupt_offsets(MASTER_PIC_INTERRUPT_OFFSET, SLAVE_PIC_INTERRUPT_OFFSET)
            .automatic_end_of_interrupt();

        // Dedicate last interrupt line for spurious interrupts.
        const LAST_IRQ_LINE: u8 = 0b1000_0000;
        const TIMER_IRQ_LINE: u8 = 0b0000_0001;
        pic.set_master_mask(LAST_IRQ_LINE | TIMER_IRQ_LINE);
        pic.set_slave_mask(LAST_IRQ_LINE);

        IDTHandler {
            pic
        }
    }

    pub fn enable_interrupts(&mut self) {
        unsafe {
            x86::irq::enable();
        }
    }

    pub fn handle_interrupt(&mut self) -> Option<HardwareInterrupt> {
        unsafe {
            let interrupt = INTERRUPT_DEQUE.get_mut().pop_front();
            if let Some(hardware_interrupt) = &interrupt {
                RECEIVED_HARDWARE_INTERRUPT_BITFLAGS &= !(1 << *hardware_interrupt as u8);
            }
            interrupt
        }
    }

    pub fn master_pic_spurious_interrupts_count() -> usize {
        unsafe { MASTER_PIC_SPURIOUS_INTERRUPT_COUNT }
    }
    pub fn slave_pic_spurious_interrupts_count() -> usize {
        unsafe { SLAVE_PIC_SPURIOUS_INTERRUPT_COUNT }
    }
}

static mut MASTER_PIC_SPURIOUS_INTERRUPT_COUNT: usize = 0;
static mut SLAVE_PIC_SPURIOUS_INTERRUPT_COUNT: usize = 0;


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

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
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
        panic!("Interrupt {:?}, number: {}", exception, interrupt_number);
    } else {
        let hardware_interrupt = HardwareInterrupt::from_interrupt_number(interrupt_number);

        if let Ok(interrupt) = hardware_interrupt {
            unsafe {
                let flag = 1 << interrupt as u8;
                if flag & RECEIVED_HARDWARE_INTERRUPT_BITFLAGS == 0 {
                    INTERRUPT_DEQUE.get_mut().push_back(interrupt).unwrap();
                    RECEIVED_HARDWARE_INTERRUPT_BITFLAGS |= flag;
                }
            }
        }

        if interrupt_number == MASTER_PIC_SPURIOUS_INTERRUPT as u32 {
            writeln!(terminal, "Spurious interrupt form master PIC");

            unsafe {
                MASTER_PIC_SPURIOUS_INTERRUPT_COUNT += 1;
            }
        }

        if interrupt_number == SLAVE_PIC_SPURIOUS_INTERRUPT as u32 {
            writeln!(terminal, "Spurious interrupt form slave PIC");

            unsafe {
                SLAVE_PIC_SPURIOUS_INTERRUPT_COUNT += 1;
            }
        }
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
