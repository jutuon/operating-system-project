
use bitflags::bitflags;

const GIBIBYTE: u64 = MIBIBYTE*1024;
const MIBIBYTE: u64 = 1024*1024;

use core::sync::atomic::{AtomicBool, Ordering};

static PAGE_TABLE_HANDLE_CREATED: AtomicBool = AtomicBool::new(false);

extern "C" {
    #[allow(improper_ctypes)]
    pub static READ_WRITE_PAGE_START_LOCATION: ();
}

#[repr(align(4096), C)] // 1024*4 = PAGE_TABLE_SIZE
pub struct PageTableData {
    level3: [L3PageTableEntry; 512],
    level2_1: [L2PageTableEntry2MB; 512],
    level2_2: [L2PageTableEntry2MB; 512],
    level2_3: [L2PageTableEntry2MB; 512],
    level2_4: [L2PageTableEntry2MB; 512],
}

#[used]
static mut PAGE_TABLE_DATA: PageTableData = PageTableData {
    level3: [L3PageTableEntry::zero(); 512],
    level2_1: [L2PageTableEntry2MB::zero(); 512],
    level2_2: [L2PageTableEntry2MB::zero(); 512],
    level2_3: [L2PageTableEntry2MB::zero(); 512],
    level2_4: [L2PageTableEntry2MB::zero(); 512],
};


pub struct GlobalPageTable {
    data: &'static mut PageTableData,
}

impl GlobalPageTable {
    pub unsafe fn new_unsafe() -> Self {
        Self {
            data: &mut PAGE_TABLE_DATA,
        }
    }

    pub fn new() -> Option<Self> {
        if PAGE_TABLE_HANDLE_CREATED.compare_and_swap(false, true, Ordering::SeqCst) {
            None
        } else {
            let mut page_table = unsafe { Self::new_unsafe() };
            page_table.load_identity_map();
            Some(page_table)
        }
    }

    pub fn load_identity_map(&mut self) {
        self.data.level3[0] = L3PageTableEntry::new(self.data.level2_1.as_ptr() as u64, L3Flags::PRESENT);
        self.data.level3[1] = L3PageTableEntry::new(self.data.level2_2.as_ptr() as u64, L3Flags::PRESENT);
        self.data.level3[2] = L3PageTableEntry::new(self.data.level2_3.as_ptr() as u64, L3Flags::PRESENT);
        self.data.level3[3] = L3PageTableEntry::new(self.data.level2_4.as_ptr() as u64, L3Flags::PRESENT);

        fn fill_page_table<F: EntryFlags>(mut flags: F, mut start_address: u64, table: &mut [GenericPageTableEntry<F, PhysicalAddressHandler2MBytesPDE>; 512], address_offset: u64, read_write_flag: F) {
            for entry in table.iter_mut() {
                unsafe {
                    if start_address >= &READ_WRITE_PAGE_START_LOCATION as *const () as u64 {
                        flags |= read_write_flag;
                    }
                }

                *entry = <GenericPageTableEntry<_, _>>::new(start_address, flags);
                start_address += address_offset;
            }
        }
        let flags = L2Flags2MB::PRESENT | L2Flags2MB::USER_SUPERVISOR;
        fill_page_table(flags, 0, &mut self.data.level2_1, MIBIBYTE*2, L2Flags2MB::READ_WRITE);
        fill_page_table(flags, GIBIBYTE, &mut self.data.level2_2, MIBIBYTE*2, L2Flags2MB::READ_WRITE);
        fill_page_table(flags, GIBIBYTE*2, &mut self.data.level2_3, MIBIBYTE*2, L2Flags2MB::READ_WRITE);
        fill_page_table(flags, GIBIBYTE*3, &mut self.data.level2_4, MIBIBYTE*2, L2Flags2MB::READ_WRITE);

        // Allow writing to VGA text buffer.
        self.data.level2_1[0] = <GenericPageTableEntry<_, _>>::new(0, flags | L2Flags2MB::READ_WRITE | L2Flags2MB::PAGE_LEVEL_CACHE_DISABLE);
    }

