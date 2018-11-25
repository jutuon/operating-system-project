#![feature(global_asm)]

#![no_std]
#![no_main]

global_asm!(include_str!("assembly.s"));

pub mod vga_text;
pub mod terminal;
pub mod gdt;
pub mod idt;

use self::terminal::{Terminal};
use self::gdt::GDT;
use self::idt::IDT;

use core::fmt::Write;

#[no_mangle]
extern "C" fn kernel_main() -> ! {
    let mut vga_handle = vga_text::VgaTextBuffer::new().unwrap();
    vga_handle.clear();

    let mut terminal = Terminal::new(vga_handle);

    writeln!(terminal, "Hello world");

    check_cpu_features(&mut terminal).expect("error: CPU is not compatible");

    enable_cpu_features();

    writeln!(terminal, "PAE and NX-bit enabled.");

    GDT::load_gdt();

    writeln!(terminal, "GDT loaded.");

    IDT::load_idt();

    writeln!(terminal, "IDT loaded.");

    IDT::enable_interrupts();

    writeln!(terminal, "Interrupts enabled.");
    loop {
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
                    writeln!(log, "error: unknown CPU vendor '{}'", unknown_vendor);
                    return Err(());
                }
            }
        },
        None => {
            writeln!(log, "error: couldn't query CPU vendor");
            return Err(());
        }
    }

    match cpu_id.get_extended_function_info() {
        Some(features) => {
            if !features.has_execute_disable() {
                writeln!(log, "error: CPU doesn't support NX-bit");
                return Err(())
            }
        },
        None => {
            writeln!(log, "error: CPU extended function info query failed");
            return Err(())
        }
    }

    match cpu_id.get_feature_info() {
        Some(features) => {
            if !features.has_pae() {
                writeln!(log, "error: CPU doesn't support PAE");
                Err(())
            } else {
                Ok(())
            }
        },
        None => {
            writeln!(log, "error: CPU feature query failed");
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
    let text_buffer = unsafe {
        vga_text::VgaTextBuffer::new_unsafe()
    };

    let mut terminal = Terminal::new(text_buffer);

    writeln!(terminal, "{:#?}", info);

    loop {
        unsafe {
            x86::halt()
        }
    }
}
