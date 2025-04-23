use crate::constants::PAGE_SIZE;
use crate::types::{RamMemory, RamAddress, RamAccess};

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

    pub fn is_readable(&self, from_address: RamAddress, to_address: RamAddress) -> bool {
        let from_page = from_address / PAGE_SIZE;
        let to_page = to_address / PAGE_SIZE;
        //println!("Checking readability from {} to {}", from_address, to_address);
        for page in from_page..=to_page {
            if let Some(page) = self.pages[page as usize].as_ref() {
                if page.flags.access.get(&RamAccess::Read).is_none() {
                    return false;
                }
            } else {
                return false;
            }
        }
        return true;
    }

    pub fn read(&self, address: RamAddress, num_bytes: RamAddress) -> Vec<u8> {
        let mut bytes = Vec::new();
        //println!("Reading {} bytes from address {}", num_bytes, address);
        for i in address..address + num_bytes {
            let page_target = i / PAGE_SIZE;
            let offset = i % PAGE_SIZE;
            if let Some(page) = self.pages[page_target as usize].as_ref() {
                bytes.push(page.data[offset as usize]);
            }
        }
        return bytes;
    }
}


