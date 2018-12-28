
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

#[no_mangle]
extern "C" fn rust_interrupt_handler(interrupt_number: u32) {
    use core::fmt::Write;

    let text_buffer = unsafe {
        crate::vga_text::VgaTextBuffer::new_unsafe()
    };

    let mut terminal = crate::terminal::Terminal::new(text_buffer);

    writeln!(terminal, "Interrupt {}", interrupt_number);
}


#[no_mangle]
extern "C" fn rust_interrupt_handler_with_error(
    interrupt_number: u32,
    error_code: u32
) {
    panic!("Interrupt {}, error: {:#08x}",
        interrupt_number, error_code);
}
