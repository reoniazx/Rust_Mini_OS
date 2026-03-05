// ─────────────────────────────────────────────────────────────
//  I/O & Deadlock Simulation
//
//  จำลอง Resource Allocation Graph (RAG)
//  - Process ขอ resource → ถ้าว่างให้เลย, ถ้าไม่ว่างรอ
//  - ตรวจ Deadlock ด้วย Cycle Detection
//  - Banker's Algorithm (Safety Check)
// ─────────────────────────────────────────────────────────────

use std::collections::HashMap;

// ─── I/O Device ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct IoDevice {
    pub instances: usize,     // จำนวน instance ทั้งหมด
    pub available: usize,     // instance ที่ยังว่าง
    pub held_by: Vec<u32>,    // pid ที่ถือครองอยู่
    pub wait_queue: Vec<u32>, // pid ที่รอ
}

impl IoDevice {
    pub fn new(instances: usize) -> Self {
        IoDevice {
            instances,
            available: instances,
            held_by: Vec::new(),
            wait_queue: Vec::new(),
        }
    }
}

// ─── Deadlock Manager ────────────────────────────────────────

pub struct DeadlockManager {
    pub devices: HashMap<String, IoDevice>,
    /// allocation[pid][device] = จำนวนที่ถือครอง
    pub allocation: HashMap<u32, HashMap<String, usize>>,
    /// request[pid][device] = จำนวนที่รอ
    pub request: HashMap<u32, HashMap<String, usize>>,
    /// max_need[pid][device] = ต้องการสูงสุด (Banker's)
    pub max_need: HashMap<u32, HashMap<String, usize>>,
    pub processes: Vec<u32>,
}

impl DeadlockManager {
    pub fn new() -> Self {
        DeadlockManager {
            devices: HashMap::new(),
            allocation: HashMap::new(),
            request: HashMap::new(),
            max_need: HashMap::new(),
            processes: Vec::new(),
        }
    }

    // ── add_device ───────────────────────────────────────────
    /// add_device <name> <instances>
    pub fn add_device(&mut self, name: &str, instances: usize) {
        self.devices
            .insert(name.to_string(), IoDevice::new(instances));
        println!("[OK] เพิ่ม device '{}' จำนวน {} instance", name, instances);
    }

    // ── add_process ──────────────────────────────────────────
    /// ลงทะเบียน process และกำหนด max need (Banker's)
    /// max_str เช่น "printer:2,disk:1"
    pub fn add_process(&mut self, pid: u32, max_str: &str) {
        if !self.processes.contains(&pid) {
            self.processes.push(pid);
        }
        self.allocation.entry(pid).or_default();
        self.request.entry(pid).or_default();

        let max = self.max_need.entry(pid).or_default();
        for part in max_str.split(',') {
            let kv: Vec<&str> = part.trim().splitn(2, ':').collect();
            if kv.len() == 2 {
                if let Ok(n) = kv[1].parse::<usize>() {
                    max.insert(kv[0].to_string(), n);
                }
            }
        }
        println!("[OK] เพิ่ม P{} max_need={}", pid, max_str);
    }

    // ── request_resource ─────────────────────────────────────
    /// process ขอ resource
    pub fn request_resource(&mut self, pid: u32, device: &str, amount: usize) {
        let dev = match self.devices.get_mut(device) {
            Some(d) => d,
            None => {
                println!("[ERR] ไม่พบ device '{}'", device);
                return;
            }
        };

        if dev.available >= amount {
            // ให้ทันที
            dev.available -= amount;
            for _ in 0..amount {
                dev.held_by.push(pid);
            }

            *self
                .allocation
                .entry(pid)
                .or_default()
                .entry(device.to_string())
                .or_insert(0) += amount;

            println!(
                "[IO]  P{} ได้รับ '{}' x{} → available={}",
                pid, device, amount, dev.available
            );
        } else {
            // ต้องรอ → บันทึกใน request
            dev.wait_queue.push(pid);
            *self
                .request
                .entry(pid)
                .or_default()
                .entry(device.to_string())
                .or_insert(0) += amount;

            println!(
                "[IO]  P{} รอ '{}' x{} (available={} ไม่พอ) → เพิ่มใน wait queue",
                pid, device, amount, dev.available
            );
        }
    }

    // ── release_resource ─────────────────────────────────────
    pub fn release_resource(&mut self, pid: u32, device: &str, amount: usize) {
        let dev = match self.devices.get_mut(device) {
            Some(d) => d,
            None => {
                println!("[ERR] ไม่พบ device '{}'", device);
                return;
            }
        };

        // คืน instance
        let released = amount.min(dev.held_by.iter().filter(|&&p| p == pid).count());
        let mut removed = 0;
        dev.held_by.retain(|&p| {
            if p == pid && removed < released {
                removed += 1;
                false
            } else {
                true
            }
        });
        dev.available += released;

        // อัปเดต allocation
        if let Some(alloc) = self.allocation.get_mut(&pid) {
            let entry = alloc.entry(device.to_string()).or_insert(0);
            *entry = entry.saturating_sub(released);
        }

        println!(
            "[IO]  P{} คืน '{}' x{} → available={}",
            pid, device, released, dev.available
        );

        // ปลุก process ที่รออยู่ (ถ้ามี)
        self.wake_waiting(device);
    }

