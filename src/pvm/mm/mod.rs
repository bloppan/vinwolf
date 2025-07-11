use crate::pvm::pvm_constants::{NUM_PAGES, PAGE_SIZE, LOWEST_ACCESIBLE_PAGE};
use crate::types::{RamMemory, RamAddress, RamAccess, Page};
pub mod program_init;

impl RamMemory {

    pub fn insert(&mut self, address: RamAddress, value: u8) {
        let page_target = address / PAGE_SIZE;
        let offset = address % PAGE_SIZE;
        //println!("Inserting value {} at address {}", value, address);
        if let Some(page) = self.pages[page_target as usize].as_mut() {
            //println!("Inserting value {} at address {}", value, address);
            page.data[offset as usize] = value;
        }
    }

    pub fn is_readable(&self, from_address: RamAddress, num_bytes: RamAddress) -> bool {

        if num_bytes == 0 {
            return true;
        }
        
        let from_page = from_address / PAGE_SIZE;
        let to_page = (from_address + num_bytes).saturating_sub(1) / PAGE_SIZE;

        self.access_page(from_page, to_page, RamAccess::Read)
    }

    pub fn read(&self, start_address: RamAddress, num_bytes: RamAddress) -> Vec<u8> {
        let mut bytes = Vec::new();
        //println!("Reading {} bytes from address {}", num_bytes, address);
        for i in start_address..start_address + num_bytes {
            let page_target = (i % RamAddress::MAX) / PAGE_SIZE;
            let offset = i % PAGE_SIZE;
            bytes.push(self.pages[page_target as usize].as_ref().unwrap().data[offset as usize])
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

    pub fn write(&mut self, start_address: RamAddress, bytes: Vec<u8>) {
        
        for i in start_address..start_address + bytes.len() as RamAddress {
            let page_target = (i % RamAddress::MAX) / PAGE_SIZE;
            let offset = i % PAGE_SIZE;
            self.pages[page_target as usize].as_mut().unwrap().data[offset as usize] = bytes[(i - start_address) as usize];
        }
    }

    fn access_page(&self, from_page: RamAddress, to_page: RamAddress, access: RamAccess) -> bool {

        for page in from_page..=to_page {

            // Check if the page is in the range of the highest inaccessible page (0xFFFF0000)
            if (page % NUM_PAGES) < LOWEST_ACCESIBLE_PAGE {
                println!("Page target {:?} out of bounds", page);
                // TODO
                return false;
            }

            if let Some(page) = self.pages[(page % NUM_PAGES) as usize].as_ref() {
                if page.flags.access.get(&access).is_none() {
                    // TODO no access 
                    return false;
                }
            } else {
                // TODO page fault
                println!("page_fault: page {:?}", page);
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
            new_page.as_mut().unwrap().flags.access.insert(RamAccess::Write);
            new_page.as_mut().unwrap().flags.access.insert(RamAccess::Read);
            self.pages[(page % NUM_PAGES) as usize] = new_page;
        }

        return true;
    }   

}


