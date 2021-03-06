
use x86::segmentation::*;
use x86::dtables::*;


use crate::tss::{TSS_DATA, TSS};

#[repr(C, packed)]
pub struct GDT {
    _null: Descriptor,
    code: Descriptor,
    data: Descriptor,
    task: Descriptor,
}

#[used]
static mut GDT_DATA: GDT = GDT {
    _null: Descriptor::NULL,
    code: Descriptor::NULL,
    data: Descriptor::NULL,
    task: Descriptor::NULL,
};

impl GDT {
    pub fn load_gdt() {
        let code = DescriptorBuilder::code_descriptor(0, u32::max_value(), CodeSegmentType::ExecuteRead)
            .limit_granularity_4kb()
            .present()
            .db()
            .dpl(x86::Ring::Ring0)
            .finish();
        let data = DescriptorBuilder::data_descriptor(0, u32::max_value(), DataSegmentType::ReadWrite)
            .limit_granularity_4kb()
            .present()
            .db()
            .dpl(x86::Ring::Ring0)
            .finish();
        let task = <DescriptorBuilder as GateDescriptorBuilder<u32>>::tss_descriptor(&TSS_DATA as *const _ as u64, core::mem::size_of::<TSS>() as u64, true)
            .present()
            .dpl(x86::Ring::Ring0)
            .finish();

        unsafe {
            GDT_DATA.code = code;
            GDT_DATA.data = data;
            GDT_DATA.task = task;
        }

        let code_segment_selector = SegmentSelector::new(1, x86::Ring::Ring0);
        let data_and_stack_segment_selector = SegmentSelector::new(2, x86::Ring::Ring0);

        unsafe {
            let pointer = DescriptorTablePointer::new(&GDT_DATA);
            lgdt(&pointer);

            // Set LDTR to null, because it is not used.
            load_ldtr(SegmentSelector::new(0, x86::Ring::Ring0));

            x86::bits32::segmentation::load_cs(code_segment_selector);
            load_ds(data_and_stack_segment_selector);
            load_ss(data_and_stack_segment_selector);
            load_es(data_and_stack_segment_selector);
            load_fs(data_and_stack_segment_selector);
            load_gs(data_and_stack_segment_selector);
        }
    }

}