    fn wake_waiting(&mut self, device: &str) {
        let dev = match self.devices.get_mut(device) {
            Some(d) => d,
            None => return,
        };

        let mut woken = Vec::new();
        let mut remaining_queue = Vec::new();

        for &waiter in &dev.wait_queue.clone() {
            if dev.available > 0 {
                dev.available -= 1;
                dev.held_by.push(waiter);
                woken.push(waiter);
            } else {
                remaining_queue.push(waiter);
            }
        }
        dev.wait_queue = remaining_queue;

        for pid in woken {
            println!("[IO]  P{} ได้รับ '{}' (จาก wait queue)", pid, device);
            *self
                .allocation
                .entry(pid)
                .or_default()
                .entry(device.to_string())
                .or_insert(0) += 1;
            // ลบออกจาก request
            if let Some(req) = self.request.get_mut(&pid) {
                let e = req.entry(device.to_string()).or_insert(0);
                *e = e.saturating_sub(1);
            }
        }
    }

    // ── detect_deadlock ──────────────────────────────────────
    /// ตรวจ Deadlock ด้วย Resource Allocation Graph (Cycle Detection)
    pub fn detect_deadlock(&self) {
        println!("\n[DEADLOCK DETECTION] Resource Allocation Graph");
        println!("  {}", "─".repeat(50));

        // สร้าง wait-for graph: pid A รอ pid B
        // ถ้า A รอ device X และ B ถือ X อยู่
        let mut wait_for: HashMap<u32, Vec<u32>> = HashMap::new();

        for (&pid, reqs) in &self.request {
            for (device, &amount) in reqs {
                if amount == 0 {
                    continue;
                }
                let dev = match self.devices.get(device) {
                    Some(d) => d,
                    None => continue,
                };
                // หา pid ที่ถือ device นี้อยู่
                for &holder in &dev.held_by {
                    if holder != pid {
                        wait_for.entry(pid).or_default().push(holder);
                    }
                }
            }
        }

        // แสดง wait-for graph
        if wait_for.is_empty() {
            println!("  Wait-For Graph: (ว่าง — ไม่มีใครรอ)");
        } else {
            println!("  Wait-For Graph:");
            for (&pid, waitees) in &wait_for {
                let ws: Vec<String> = waitees.iter().map(|p| format!("P{}", p)).collect();
                println!("    P{} → {}", pid, ws.join(", "));
            }
        }

        // ตรวจ Cycle ด้วย DFS
        let deadlocked = find_cycle(&wait_for);

        println!();
        if deadlocked.is_empty() {
            println!("  ✅ ไม่พบ Deadlock");
        } else {
            let pids: Vec<String> = deadlocked.iter().map(|p| format!("P{}", p)).collect();
            println!("  ❌ พบ DEADLOCK! กลุ่ม process ที่ติดกัน: [{}]", pids.join(", "));
        }
    }

