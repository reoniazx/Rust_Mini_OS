mod deadlock;
mod disk;
mod memory;
mod page_replace;
mod process;
mod scheduler;
mod utils;

use deadlock::DeadlockManager;
use disk::DiskManager;
use memory::MemoryManager;
use page_replace::{fifo, lru, optimal, print_page_result};
use process::ProcessManager;
use scheduler::{fcfs, priority_scheduling, round_robin, sjf};
use std::io::{self, Write};
use utils::{print_gantt, print_results};

fn main() {
    let mut mgr = ProcessManager::new();
    let mut mem = MemoryManager::new();
    let mut dsk = DiskManager::new();
    let mut dlk = DeadlockManager::new();
    let mut last_result: Option<scheduler::ScheduleResult> = None;

    println!("╔══════════════════════════════════════╗");
    println!("║       OS Simulator — Rust Edition    ║");
    println!("╚══════════════════════════════════════╝");
    println!("  พิมพ์ 'help' เพื่อดูคำสั่งทั้งหมด\n");

    loop {
        print!("os-sim> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "help" => print_help(),

            // ── Process ────────────────────────────────────────────
            "add_process" | "add" => {
                // add_process <pid> <arrival> <burst> <priority>
                // (pid ถูก ignore — กำหนดอัตโนมัติ)
                if parts.len() < 4 {
                    eprintln!("  usage: add_process <pid> <arrival> <burst> <priority>");
                    continue;
                }
                let at = parse_u32(parts[2]);
                let bt = parse_u32(parts[3]);
                let pri = if parts.len() >= 5 {
                    parse_u32(parts[4])
                } else {
                    1
                };
                let pid = mgr.add(at, bt, pri);
                println!(
                    "[OK] Added process: PID={} AT={} BT={} PR={}",
                    pid, at, bt, pri
                );
                let new_pids: Vec<String> =
                    mgr.processes.iter().map(|p| p.pid.to_string()).collect();
                println!("ReadyQueue: (empty) | New: [{}]", new_pids.join(", "));
            }

            "list_process" | "ps" => {
                mgr.list();
            }

            "clear" => {
                mgr.clear();
                println!("[OK] ล้างรายการ process ทั้งหมด");
            }

            // ── Scheduler ──────────────────────────────────────────
            "run_scheduler" => {
                // run_scheduler <ALG> [quantum]
                if parts.len() < 2 {
                    eprintln!("  usage: run_scheduler <FCFS|SJF|RR|PRIORITY> [quantum]");
                    continue;
                }
                if mgr.processes.is_empty() {
                    println!("  ยังไม่มี process");
                    continue;
                }

                let res = match parts[1].to_uppercase().as_str() {
                    "FCFS" => {
                        println!("[RUN] Scheduler=FCFS");
                        fcfs(&mgr.processes)
                    }
                    "SJF" => {
                        println!("[RUN] Scheduler=SJF (non-preemptive)");
                        sjf(&mgr.processes)
                    }
                    "RR" => {
                        let q = if parts.len() >= 3 {
                            parse_u32(parts[2])
                        } else {
                            4
                        };
                        println!("[RUN] Scheduler=RR q={}", q);
                        round_robin(&mgr.processes, q)
                    }
                    "PRIORITY" => {
                        println!("[RUN] Scheduler=PRIORITY (non-preemptive)");
                        priority_scheduling(&mgr.processes)
                    }
                    _ => {
                        println!("  ไม่รู้จัก algorithm '{}'", parts[1]);
                        continue;
                    }
                };

                print_gantt(&res.gantt);
                print_results(&res);
                last_result = Some(res);
            }

            "show_gantt" => match &last_result {
                Some(res) => print_gantt(&res.gantt),
                None => println!("  ยังไม่ได้รัน scheduler"),
            },

            "show_table" => match &last_result {
                Some(res) => print_results(res),
                None => println!("  ยังไม่ได้รัน scheduler"),
            },

            // ── Page Replacement ───────────────────────────────────
            "simulate_page" => {
                // simulate_page <FIFO|LRU|OPTIMAL> <frames> <ref...>
                if parts.len() < 4 {
                    eprintln!("  usage: simulate_page <FIFO|LRU|OPTIMAL> <frames> <ref...>");
                    continue;
                }
                let alg = parts[1].to_uppercase();
                let frames = parse_usize(parts[2]);
                let refs: Vec<u32> = parts[3..].iter().map(|s| parse_u32(s)).collect();

                let summary = match alg.as_str() {
                    "FIFO" => fifo(frames, &refs),
                    "LRU" => lru(frames, &refs),
                    "OPTIMAL" | "OPT" => optimal(frames, &refs),
                    _ => {
                        println!("  ไม่รู้จัก algorithm '{}'", alg);
                        continue;
                    }
                };
                print_page_result(&alg, frames, &summary);
            }

            // ── Memory (Paging) ────────────────────────────────────
            "simulate_memory" => {
                // simulate_memory paging
                mem.init();
            }

            "alloc" => {
                // alloc <pid> <kb>
                if parts.len() < 3 {
                    eprintln!("  usage: alloc <pid> <kb>");
                    continue;
                }
                if !mem.enabled {
                    println!("  รัน 'simulate_memory paging' ก่อน");
                    continue;
                }
                let pid = parse_u32(parts[1]);
                let kb = parse_u32(parts[2]);
                if let Err(e) = mem.alloc(pid, kb) {
                    println!("  [ERR] {}", e);
                }
            }

            "translate" => {
                // translate <pid> <logical_addr>
                if parts.len() < 3 {
                    eprintln!("  usage: translate <pid> <logical_addr>");
                    continue;
                }
                if !mem.enabled {
                    println!("  รัน 'simulate_memory paging' ก่อน");
                    continue;
                }
                let pid = parse_u32(parts[1]);
                let addr = parse_u32(parts[2]);
                if let Err(e) = mem.translate(pid, addr) {
                    println!("  [ERR] {}", e);
                }
            }

            "free" => {
                if parts.len() < 2 {
                    eprintln!("  usage: free <pid>");
                    continue;
                }
                if !mem.enabled {
                    println!("  รัน 'simulate_memory paging' ก่อน");
                    continue;
                }
                let pid = parse_u32(parts[1]);
                if let Err(e) = mem.free(pid) {
                    println!("  [ERR] {}", e);
                }
            }

            "frames" => {
                mem.show_frames();
            }

            // ── Disk / File Allocation ─────────────────────────────
            "simulate_disk" => {
                // simulate_disk <contiguous|linked|indexed>
                let mode = if parts.len() >= 2 {
                    parts[1]
                } else {
                    "contiguous"
                };
                dsk.init(mode);
            }

            "create" => {
                // create <filename> <blocks>
                if parts.len() < 3 {
                    eprintln!("  usage: create <file> <blocks>");
                    continue;
                }
                if dsk.alloc_mode.is_none() {
                    println!("  รัน 'simulate_disk <mode>' ก่อน");
                    continue;
                }
                let size = parse_usize(parts[2]);
                if let Err(e) = dsk.create(parts[1], size) {
                    println!("  [ERR] {}", e);
                }
            }

            "delete" => {
                if parts.len() < 2 {
                    eprintln!("  usage: delete <file>");
                    continue;
                }
                if let Err(e) = dsk.delete(parts[1]) {
                    println!("  [ERR] {}", e);
                }
            }

            "ls" => {
                dsk.ls();
            }

            "map" => {
                if parts.len() < 2 {
                    eprintln!("  usage: map <file>");
                    continue;
                }
                if let Err(e) = dsk.map(parts[1]) {
                    println!("  [ERR] {}", e);
                }
            }

            // ── I/O & Deadlock ────────────────────────────────────
            "add_device" => {
                // add_device <name> <instances>
                if parts.len() < 3 {
                    eprintln!("  usage: add_device <name> <instances>");
                    continue;
                }
                let n = parse_usize(parts[2]);
                dlk.add_device(parts[1], n);
            }

            "add_proc_dl" => {
                // add_proc_dl <pid> <max_need>  เช่น  add_proc_dl 1 printer:2,disk:1
                if parts.len() < 3 {
                    eprintln!("  usage: add_proc_dl <pid> <device:n,...>");
                    continue;
                }
                let pid = parse_u32(parts[1]);
                dlk.add_process(pid, parts[2]);
            }

            "io_request" => {
                // io_request <pid> <device> <amount>
                if parts.len() < 4 {
                    eprintln!("  usage: io_request <pid> <device> <amount>");
                    continue;
                }
                let pid = parse_u32(parts[1]);
                let amount = parse_usize(parts[3]);
                dlk.request_resource(pid, parts[2], amount);
            }

            "io_release" => {
                // io_release <pid> <device> <amount>
                if parts.len() < 4 {
                    eprintln!("  usage: io_release <pid> <device> <amount>");
                    continue;
                }
                let pid = parse_u32(parts[1]);
                let amount = parse_usize(parts[3]);
                dlk.release_resource(pid, parts[2], amount);
            }

            "detect_deadlock" => {
                dlk.detect_deadlock();
            }
            "bankers" => {
                dlk.bankers_algorithm();
            }
            "io_status" => {
                dlk.show_status();
            }

            "exit" | "quit" | "q" => {
                println!("  ลาก่อน!");
                break;
            }

            _ => println!("  ไม่รู้จักคำสั่ง '{}'  พิมพ์ 'help' เพื่อดูคำสั่ง", parts[0]),
        }
        println!();
    }
}

