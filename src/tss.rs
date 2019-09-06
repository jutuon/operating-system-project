use x86::bits32::task::TaskStateSegment;
use x86::segmentation::*;
use x86::task::load_tr;

#[repr(transparent)]
pub struct TSS {
    _start: TaskStateSegment,
}

#[used]
pub static TSS_DATA: TSS = TSS {
    _start: TaskStateSegment::new()
};


pub struct KernelTask;

impl KernelTask {
    pub fn load() -> Self {
        unsafe {
            load_tr(SegmentSelector::new(3, x86::Ring::Ring0));
        }
        KernelTask
    }
}
