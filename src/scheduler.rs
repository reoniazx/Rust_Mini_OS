use crate::process::Pcb;

/// One slice on the Gantt Chart
#[derive(Debug, Clone)]
pub struct GanttSlice {
    pub pid: u32,
    pub start: u32,
    pub end: u32,
}

/// Result per process
#[derive(Debug)]
pub struct ProcResult {
    pub pid: u32,
    pub arrival_time: u32,
    pub burst_time: u32,
    pub finish_time: u32,
    pub turnaround_time: u32,
    pub waiting_time: u32,
}

pub struct ScheduleResult {
    pub gantt: Vec<GanttSlice>,
    pub results: Vec<ProcResult>,
    pub avg_wt: f64,
    pub avg_tat: f64,
}

// ─────────────────────────────────────────
//  FCFS
// ─────────────────────────────────────────
pub fn fcfs(processes: &[Pcb]) -> ScheduleResult {
    let mut procs: Vec<Pcb> = processes.to_vec();
    procs.sort_by_key(|p| p.arrival_time);

    let mut time = 0u32;
    let mut gantt: Vec<GanttSlice> = Vec::new();
    let mut results: Vec<ProcResult> = Vec::new();

    for p in &procs {
        if time < p.arrival_time {
            time = p.arrival_time; // CPU idle
        }
        let start = time;
        time += p.burst_time;
        gantt.push(GanttSlice {
            pid: p.pid,
            start,
            end: time,
        });

        let tat = time - p.arrival_time;
        let wt = tat - p.burst_time;
        results.push(ProcResult {
            pid: p.pid,
            arrival_time: p.arrival_time,
            burst_time: p.burst_time,
            finish_time: time,
            turnaround_time: tat,
            waiting_time: wt,
        });
    }

    compute_averages(gantt, results)
}

// ─────────────────────────────────────────
//  SJF (Non-preemptive)
// ─────────────────────────────────────────
pub fn sjf(processes: &[Pcb]) -> ScheduleResult {
    let mut remaining: Vec<Pcb> = processes.to_vec();
    let mut time = 0u32;
    let mut gantt: Vec<GanttSlice> = Vec::new();
    let mut results: Vec<ProcResult> = Vec::new();

    while !remaining.is_empty() {
        // Find arrived process with shortest burst_time
        let ready: Vec<usize> = remaining
            .iter()
            .enumerate()
            .filter(|(_, p)| p.arrival_time <= time)
            .map(|(i, _)| i)
            .collect();

        if ready.is_empty() {
            // CPU idle — jump to nearest arrival time
            let next = remaining.iter().map(|p| p.arrival_time).min().unwrap();
            time = next;
            continue;
        }

        let idx = ready
            .into_iter()
            .min_by_key(|&i| remaining[i].burst_time)
            .unwrap();

        let p = remaining.remove(idx);
        let start = time;
        time += p.burst_time;
        gantt.push(GanttSlice {
            pid: p.pid,
            start,
            end: time,
        });

        let tat = time - p.arrival_time;
        let wt = tat - p.burst_time;
        results.push(ProcResult {
            pid: p.pid,
            arrival_time: p.arrival_time,
            burst_time: p.burst_time,
            finish_time: time,
            turnaround_time: tat,
            waiting_time: wt,
        });
    }

    compute_averages(gantt, results)
}

// ─────────────────────────────────────────
//  Round Robin
// ─────────────────────────────────────────
pub fn round_robin(processes: &[Pcb], quantum: u32) -> ScheduleResult {
    let mut procs: Vec<Pcb> = processes.to_vec();
    procs.sort_by_key(|p| p.arrival_time);

    let mut queue: std::collections::VecDeque<Pcb> = std::collections::VecDeque::new();
    let mut time = 0u32;
    let mut gantt: Vec<GanttSlice> = Vec::new();
    let mut results: Vec<ProcResult> = Vec::new();
    let mut arrived_idx = 0usize;

    // Add initial processes that arrive at time 0 to queue
    while arrived_idx < procs.len() && procs[arrived_idx].arrival_time <= time {
        queue.push_back(procs[arrived_idx].clone());
        arrived_idx += 1;
    }

    while !queue.is_empty() {
        let mut p = queue.pop_front().unwrap();

        let run = p.remaining_time.min(quantum);
        let start = time;
        time += run;
        p.remaining_time -= run;

        gantt.push(GanttSlice {
            pid: p.pid,
            start,
            end: time,
        });

        // Add processes that arrive during this slice to queue
        while arrived_idx < procs.len() && procs[arrived_idx].arrival_time <= time {
            queue.push_back(procs[arrived_idx].clone());
            arrived_idx += 1;
        }

        if p.remaining_time == 0 {
            let tat = time - p.arrival_time;
            let wt = tat - p.burst_time;
            results.push(ProcResult {
                pid: p.pid,
                arrival_time: p.arrival_time,
                burst_time: p.burst_time,
                finish_time: time,
                turnaround_time: tat,
                waiting_time: wt,
            });
        } else {
            queue.push_back(p); // Not finished, put back in queue
        }

        // If queue is empty but there are still processes that haven't arrived
        if queue.is_empty() && arrived_idx < procs.len() {
            time = procs[arrived_idx].arrival_time;
            while arrived_idx < procs.len() && procs[arrived_idx].arrival_time <= time {
                queue.push_back(procs[arrived_idx].clone());
                arrived_idx += 1;
            }
        }
    }

    // Sort results by pid
    results.sort_by_key(|r| r.pid);
    compute_averages(gantt, results)
}

// ─────────────────────────────────────────
//  Priority (Non-preemptive, lower value = higher priority)
// ─────────────────────────────────────────
pub fn priority_scheduling(processes: &[Pcb]) -> ScheduleResult {
    let mut remaining: Vec<Pcb> = processes.to_vec();
    let mut time = 0u32;
    let mut gantt: Vec<GanttSlice> = Vec::new();
    let mut results: Vec<ProcResult> = Vec::new();

    while !remaining.is_empty() {
        let ready: Vec<usize> = remaining
            .iter()
            .enumerate()
            .filter(|(_, p)| p.arrival_time <= time)
            .map(|(i, _)| i)
            .collect();

        if ready.is_empty() {
            let next = remaining.iter().map(|p| p.arrival_time).min().unwrap();
            time = next;
            continue;
        }

        let idx = ready
            .into_iter()
            .min_by_key(|&i| remaining[i].priority)
            .unwrap();

        let p = remaining.remove(idx);
        let start = time;
        time += p.burst_time;
        gantt.push(GanttSlice {
            pid: p.pid,
            start,
            end: time,
        });

        let tat = time - p.arrival_time;
        let wt = tat - p.burst_time;
        results.push(ProcResult {
            pid: p.pid,
            arrival_time: p.arrival_time,
            burst_time: p.burst_time,
            finish_time: time,
            turnaround_time: tat,
            waiting_time: wt,
        });
    }

    compute_averages(gantt, results)
}

// ─────────────────────────────────────────
//  Helper
// ─────────────────────────────────────────
fn compute_averages(gantt: Vec<GanttSlice>, results: Vec<ProcResult>) -> ScheduleResult {
    let n = results.len() as f64;
    let avg_wt = results.iter().map(|r| r.waiting_time as f64).sum::<f64>() / n;
    let avg_tat = results
        .iter()
        .map(|r| r.turnaround_time as f64)
        .sum::<f64>()
        / n;
    ScheduleResult {
        gantt,
        results,
        avg_wt,
        avg_tat,
    }
}
