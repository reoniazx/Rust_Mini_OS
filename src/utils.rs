use crate::scheduler::{GanttSlice, ScheduleResult};

/// วาด Gantt Chart แบบ text
pub fn print_gantt(gantt: &[GanttSlice]) {
    println!("\n  Gantt Chart:");

    // แถวบน: ชื่อ process
    print!("  |");
    for s in gantt {
        let width = (s.end - s.start) as usize * 2;
        let label = format!("P{}", s.pid);
        print!("{:^width$}|", label, width = width.max(label.len() + 2));
    }
    println!();

    // แถวล่าง: เวลา
    print!("  ");
    let mut prev_end = u32::MAX;
    for s in gantt {
        if s.start != prev_end {
            print!("{:<4}", s.start);
        } else {
            print!("{:<4}", "");
        }
        let width = (s.end - s.start) as usize * 2;
        print!("{:>width$}", s.end, width = width.max(3));
        prev_end = s.end;
    }
    println!();
}

/// แสดงตาราง metrics ต่อ process
pub fn print_results(res: &ScheduleResult) {
    println!(
        "\n  {:<6} {:<10} {:<8} {:<8} {:<12} {:<10}",
        "PID", "Arrival", "Burst", "Finish", "Turnaround", "Waiting"
    );
    println!("  {}", "─".repeat(58));

    for r in &res.results {
        println!(
            "  {:<6} {:<10} {:<8} {:<8} {:<12} {:<10}",
            r.pid, r.arrival_time, r.burst_time, r.finish_time, r.turnaround_time, r.waiting_time
        );
    }

    println!("  {}", "─".repeat(58));
    println!("  Avg Waiting Time     : {:.2}", res.avg_wt);
    println!("  Avg Turnaround Time  : {:.2}", res.avg_tat);
}
