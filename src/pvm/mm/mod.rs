use std::collections::HashMap;

use crate::constants::{PAGE_SIZE, NUM_PAGES};
use crate::types::{
    PageFlags, Page, PageMap, Program, PageTable, RamMemory, RamAddress, RegSize, Context, ExitReason, MemoryChunk
};


impl Default for PageTable {
    fn default() -> Self {
        PageTable {
            pages: HashMap::new(),
        }
    }
}

impl Default for Page {
    fn default() -> Self {
        Page {
            flags: PageFlags::default(),
            data: Box::new([0u8; PAGE_SIZE as usize]),
        }
    }
}

impl Default for PageFlags {
    fn default() -> Self {
        PageFlags {
            is_writable: false,
            referenced: false,
            modified: false,
        }
    }
}

impl Default for PageMap {
    fn default() -> Self {
        PageMap {
            address: 0,
            length: 0,
            is_writable: false,
        }
    }
}

impl Default for RamMemory {
    fn default() -> Self {
        let mut v: Vec<Option<Page>> = Vec::with_capacity(NUM_PAGES as usize);
        for _ in 0..NUM_PAGES {
            v.push(None);
        }
        RamMemory {
            pages: v.into_boxed_slice(),
        }
    }
}

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


impl Default for MemoryChunk {
    fn default() -> Self {
        MemoryChunk {
            address: 0,
            contents: vec![],
        }
    }
}