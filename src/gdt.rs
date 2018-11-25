
use x86::segmentation::*;
use x86::dtables::*;

pub struct GDT {
    null: Descriptor,
    code: Descriptor,
    data: Descriptor,
}

#[used]
static mut GDT_DATA: GDT = GDT {
    null: Descriptor::NULL,
    code: Descriptor::NULL,
    data: Descriptor::NULL,
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

        unsafe {
            GDT_DATA.code = code;
            GDT_DATA.data = data;
        }

        let code_segment_selector = SegmentSelector::new(1, x86::Ring::Ring0);
        let data_and_stack_segment_selector = SegmentSelector::new(2, x86::Ring::Ring0);

        unsafe {
            let pointer = DescriptorTablePointer::new(&GDT_DATA);
            lgdt(&pointer);
            x86::bits32::segmentation::load_cs(code_segment_selector);
            load_ds(data_and_stack_segment_selector);
            load_ss(data_and_stack_segment_selector);
            load_es(SegmentSelector::RPL_0);
            load_fs(SegmentSelector::RPL_0);
            load_gs(SegmentSelector::RPL_0);
        }
    }

}
