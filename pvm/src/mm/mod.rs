use constants::pvm::{NUM_PAGES, PAGE_SIZE, LOWEST_ACCESIBLE_PAGE};
use crate::pvm_types::{RamMemory, RamAddress, PageFlags, RamAccess, Page};
pub mod program_init;
use utils::log;

impl RamMemory {

    /*pub fn insert(&mut self, address: RamAddress, value: u8) {
        let page_target = address / PAGE_SIZE;
        let offset = address % PAGE_SIZE;
        //println!("Inserting value {} at address {}", value, address);
        if let Some(page) = self.pages[page_target as usize].as_mut() {
            //println!("Inserting value {} at address {}", value, address);
            page.data[offset as usize] = value;
        }
    }*/

    pub fn is_readable(&self, from_address: RamAddress, num_bytes: RamAddress) -> bool {

        if num_bytes == 0 {
            return true;
        }
        
        let from_page = from_address / PAGE_SIZE;
        let to_page = (from_address + num_bytes).saturating_sub(1) / PAGE_SIZE;

        self.access_page(from_page, to_page, RamAccess::Read)
    }

    pub fn read(&self, start_address: RamAddress, num_bytes: RamAddress) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(num_bytes as usize);
        let mut current_address = start_address;

        while current_address < start_address + num_bytes {
            let page_target = current_address / PAGE_SIZE;
            let offset = current_address % PAGE_SIZE;
            let remaining_in_page = PAGE_SIZE - offset;
            let bytes_to_read = std::cmp::min(remaining_in_page, start_address + num_bytes - current_address);

            let page = self.pages[page_target as usize].as_ref().unwrap();
            let slice = &page.data[offset as usize..(offset + bytes_to_read) as usize];
            bytes.extend_from_slice(slice);

            current_address += bytes_to_read;
        }

        return bytes;
    }

    pub fn is_writable(&self, from_address: RamAddress, num_bytes: RamAddress) -> bool {

        if num_bytes == 0 {
            return true;
        }
        
        let from_page = from_address / PAGE_SIZE;
        let to_page = (from_address + num_bytes).saturating_sub(1) / PAGE_SIZE;
        
        self.access_page(from_page, to_page, RamAccess::Write)
    }

    pub fn write(&mut self, start_address: RamAddress, data: &[u8]) {
        let mut current_address = start_address;
        let mut data_idx = 0;

        while data_idx < data.len() {
            let page_target = current_address / PAGE_SIZE;
            let offset = current_address % PAGE_SIZE;
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

    fn access_page(&self, from_page: RamAddress, to_page: RamAddress, access: RamAccess) -> bool {

        for page in from_page..=to_page {

            // Check if the page is in the range of the highest inaccessible page (0xFFFF0000)
            if (page % NUM_PAGES) < LOWEST_ACCESIBLE_PAGE {
                log::error!("Page target {:?} out of bounds", page);
                // TODO
                return false;
            }

            if let Some(page) = self.pages[(page % NUM_PAGES) as usize].as_ref() {
                match access {
                    RamAccess::Read => {
                        if !page.flags.read_access {
                            return false;
                        }
                    },
                    RamAccess::Write => {
                        if !page.flags.write_access {
                            return false;
                        }
                    }
                }
            } else {
                // TODO page fault
                log::error!("page_fault: page {:?}", page);
                return false;
            }
        }

        return true;
    }

    pub fn allocate_pages(&mut self, from_page: RamAddress, count: RamAddress) -> bool {

        let to_page = from_page + count;

        /*if !self.access_page(from_page, to_page, RamAccess::Write) {
                println!("allocate page: no access from_page {:?} to_page {:?}", from_page, to_page);
                return false;
        }*/

        for page in from_page..=to_page {
            //println!("allocate page: {:?}", page);
            let mut new_page = Some(Page::default());
            new_page.as_mut().unwrap().flags.read_access = true;
            new_page.as_mut().unwrap().flags.write_access = true;
            self.pages[(page % NUM_PAGES) as usize] = new_page;
        }

        return true;
    }   

}


