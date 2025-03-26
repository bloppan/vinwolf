use crate::constants::PAGE_SIZE;
use crate::types::{RamMemory, RamAddress};

impl RamMemory {
    pub fn insert(&mut self, address: RamAddress, value: u8) {
        let page_target = address / PAGE_SIZE;
        let offset = address % PAGE_SIZE;
        println!("Inserting value {} at address {}", value, address);
        if let Some(page) = self.pages[page_target as usize].as_mut() {
            println!("Inserting value {} at address {}", value, address);
            page.data[offset as usize] = value;
        }
    }
}


