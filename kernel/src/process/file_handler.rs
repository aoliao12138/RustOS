use alloc::{boxed::Box, sync::Arc};
use core::fmt::{Debug, Error, Formatter};

use simple_filesystem::INode;
use ucore_memory::paging::{PageTable, InactivePageTable};

use crate::memory::{FrameAllocator, GlobalFrameAlloc, KernelStack, MemoryArea, MemoryAttr, MemoryHandler, MemorySet};

/// Delay mapping a page to an area of a file.
#[derive(Clone)]
pub struct FileHandler {
    pub inode: Arc<INode>,
    pub mem_start: usize,
    pub file_start: usize,
    pub file_end: usize,
    pub flags: MemoryAttr,
    pub allocator: GlobalFrameAlloc,
}

impl MemoryHandler for FileHandler {
    fn box_clone(&self) -> Box<MemoryHandler> {
        Box::new(self.clone())
    }

    fn map(&self, pt: &mut PageTable, addr: usize) {
        let entry = pt.map(addr, 0);
        entry.set_present(false);
        entry.update();
    }

    fn unmap(&self, pt: &mut PageTable, addr: usize) {
        let entry = pt.get_entry(addr).expect("failed to get entry");
        if entry.present() {
            self.allocator.dealloc(entry.target());
        }
        pt.unmap(addr);
    }

    fn page_fault_handler(&self, pt: &mut PageTable, addr: usize) -> bool {
        let addr = addr & !(PAGE_SIZE - 1);
        let entry = pt.get_entry(addr).expect("failed to get entry");
        if entry.present() {
            return false;
        }
        let frame = self.allocator.alloc().expect("failed to alloc frame");
        entry.set_target(frame);
        self.flags.apply(entry);
        let data = pt.get_page_slice_mut(addr);
        let file_offset = addr + self.file_start - self.mem_start;
        let read_size = (self.file_end as isize - file_offset as isize)
            .min(PAGE_SIZE as isize).max(0) as usize;
        if read_size != 0 {
            let len = self.inode.read_at(file_offset, &mut data[..read_size]).unwrap();
            assert_eq!(len, read_size);
        }
        if read_size != PAGE_SIZE {
            data[read_size..].iter_mut().for_each(|x| *x = 0);
        }
        true
    }
}

impl Debug for FileHandler {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "FileHandler")
    }
}

const PAGE_SIZE: usize = 0x1000;