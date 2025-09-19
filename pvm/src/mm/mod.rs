use constants::pvm::{PAGE_SIZE, LOWEST_ACCESIBLE_PAGE};
use crate::pvm_types::{ExitReason, Page, RamAccess, RamAddress, RamMemory};
pub mod program_init;
use utils::log;
use crate::{mem_bounds, page_index, page_offset};

impl RamMemory {

    pub fn is_readable(&self, from_address: RamAddress, num_bytes: RamAddress) -> Result<(), ExitReason> {

        if num_bytes == 0 {
            return Ok(());
        }
        
        let from_page = page_index!(from_address);
        let to_page = page_index!((from_address + num_bytes).saturating_sub(1));

        self.access_page(from_page, to_page, RamAccess::Read)
    }

    pub fn read(&self, start_address: RamAddress, num_bytes: RamAddress) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(num_bytes as usize);
        let mut current_address = start_address;

        while current_address < start_address + num_bytes {
            let page_target = page_index!(current_address);
            let offset = page_offset!(current_address);
            let remaining_in_page = PAGE_SIZE - offset;
            let bytes_to_read = std::cmp::min(remaining_in_page, start_address + num_bytes - current_address);

            let page = self.pages[page_target as usize].as_ref().unwrap();
            let slice = &page.data[offset as usize..(offset + bytes_to_read) as usize];
            bytes.extend_from_slice(slice);

            current_address += bytes_to_read;
        }

        return bytes;
    }

    pub fn is_writable(&self, from_address: RamAddress, num_bytes: RamAddress) -> Result<(), ExitReason> {

        if num_bytes == 0 {
            return Ok(());
        }
        
        let from_page = page_index!(from_address);
        let to_page = page_index!((from_address + num_bytes).saturating_sub(1));
        
        self.access_page(from_page, to_page, RamAccess::Write)
    }

    pub fn write(&mut self, start_address: RamAddress, data: &[u8]) {
        let mut current_address = start_address;
        let mut data_idx = 0;

        while data_idx < data.len() {
            let page_target = page_index!(current_address);
            let offset = page_offset!(current_address);
            let remaining_in_page = PAGE_SIZE - offset;
            let bytes_to_write = std::cmp::min(remaining_in_page as usize, data.len() - data_idx);
            let page = self.pages[page_target as usize].as_mut().unwrap();

            page.flags.modified = true;
            let dest_slice = &mut page.data[offset as usize..(offset + remaining_in_page) as usize];
            dest_slice[..bytes_to_write].copy_from_slice(&data[data_idx..data_idx + bytes_to_write]);
            current_address += bytes_to_write as RamAddress;
            data_idx += bytes_to_write;
        }
    }

    fn access_page(&self, from_page: RamAddress, to_page: RamAddress, access: RamAccess) -> Result<(), ExitReason> {

        for page_idx in from_page..=to_page {

            // Check if the page is in the range of the highest inaccessible page (0xFFFF0000)
            if (mem_bounds!(page_idx)) < LOWEST_ACCESIBLE_PAGE {
                log::error!("Page target {:?} out of bounds", page_idx);
                // TODO
                return Err(ExitReason::PageFault(page_idx * PAGE_SIZE));
            }

            if let Some(page) = self.pages[(mem_bounds!(page_idx)) as usize].as_ref() {
                match access {
                    RamAccess::Read => {
                        if !page.flags.read_access {
                            return Err(ExitReason::PageFault(page_idx * PAGE_SIZE));
                        }
                    },
                    RamAccess::Write => {
                        if !page.flags.write_access {
                            return Err(ExitReason::PageFault(page_idx * PAGE_SIZE));
                        }
                    }
                }
            } else {
                log::error!("page_fault: page {:?}", page_idx);
                return Err(ExitReason::PageFault(page_idx * PAGE_SIZE));
            }
        }

        Ok(())
    }

    pub fn allocate_pages(&mut self, from_page: RamAddress, count: RamAddress) -> bool {

        let to_page = from_page + count;

        // TODO check access?
        /*if !self.access_page(from_page, to_page, RamAccess::Write) {
                println!("allocate page: no access from_page {:?} to_page {:?}", from_page, to_page);
                return false;
        }*/

        for page in from_page..=to_page {
            //println!("allocate page: {:?}", page);
            let mut new_page = Some(Page::default());
            new_page.as_mut().unwrap().flags.read_access = true;
            new_page.as_mut().unwrap().flags.write_access = true;
            self.pages[(mem_bounds!(page)) as usize] = new_page;
        }

        return true;
    }   

}