fn parse_u32(s: &str) -> u32 {
    s.parse::<u32>().unwrap_or_else(|_| {
        eprintln!("  ⚠ '{}' ไม่ถูกต้อง ใช้ 0", s);
        0
    })
}
fn parse_usize(s: &str) -> usize {
    s.parse::<usize>().unwrap_or_else(|_| {
        eprintln!("  ⚠ '{}' ไม่ถูกต้อง ใช้ 0", s);
        0
    })
}

fn print_help() {
    println!(
        "
  ─── Process Management ──────────────────────────────────────
  add_process <pid> <arrival> <burst> <priority>
  list_process
  clear

  ─── CPU Scheduling ──────────────────────────────────────────
  run_scheduler FCFS
  run_scheduler SJF
  run_scheduler RR [quantum]
  run_scheduler PRIORITY
  show_gantt
  show_table

  ─── Page Replacement ────────────────────────────────────────
  simulate_page <FIFO|LRU|OPTIMAL> <frames> <ref ref ...>

  ─── Memory (Paging) ─────────────────────────────────────────
  simulate_memory paging
  alloc <pid> <kb>
  translate <pid> <logical_addr>
  free <pid>
  frames

  ─── File Allocation ─────────────────────────────────────────
  simulate_disk <contiguous|linked|indexed>
  create <file> <blocks>
  delete <file>
  ls
  map <file>

  ─── อื่นๆ ───────────────────────────────────────────────────
  help
  exit

  ─── I/O & Deadlock (เสริม) ───────────────────────────────────
  add_device <device> <instances>       เพิ่ม I/O device
  add_proc_dl <pid> <device:n,...>      ลงทะเบียน process + max need
  io_request <pid> <device> <amount>    ขอ resource
  io_release <pid> <device> <amount>    คืน resource
  io_status                             แสดงสถานะ device ทั้งหมด
  detect_deadlock                       ตรวจ deadlock (RAG cycle)
  bankers                               Banker Algorithm (safe sequence)
"
    );
}
