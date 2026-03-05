use std::collections::HashMap;

const DISK_SIZE: usize = 50; // as per example slide (can be changed to 1000)

#[derive(Debug, Clone)]
pub enum AllocType { Contiguous, Linked, Indexed }

impl std::fmt::Display for AllocType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AllocType::Contiguous => write!(f, "contiguous"),
            AllocType::Linked     => write!(f, "linked"),
            AllocType::Indexed    => write!(f, "indexed"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name:        String,
    pub alloc_type:  AllocType,
    pub blocks:      Vec<usize>, // all block numbers used
    pub index_block: Option<usize>, // for Indexed allocation
}

pub struct DiskManager {
    pub disk:     Vec<bool>, // true = occupied
    pub files:    HashMap<String, FileEntry>,
    pub alloc_mode: Option<AllocType>,
}

impl DiskManager {
    pub fn new() -> Self {
        DiskManager {
            disk:       vec![false; DISK_SIZE],
            files:      HashMap::new(),
            alloc_mode: None,
        }
    }

    pub fn init(&mut self, mode: &str) {
        self.disk = vec![false; DISK_SIZE];
        self.files.clear();
        self.alloc_mode = match mode.to_lowercase().as_str() {
            "linked"  => Some(AllocType::Linked),
            "indexed" => Some(AllocType::Indexed),
            _         => Some(AllocType::Contiguous),
        };
        let mode_name = self.alloc_mode.as_ref().unwrap().to_string().to_uppercase();
        println!("[INIT] Disk blocks={} Allocation={}", DISK_SIZE, mode_name);
        println!("Commands:");
        println!("  create <file> <blocks>");
        println!("  delete <file>");
        println!("  ls");
        println!("  map <file>");
    }

    pub fn create(&mut self, name: &str, size: usize) -> Result<(), String> {
        if self.files.contains_key(name) {
            return Err(format!("File '{}' already exists", name));
        }

        match &self.alloc_mode.clone().unwrap_or(AllocType::Contiguous) {
            AllocType::Contiguous => self.create_contiguous(name, size),
            AllocType::Linked     => self.create_linked(name, size),
            AllocType::Indexed    => self.create_indexed(name, size),
        }
    }

    fn create_contiguous(&mut self, name: &str, size: usize) -> Result<(), String> {
        // Find a contiguous run with enough free space
        let start = self.find_contiguous(size)
            .ok_or_else(|| format!("No contiguous space for {} blocks", size))?;

        let blocks: Vec<usize> = (start..start + size).collect();
        for &b in &blocks { self.disk[b] = true; }

        println!("[OK] Created {} blocks={} start={}..{}", name, size, start, start + size - 1);
        self.files.insert(name.to_string(), FileEntry {
            name:        name.to_string(),
            alloc_type:  AllocType::Contiguous,
            blocks,
            index_block: None,
        });
        Ok(())
    }

    fn create_linked(&mut self, name: &str, size: usize) -> Result<(), String> {
        let free: Vec<usize> = self.disk.iter().enumerate()
            .filter(|&(_, &used)| !used)
            .map(|(i, _)| i)
            .take(size)
            .collect();

        if free.len() < size {
            return Err(format!("Not enough free blocks (need {}, available {})", size, free.len()));
        }

        for &b in &free { self.disk[b] = true; }

        println!("[OK] Created {} blocks={} (linked): {:?}", name, size, free);
        self.files.insert(name.to_string(), FileEntry {
            name:        name.to_string(),
            alloc_type:  AllocType::Linked,
            blocks:      free,
            index_block: None,
        });
        Ok(())
    }

    fn create_indexed(&mut self, name: &str, size: usize) -> Result<(), String> {
        // Need size + 1 blocks (1 index block + size data blocks)
        let free: Vec<usize> = self.disk.iter().enumerate()
            .filter(|&(_, &used)| !used)
            .map(|(i, _)| i)
            .take(size + 1)
            .collect();

        if free.len() < size + 1 {
            return Err(format!("Not enough free blocks (need {}, available {})", size + 1, free.len()));
        }

        let index_block = free[0];
        let data_blocks = free[1..].to_vec();

        for &b in &free { self.disk[b] = true; }

        println!("[OK] Created {} index_block={} data_blocks={:?}", name, index_block, data_blocks);
        self.files.insert(name.to_string(), FileEntry {
            name:        name.to_string(),
            alloc_type:  AllocType::Indexed,
            blocks:      data_blocks,
            index_block: Some(index_block),
        });
        Ok(())
    }

    pub fn delete(&mut self, name: &str) -> Result<(), String> {
        let entry = self.files.remove(name)
            .ok_or_else(|| format!("File '{}' not found", name))?;

        for &b in &entry.blocks { self.disk[b] = false; }
        if let Some(ib) = entry.index_block { self.disk[ib] = false; }

        println!("[OK] Deleted '{}'", name);
        Ok(())
    }

    pub fn ls(&self) {
        if self.files.is_empty() {
            println!("  (no files)");
        } else {
            println!("FILES:");
            for (_, f) in &self.files {
                match f.alloc_type {
                    AllocType::Contiguous => {
                        let start = f.blocks[0];
                        let end   = f.blocks[f.blocks.len() - 1];
                        println!("  - {} contiguous [{}..{}]", f.name, start, end);
                    }
                    AllocType::Linked => {
                        println!("  - {} linked {:?}", f.name, f.blocks);
                    }
                    AllocType::Indexed => {
                        println!("  - {} indexed (idx={}) data={:?}",
                                 f.name, f.index_block.unwrap_or(0), f.blocks);
                    }
                }
            }
        }
        let free = self.disk.iter().filter(|&&b| !b).count();
        println!("Free blocks: {}", free);
    }

    pub fn map(&self, name: &str) -> Result<(), String> {
        let f = self.files.get(name)
            .ok_or_else(|| format!("File '{}' not found", name))?;

        let all: Vec<String> = f.blocks.iter().map(|b| b.to_string()).collect();
        print!("{} -> ", name);
        if let Some(ib) = f.index_block {
            print!("[idx={}] ", ib);
        }
        println!("{}", all.join(" "));
        Ok(())
    }

    // Find contiguous free blocks
    fn find_contiguous(&self, size: usize) -> Option<usize> {
        let mut count = 0;
        let mut start = 0;
        for (i, &used) in self.disk.iter().enumerate() {
            if !used {
                if count == 0 { start = i; }
                count += 1;
                if count == size { return Some(start); }
            } else {
                count = 0;
            }
        }
        None
    }
}