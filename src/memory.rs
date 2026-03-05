use std::collections::HashMap;

const TOTAL_FRAMES: usize = 256; // 1024 KB / 4 KB per page
pub const PAGE_SIZE: u32 = 4096; // 4 KB

/// Page Table Entry: VPN → PFN
#[derive(Debug, Clone)]
pub struct PageTableEntry {
    pub vpn: u32,
    pub pfn: u32,
}

pub struct MemoryManager {
    /// frame_map[pfn] = Some(pid) if in use
    frame_map: Vec<Option<u32>>,
    /// page_table[pid] = list of VPN→PFN mappings
    page_tables: HashMap<u32, Vec<PageTableEntry>>,
    pub enabled: bool,
}

impl MemoryManager {
    pub fn new() -> Self {
        MemoryManager {
            frame_map: vec![None; TOTAL_FRAMES],
            page_tables: HashMap::new(),
            enabled: false,
        }
    }

    pub fn init(&mut self) {
        self.enabled = true;
        self.frame_map = vec![None; TOTAL_FRAMES];
        self.page_tables.clear();
        println!("[INIT] Memory=1024KB PageSize=4KB Frames={}", TOTAL_FRAMES);
        println!("[OK] Paging enabled (simulation)");
        println!("Commands:");
        println!("  alloc <pid> <kb>");
        println!("  translate <pid> <logical_addr>");
        println!("  free <pid>");
    }

    /// Allocate memory for pid of size_kb KB
    pub fn alloc(&mut self, pid: u32, size_kb: u32) -> Result<(), String> {
        let pages_needed = (size_kb as usize + 3) / 4; // ceil(size_kb / 4)

        // Find free frames
        let free_frames: Vec<usize> = self.frame_map
            .iter()
            .enumerate()
            .filter(|(_, v)| v.is_none())
            .map(|(i, _)| i)
            .take(pages_needed)
            .collect();

        if free_frames.len() < pages_needed {
            return Err(format!(
                "Not enough free frames (need {} frames, available {})",
                pages_needed, free_frames.len()
            ));
        }

        let entries: Vec<PageTableEntry> = free_frames
            .iter()
            .enumerate()
            .map(|(vpn, &pfn)| {
                self.frame_map[pfn] = Some(pid);
                PageTableEntry { vpn: vpn as u32, pfn: pfn as u32 }
            })
            .collect();

        println!("[OK] Alloc PID={} size={}KB => pages={}", pid, size_kb, pages_needed);
        println!("PageTable(P{}):", pid);
        for e in &entries {
            println!("  VPN {} -> PFN {}", e.vpn, e.pfn);
        }

        self.page_tables.insert(pid, entries);
        Ok(())
    }

    /// Translate Logical Address to Physical Address
    pub fn translate(&self, pid: u32, logical_addr: u32) -> Result<(), String> {
        let table = self.page_tables.get(&pid)
            .ok_or_else(|| format!("Page table not found for PID={}", pid))?;

        let vpn    = logical_addr / PAGE_SIZE;
        let offset = logical_addr % PAGE_SIZE;

        let entry = table.iter().find(|e| e.vpn == vpn)
            .ok_or_else(|| format!("VPN={} not found in page table of PID={}", vpn, pid))?;

        let physical_addr = entry.pfn * PAGE_SIZE + offset;

        println!("Translate PID={} LA={}", pid, logical_addr);
        println!("  PageSize={} => VPN={} Offset={}", PAGE_SIZE, vpn, offset);
        println!("  PFN={} => PA = {}*{} + {} = {}", entry.pfn, entry.pfn, PAGE_SIZE, offset, physical_addr);
        println!("[OK] Physical Address = {} (bytes)", physical_addr);
        Ok(())
    }

    /// Free all frames of pid
    pub fn free(&mut self, pid: u32) -> Result<(), String> {
        let table = self.page_tables.remove(&pid)
            .ok_or_else(|| format!("Page table not found for PID={}", pid))?;

        for e in &table {
            self.frame_map[e.pfn as usize] = None;
        }
        println!("[OK] Freed {} frames from PID={}", table.len(), pid);
        Ok(())
    }

    pub fn show_frames(&self) {
        let used = self.frame_map.iter().filter(|f| f.is_some()).count();
        println!("Frames used: {}/{}", used, TOTAL_FRAMES);
    }
}