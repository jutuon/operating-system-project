#![feature(const_fn)]

#![no_std]
#![no_main]

pub mod vga_text;
pub mod terminal;
pub mod page_table;
pub mod gdt;
pub mod idt;
pub mod input;

use self::terminal::{Terminal};
use self::gdt::GDT;
use self::idt::IDTHandler;

use core::fmt::Write;

#[no_mangle]
extern "C" fn kernel_main(eax: u32, ebx: u32) -> ! {
    let mut vga_handle = vga_text::new_vga_text_mode().unwrap();
    vga_handle.clear_screen(vga::driver::text::VgaChar::empty());

    let mut terminal = Terminal::new(vga_handle, true);

    let _ = writeln!(terminal, "Hello world");

    if eax != 0x36d76289 {
        panic!("Boot loader was not Multiboot2-compliant, eax: {}", eax);
    }

    let boot_info = unsafe {
        multiboot2::load(ebx as usize)
    };

    let _ = writeln!(terminal, "{:?}", boot_info);

    check_cpu_features(&mut terminal).expect("error: CPU is not compatible");

    enable_cpu_features();

    GDT::load_gdt();

    let mut idt_handler = IDTHandler::new();

    let mut page_table = page_table::GlobalPageTable::new().expect("Page table handle loading failed");
    page_table.load_identity_map();

    unsafe {
        let cr3_data = page_table::pae_cr3_format(page_table.level3_start_address(), false, false);
        x86::controlregs::cr3_write(cr3_data as u64);
        x86::controlregs::cr0_write(x86::controlregs::Cr0::CR0_WRITE_PROTECT | x86::controlregs::Cr0::CR0_ENABLE_PAGING | x86::controlregs::cr0());
    }

    let mut input_module = match self::input::Input::init() {
        Ok(input) => {
            Some(input)
        },
        Err(e) => {
            let _ = writeln!(terminal, "Couldn't initialize keyboard: {:?}", e);
            None
        }
    };

    idt_handler.enable_interrupts();

    loop {
        while let Some(hardware_interrupt) = idt_handler.handle_interrupt() {
            use self::idt::HardwareInterrupt;
            match hardware_interrupt {
                HardwareInterrupt::Keyboard => {
                    if let Some(input) = &mut input_module {
                        let key = input.handle_keyboard_interrupt();

                        match key {
                            Ok(Some(k)) => terminal.update_command_line(k),
                            Ok(None) => (),
                            Err(e) => {
                                let _ = writeln!(terminal, "Keyboard error: {:?}", e);
                            }
                        }
                    }
                },
                hardware_interrupt => {
                    let _ = writeln!(terminal, "HardwareInterrupt: {:?}", hardware_interrupt);
                }
            }
        }

        // TODO: Here is a possible deadlock if IDT interrupt handler runs just before
        //       x86::halt() and if there won't be more interrupts until a device driver
        //       handles the received interrupt.

        unsafe {
            x86::halt()
        }
    }
}

fn check_cpu_features(log: &mut impl Write) -> Result<(), ()> {
    use x86::cpuid::CpuId;

    let cpu_id = CpuId::new();

    match cpu_id.get_vendor_info() {
        Some(vendor_info) => {
            match vendor_info.as_string() {
                "AuthenticAMD" | "GenuineIntel" => (),
                unknown_vendor => {
                    let _ = writeln!(log, "error: unknown CPU vendor '{}'", unknown_vendor);
                    return Err(());
                }
            }
        },
        None => {
            let _ = writeln!(log, "error: couldn't query CPU vendor");
            return Err(());
        }
    }

    match cpu_id.get_extended_function_info() {
        Some(features) => {
            if !features.has_execute_disable() {
                let _ = writeln!(log, "error: CPU doesn't support NX-bit");
                return Err(())
            }
        },
        None => {
            let _ = writeln!(log, "error: CPU extended function info query failed");
            return Err(())
        }
    }

    match cpu_id.get_feature_info() {
        Some(features) => {
            if !features.has_pae() {
                let _ = writeln!(log, "error: CPU doesn't support PAE");
                Err(())
            } else {
                Ok(())
            }
        },
        None => {
            let _ = writeln!(log, "error: CPU feature query failed");
            Err(())
        }
    }
}


fn enable_cpu_features() {
    use x86::controlregs::{Cr4, cr4_write, cr4};
    use x86::msr::{wrmsr, IA32_EFER, rdmsr};

    unsafe {
        cr4_write(Cr4::CR4_ENABLE_PAE | cr4());

        wrmsr(IA32_EFER, (1 << 11) | rdmsr(IA32_EFER)); // bit 11 enables NX-bit support
    }

}

use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let text_mode = unsafe {
        vga_text::new_vga_text_mode_unsafe()
    };

    let mut terminal = Terminal::new(text_mode, false);

    let _ = writeln!(terminal, "{:#?}", info);

    loop {
        unsafe {
            x86::halt()
        }
    }
}