    pub fn level3_start_address(&self) -> usize {
        self.data.level3.as_ptr() as usize
    }
}

// Availible to software BITS 9-11

use core::marker::PhantomData;

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct GenericPageTableEntry<F: EntryFlags, A: PhysicalAddressHandler>(u64, PhantomData<F>, PhantomData<A>);

impl <F: EntryFlags, A: PhysicalAddressHandler> GenericPageTableEntry<F, A> {
    const fn zero() -> Self {
        GenericPageTableEntry(0, PhantomData, PhantomData)
    }
}

pub trait PhysicalAddressHandler {
    const MASK: u64;
    const MASK_STR: &'static str;
    const REQUIRED_FLAGS: u64;

    fn extract(entry: &u64) -> u64 {
        *entry & Self::MASK
    }

    fn to_entry_format(address: &u64) -> u64 {
        if *address & !Self::MASK != 0 {
            panic!("page table entry address contains additional bits than mask '{}', address = {}", Self::MASK_STR, *address);
        }
        *address | Self::REQUIRED_FLAGS
    }
}

pub trait EntryFlags: Sized + Copy + Clone + core::ops::Not<Output=Self> + core::ops::BitOr + core::ops::BitOrAssign {
    fn all() -> Self;
    fn bits(&self) -> u64;
    fn from_bits_truncate(value: u64) -> Self;
}

impl <F: EntryFlags, A: PhysicalAddressHandler> GenericPageTableEntry<F, A> {
    fn new(address: u64, flags: F) -> Self {
        let entry = A::to_entry_format(&address) | flags.bits();
        GenericPageTableEntry(entry, PhantomData, PhantomData)
    }

    pub fn flags(&self) -> F {
        F::from_bits_truncate(self.0)
    }

    pub fn flags_mut(&mut self, flags: F) {
        let mask = !F::all();
        self.0 = (self.0 & mask.bits()) | flags.bits();
    }

    pub fn address(&self) -> u64 {
        A::extract(&self.0)
    }

    pub fn address_mut(&mut self, address: u64) {
        let mask = !A::to_entry_format(&u64::max_value());
        self.0 = (self.0 & mask) | A::to_entry_format(&address);
    }
}

/// PDPE entry
pub type L3PageTableEntry = GenericPageTableEntry<L3Flags, PhysicalAddressHandlerNormal>;

bitflags! {
    pub struct L3Flags: u64 {
        const PRESENT = 1;
        const PAGE_LEVEL_WRITETHROUGH = 1 << 3;
        const PAGE_LEVEL_CACHE_DISABLE = 1 << 4;
    }
}

impl EntryFlags for L3Flags {
    fn all() -> Self { <Self>::all() }
    fn bits(&self) -> u64 { <Self>::bits(self) }
    fn from_bits_truncate(value: u64) -> Self { <Self>::from_bits_truncate(value)}
}


/// PDE entry
pub type L2PageTableEntry = GenericPageTableEntry<L2Flags, PhysicalAddressHandlerNormal>;

pub type L2PageTableEntry2MB = GenericPageTableEntry<L2Flags2MB, PhysicalAddressHandler2MBytesPDE>;

bitflags! {
    pub struct L2Flags: u64 {
        const PRESENT = 1;
        const READ_WRITE = 1 << 1;
        const USER_SUPERVISOR = 1 << 2;
        const PAGE_LEVEL_WRITETHROUGH = 1 << 3;
        const PAGE_LEVEL_CACHE_DISABLE = 1 << 4;
        const ACCESSED = 1 << 5;
        const NO_EXECUTE = 1 << 63;
    }
}

impl EntryFlags for L2Flags2MB {
    fn all() -> Self { <Self>::all() }
    fn bits(&self) -> u64 { <Self>::bits(self) }
    fn from_bits_truncate(value: u64) -> Self { <Self>::from_bits_truncate(value)}
}