    // ── bankers_algorithm ────────────────────────────────────
    /// Banker's Algorithm — หา Safe Sequence
    pub fn bankers_algorithm(&self) {
        println!("\n[BANKER'S ALGORITHM] Safety Check");
        println!("  {}", "─".repeat(50));

        let device_names: Vec<String> = self.devices.keys().cloned().collect();

        // available ปัจจุบัน
        let mut work: HashMap<String, usize> = device_names
            .iter()
            .map(|d| (d.clone(), self.devices[d].available))
            .collect();

        // need[pid][device] = max - allocation
        let mut need: HashMap<u32, HashMap<String, usize>> = HashMap::new();
        for &pid in &self.processes {
            let mut n = HashMap::new();
            for d in &device_names {
                let max = self
                    .max_need
                    .get(&pid)
                    .and_then(|m| m.get(d))
                    .copied()
                    .unwrap_or(0);
                let alloc = self
                    .allocation
                    .get(&pid)
                    .and_then(|a| a.get(d))
                    .copied()
                    .unwrap_or(0);
                n.insert(d.clone(), max.saturating_sub(alloc));
            }
            need.insert(pid, n);
        }

        // แสดงตาราง Allocation / Need / Available
        println!("  {:<6} {:<20} {:<20}", "PID", "Allocation", "Need");
        println!("  {}", "─".repeat(50));
        for &pid in &self.processes {
            let alloc_str = format_resource_map(self.allocation.get(&pid), &device_names);
            let need_str = format_resource_map(need.get(&pid), &device_names);
            println!("  P{:<5} {:<20} {:<20}", pid, alloc_str, need_str);
        }
        let avail_str = device_names
            .iter()
            .map(|d| format!("{}:{}", d, self.devices[d].available))
            .collect::<Vec<_>>()
            .join(" ");
        println!("\n  Available: {}", avail_str);

        // Safety Algorithm
        let mut finish: HashMap<u32, bool> = self.processes.iter().map(|&p| (p, false)).collect();
        let mut safe_seq: Vec<u32> = Vec::new();

        loop {
            let mut found = false;
            for &pid in &self.processes {
                if finish[&pid] {
                    continue;
                }

                // ตรวจว่า need[pid] <= work ทุก device
                let can_run = device_names.iter().all(|d| {
                    let n = need[&pid].get(d).copied().unwrap_or(0);
                    let w = work.get(d).copied().unwrap_or(0);
                    n <= w
                });

                if can_run {
                    // P นี้ run ได้ → คืน resource กลับ
                    for d in &device_names {
                        let alloc = self
                            .allocation
                            .get(&pid)
                            .and_then(|a| a.get(d))
                            .copied()
                            .unwrap_or(0);
                        *work.entry(d.clone()).or_insert(0) += alloc;
                    }
                    *finish.get_mut(&pid).unwrap() = true;
                    safe_seq.push(pid);
                    found = true;
                }
            }
            if !found {
                break;
            }
        }

        println!();
        if finish.values().all(|&f| f) {
            let seq: Vec<String> = safe_seq.iter().map(|p| format!("P{}", p)).collect();
            println!("  ✅ Safe State! Safe Sequence: [{}]", seq.join(" → "));
        } else {
            let unsafe_pids: Vec<String> = finish
                .iter()
                .filter(|&(_, &f)| !f)
                .map(|(p, _)| format!("P{}", p))
                .collect();
            println!(
                "  ❌ Unsafe State! Process ที่อาจติด Deadlock: [{}]",
                unsafe_pids.join(", ")
            );
        }
    }

    // ── show_status ──────────────────────────────────────────
    pub fn show_status(&self) {
        println!("\n  ── Devices ─────────────────────────────────────");
        println!(
            "  {:<12} {:<10} {:<10} {:<15} {:<15}",
            "Device", "Total", "Available", "Held by", "Wait Queue"
        );
        println!("  {}", "─".repeat(65));

        for (name, dev) in &self.devices {
            let held: Vec<String> = {
                let mut seen = std::collections::HashSet::new();
                dev.held_by
                    .iter()
                    .filter(|&&p| seen.insert(p))
                    .map(|p| format!("P{}", p))
                    .collect()
            };
            let waiting: Vec<String> = dev.wait_queue.iter().map(|p| format!("P{}", p)).collect();
            println!(
                "  {:<12} {:<10} {:<10} {:<15} {:<15}",
                name,
                dev.instances,
                dev.available,
                if held.is_empty() {
                    "-".into()
                } else {
                    held.join(",")
                },
                if waiting.is_empty() {
                    "-".into()
                } else {
                    waiting.join(",")
                }
            );
        }
    }
}

// ─── Cycle Detection (DFS) ───────────────────────────────────

fn find_cycle(graph: &HashMap<u32, Vec<u32>>) -> Vec<u32> {
    let mut visited: std::collections::HashSet<u32> = std::collections::HashSet::new();
    let mut rec_stack: std::collections::HashSet<u32> = std::collections::HashSet::new();
    let mut in_cycle: Vec<u32> = Vec::new();

    for &node in graph.keys() {
        if !visited.contains(&node) {
            dfs(node, graph, &mut visited, &mut rec_stack, &mut in_cycle);
        }
    }
    in_cycle.sort();
    in_cycle.dedup();
    in_cycle
}

fn dfs(
    node: u32,
    graph: &HashMap<u32, Vec<u32>>,
    visited: &mut std::collections::HashSet<u32>,
    rec_stack: &mut std::collections::HashSet<u32>,
    in_cycle: &mut Vec<u32>,
) {
    visited.insert(node);
    rec_stack.insert(node);

    if let Some(neighbors) = graph.get(&node) {
        for &next in neighbors {
            if !visited.contains(&next) {
                dfs(next, graph, visited, rec_stack, in_cycle);
            } else if rec_stack.contains(&next) {
                // พบ cycle
                in_cycle.push(node);
                in_cycle.push(next);
            }
        }
    }
    rec_stack.remove(&node);
}

// ─── Helper ──────────────────────────────────────────────────

fn format_resource_map(map: Option<&HashMap<String, usize>>, keys: &[String]) -> String {
    match map {
        None => "-".to_string(),
        Some(m) => keys
            .iter()
            .map(|k| format!("{}:{}", k, m.get(k).copied().unwrap_or(0)))
            .collect::<Vec<_>>()
            .join(" "),
    }
}
