
use x86::dtables::*;
use x86::segmentation::*;

extern {
    fn interrupt_hander();
}

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
                // TODO: functions
                let function_position = interrupt_hander as u32;

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
extern "C" fn rust_interrupt_handler() {
    use core::fmt::Write;

    let text_buffer = unsafe {
        crate::vga_text::VgaTextBuffer::new_unsafe()
    };

    let mut terminal = crate::terminal::Terminal::new(text_buffer);

    writeln!(terminal, "interrupt");
}


