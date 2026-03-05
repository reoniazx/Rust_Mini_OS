#[derive(Debug, Clone, PartialEq)]
pub enum ProcessState {
    New,
}

impl std::fmt::Display for ProcessState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessState::New => write!(f, "New"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pcb {
    pub pid: u32,
    pub arrival_time: u32,
    pub burst_time: u32,
    pub remaining_time: u32,
    pub priority: u32, // lower value = higher priority
    pub state: ProcessState,
}

impl Pcb {
    pub fn new(pid: u32, arrival_time: u32, burst_time: u32, priority: u32) -> Self {
        Pcb {
            pid,
            arrival_time,
            burst_time,
            remaining_time: burst_time,
            priority,
            state: ProcessState::New,
        }
    }
}

pub struct ProcessManager {
    pub processes: Vec<Pcb>,
    next_pid: u32,
}

impl ProcessManager {
    pub fn new() -> Self {
        ProcessManager {
            processes: Vec::new(),
            next_pid: 1,
        }
    }

    pub fn add(&mut self, arrival_time: u32, burst_time: u32, priority: u32) -> u32 {
        let pid = self.next_pid;
        self.next_pid += 1;
        let pcb = Pcb::new(pid, arrival_time, burst_time, priority);
        self.processes.push(pcb);
        pid
    }

    /// List all processes
    pub fn list(&self) {
        if self.processes.is_empty() {
            println!("  (no processes yet)");
            return;
        }
        println!(
            "  {:<6} {:<10} {:<12} {:<10} {:<12}",
            "PID", "Arrival", "Burst", "Priority", "State"
        );
        println!("  {}", "-".repeat(54));
        for p in &self.processes {
            println!(
                "  {:<6} {:<10} {:<12} {:<10} {:<12}",
                p.pid, p.arrival_time, p.burst_time, p.priority, p.state
            );
        }
    }

    pub fn clear(&mut self) {
        self.processes.clear();
        self.next_pid = 1;
    }
}
