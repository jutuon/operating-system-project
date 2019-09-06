
use core::sync::atomic::{AtomicUsize, AtomicU32, Ordering};
use core::num::Wrapping;

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

#[repr(transparent)]
pub struct IDT {
    entries: [Descriptor; 256],
}

static mut IDT_DATA: IDT = IDT {
    entries: [Descriptor::NULL; 256],
};

use pc_at_pic8259a::*;

pub struct PicPortIO;

impl PortIO for PicPortIO {
    fn read(&self, port: u16) -> u8 {
        unsafe { x86::io::inb(port) }
    }

    fn write(&mut self, port: u16, data: u8) {
        unsafe { x86::io::outb(port, data); }
    }
}

pub struct IDTHandler;

use core::mem::MaybeUninit;

const MASTER_PIC_INTERRUPT_OFFSET: u8 = 32;
const SLAVE_PIC_INTERRUPT_OFFSET: u8 = MASTER_PIC_INTERRUPT_OFFSET + 8;

const MASTER_PIC_SPURIOUS_INTERRUPT: u8 = MASTER_PIC_INTERRUPT_OFFSET + 7;
const SLAVE_PIC_SPURIOUS_INTERRUPT: u8 = SLAVE_PIC_INTERRUPT_OFFSET + 7;

static RECEIVED_HARDWARE_INTERRUPT_BITFLAGS: AtomicU32 = AtomicU32::new(0);
static mut INTERRUPT_DEQUE: Option<ArrayDeque<[HardwareInterrupt; 32], Saturating>> = None;

static mut PIC: Option<Pic<PicPortIO>> = None;

static MASTER_PIC_SPURIOUS_INTERRUPT_COUNT: AtomicUsize = AtomicUsize::new(0);
static SLAVE_PIC_SPURIOUS_INTERRUPT_COUNT: AtomicUsize = AtomicUsize::new(0);

pub static TIME_MILLISECONDS: AtomicUsize = AtomicUsize::new(0);

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
            INTERRUPT_DEQUE = Some(ArrayDeque::new());
        }

        let mut pic = PicInit::send_icw1(PicPortIO, InterruptTriggerMode::EdgeTriggered)
            .send_icw2_and_icw3(MASTER_PIC_INTERRUPT_OFFSET, SLAVE_PIC_INTERRUPT_OFFSET)
            .send_icw4();

        // Dedicate last interrupt line for spurious interrupts.
        const LAST_IRQ_LINE: u8 = 0b1000_0000;
        pic.set_master_mask(LAST_IRQ_LINE);
        pic.set_slave_mask(LAST_IRQ_LINE);

        unsafe {
            PIC = Some(pic);
        }

        IDTHandler
    }

    pub fn enable_interrupts(&mut self) {
        unsafe {
            x86::irq::enable();
        }
    }

    pub fn handle_interrupt(&mut self) -> Option<HardwareInterrupt> {
        unsafe {
            x86::irq::disable();
            let interrupt = INTERRUPT_DEQUE.as_mut().unwrap().pop_front();
            if let Some(hardware_interrupt) = &interrupt {
                let new = RECEIVED_HARDWARE_INTERRUPT_BITFLAGS.load(Ordering::Relaxed) & !(1 << *hardware_interrupt as u8);
                RECEIVED_HARDWARE_INTERRUPT_BITFLAGS.store(new, Ordering::Relaxed);
            }
            x86::irq::enable();
            interrupt
        }
    }

    pub fn master_pic_spurious_interrupts_count() -> usize {
        MASTER_PIC_SPURIOUS_INTERRUPT_COUNT.load(Ordering::Relaxed)
    }
    pub fn slave_pic_spurious_interrupts_count() -> usize {
        SLAVE_PIC_SPURIOUS_INTERRUPT_COUNT.load(Ordering::Relaxed)
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
    fn from_interrupt_number(interrupt_number: u8) -> Result<Self, UnknownInterrupt> {
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
    fn from_interrupt_number(interrupt_number: u8) -> Result<Self, UnknownInterrupt> {
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
    let interrupt_number: u8 = interrupt_number as u8;

    use core::fmt::Write;

    let text_mode = unsafe {
        crate::vga_text::new_vga_text_mode_unsafe()
    };

    let mut terminal = crate::terminal::Terminal::new(text_mode, false);

    let exception = Exception::from_interrupt_number(interrupt_number);

    if exception.is_ok() {
        panic!("Interrupt {:?}, number: {}", exception, interrupt_number);
    } else {
        let hardware_interrupt = HardwareInterrupt::from_interrupt_number(interrupt_number);

        if let Ok(interrupt) = hardware_interrupt {
            if let HardwareInterrupt::Timer = interrupt {
                let new_time: Wrapping<usize> = Wrapping(TIME_MILLISECONDS.load(Ordering::Relaxed)) + Wrapping(55usize);
                TIME_MILLISECONDS.store(new_time.0, Ordering::Relaxed);
            } else {
                let flag = 1 << interrupt as u8;
                let interrupt_received_bitflags = RECEIVED_HARDWARE_INTERRUPT_BITFLAGS.load(Ordering::Relaxed);
                if flag & interrupt_received_bitflags == 0 {
                    unsafe {
                        INTERRUPT_DEQUE.as_mut().unwrap().push_back(interrupt).unwrap();
                    }
                    RECEIVED_HARDWARE_INTERRUPT_BITFLAGS.store(interrupt_received_bitflags | flag, Ordering::Relaxed);
                }
            }
        }

        let pic = unsafe {
            PIC.as_mut().unwrap()
        };

        if MASTER_PIC_INTERRUPT_OFFSET <= interrupt_number && interrupt_number < MASTER_PIC_SPURIOUS_INTERRUPT {
            pic.send_eoi_to_master();
        } else if SLAVE_PIC_INTERRUPT_OFFSET <= interrupt_number && interrupt_number < SLAVE_PIC_SPURIOUS_INTERRUPT {
            pic.send_eoi_to_slave();
            pic.send_eoi_to_master();
        }

        if interrupt_number == MASTER_PIC_SPURIOUS_INTERRUPT {
            let _ = writeln!(terminal, "Spurious interrupt from master PIC");

            MASTER_PIC_SPURIOUS_INTERRUPT_COUNT.fetch_add(1, Ordering::Relaxed);
        }

        if interrupt_number == SLAVE_PIC_SPURIOUS_INTERRUPT {
            let _ = writeln!(terminal, "Spurious interrupt from slave PIC");

            SLAVE_PIC_SPURIOUS_INTERRUPT_COUNT.fetch_add(1, Ordering::Relaxed);

            pic.send_eoi_to_master();
        }
    }
}

#[no_mangle]
extern "C" fn rust_interrupt_handler_with_error(
    interrupt_number: u32,
    error_code: u32
) {
    let exception = Exception::from_interrupt_number(interrupt_number as u8);
    panic!("Interrupt {:?}, number: {}, error: {:#08x}",
        exception, interrupt_number, error_code);
}