bitflags! {
    pub struct L2Flags2MB: u64 {
        const PRESENT = 1;
        const READ_WRITE = 1 << 1;
        const USER_SUPERVISOR = 1 << 2;
        const PAGE_LEVEL_WRITETHROUGH = 1 << 3;
        const PAGE_LEVEL_CACHE_DISABLE = 1 << 4;
        const ACCESSED = 1 << 5;
        const DIRTY = 1 << 6;
        const GLOBAL_PAGE = 1 << 8;
        const PAGE_ATTRIBUTE_TABLE = 1 << 12;
        const NO_EXECUTE = 1 << 63;
    }
}

impl EntryFlags for L2Flags {
    fn all() -> Self { <Self>::all() }
    fn bits(&self) -> u64 { <Self>::bits(self) }
    fn from_bits_truncate(value: u64) -> Self { <Self>::from_bits_truncate(value)}
}

/// PTE entry
pub type L1PageTableEntry = GenericPageTableEntry<L1Flags, PhysicalAddressHandlerNormal>;


bitflags! {
    pub struct L1Flags: u64 {
        const PRESENT = 1;
        const READ_WRITE = 1 << 1;
        const USER_SUPERVISOR = 1 << 2;
        const PAGE_LEVEL_WRITETHROUGH = 1 << 3;
        const PAGE_LEVEL_CACHE_DISABLE = 1 << 4;
        const ACCESSED = 1 << 5;
        const DIRTY = 1 << 6;
        const PAGE_ATTRIBUTE_TABLE = 1 << 7;
        const GLOBAL_PAGE = 1 << 8;
        const NO_EXECUTE = 1 << 63;
    }
}

impl EntryFlags for L1Flags {
    fn all() -> Self { <Self>::all() }
    fn bits(&self) -> u64 { <Self>::bits(self) }
    fn from_bits_truncate(value: u64) -> Self { <Self>::from_bits_truncate(value)}
}

const PHYSICAL_ADDRESS_MASK_STR: &str = "(u64::max_value() << 12) >> 12";
const PHYSICAL_ADDRESS_MASK: u64 = (u64::max_value() << 12) >> 12;

const PHYSICAL_ADDRESS_MASK_2MB_PDE_STR: &str = "(u64::max_value() << 21) >> 12";
const PHYSICAL_ADDRESS_MASK_2MB_PDE: u64 = (u64::max_value() << 21) >> 12;

#[derive(Copy, Clone)]
pub struct PhysicalAddressHandlerNormal;

impl PhysicalAddressHandler for PhysicalAddressHandlerNormal {
    const MASK: u64 = PHYSICAL_ADDRESS_MASK;
    const MASK_STR: &'static str = PHYSICAL_ADDRESS_MASK_STR;
    const REQUIRED_FLAGS: u64 = 0;
}

const ENABLE_2MB_PAGES: u64 = 1 << 7;

#[derive(Copy, Clone)]
pub struct PhysicalAddressHandler2MBytesPDE;

impl PhysicalAddressHandler for PhysicalAddressHandler2MBytesPDE {
    const MASK: u64 = PHYSICAL_ADDRESS_MASK_2MB_PDE;
    const MASK_STR: &'static str = PHYSICAL_ADDRESS_MASK_2MB_PDE_STR;
    const REQUIRED_FLAGS: u64 = ENABLE_2MB_PAGES;
}

pub fn pae_cr3_format(address: usize, page_level_writethrough: bool, page_level_cache_disable: bool) -> u32 {
    let pointer_size = core::mem::size_of::<usize>();
    if core::mem::size_of::<usize>() != 4 {
        panic!("unsupported pointer size {} bytes", pointer_size);
    }

    let modulo = address % 32;

    if modulo != 0 {
        panic!("ERROR: modulo {}", modulo);
    }

    let mut result = address & (!0 << 5);

    if page_level_writethrough {
        result |= 1 << 3;
    }

    if page_level_cache_disable {
        result |= 1 << 4;
    }

    result as u32
}
