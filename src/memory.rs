use core::panic;

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{structures::paging::{page_table::FrameError, FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB}, PhysAddr, VirtAddr};

pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;
    let (level_4_table_frame, _) = Cr3::read();
    let phy = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phy.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

pub struct EmptyFrameAllocator;
pub struct BootInfoFrameAllocator{
    memory_map:&'static MemoryMap,
    next:usize
}

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        None
    }
}

impl BootInfoFrameAllocator{
    pub unsafe fn init(memory_map:&'static MemoryMap)->Self{
        BootInfoFrameAllocator{
            memory_map,
            next:0
        }
    }

    fn usable_frames(&self)->impl Iterator<Item = PhysFrame>{
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r|r.region_type == MemoryRegionType::Usable);

        let addr_ranges = usable_regions.map(|r| r.range.start_addr() .. r.range.end_addr());
        let frame_address = addr_ranges.flat_map(|r|r.step_by(4096));
        frame_address.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}
unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator{
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next+1;
        frame
    }
}
pub unsafe fn init(physical_memory_offset: VirtAddr)->OffsetPageTable<'static>{
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    translate_addr_inner(addr, physical_memory_offset)
}

fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    use x86_64::registers::control::Cr3;
    // first to read the level4 table from the cr3 registry
    let (level_4_table_frame, _) = Cr3::read();
    let table_index = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];

    let mut frame = level_4_table_frame;

    // loop the table index and get the physical memory
    for &index in &table_index {
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*table_ptr };

        let entry = &table[index];
        
        frame = match entry.frame() {
            Ok(frame)=>frame,
            Err(FrameError::FrameNotPresent)=> return None,
            Err(FrameError::HugeFrame)=> panic!("huge page not support"),
        };
    }
    Some(frame.start_address()+ u64::from(addr.page_offset()))
}


pub fn create_example_mapping(page:Page,mapper:&mut OffsetPageTable,frame_allocator:&mut impl FrameAllocator<Size4KiB>){
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to faild").flush();
}

