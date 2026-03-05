use std::collections::{HashMap, VecDeque};

#[derive(Debug)]
pub struct PageResult {
    pub reference: u32,
    pub frames: Vec<Option<u32>>,
    pub fault: bool,
    pub evicted: Option<u32>,
}

pub struct PageSummary {
    pub results: Vec<PageResult>,
    pub faults: usize,
    pub hits: usize,
    pub hit_ratio: f64,
}

// ─────────────────────────────────────────
//  FIFO
// ─────────────────────────────────────────
pub fn fifo(frames: usize, refs: &[u32]) -> PageSummary {
    let mut queue: VecDeque<u32> = VecDeque::new(); // order of entry
    let mut frame_set: Vec<Option<u32>> = vec![None; frames];
    let mut results = Vec::new();
    let mut faults = 0;
    let mut hits = 0;

    for &page in refs {
        let in_memory = queue.contains(&page);

        let evicted;
        if in_memory {
            hits += 1;
            evicted = None;
        } else {
            faults += 1;
            if queue.len() < frames {
                // Still has free space
                let slot = frame_set.iter().position(|f| f.is_none()).unwrap();
                frame_set[slot] = Some(page);
                queue.push_back(page);
                evicted = None;
            } else {
                // Evict the first page that entered
                let victim = queue.pop_front().unwrap();
                let slot = frame_set.iter().position(|f| *f == Some(victim)).unwrap();
                frame_set[slot] = Some(page);
                queue.push_back(page);
                evicted = Some(victim);
            }
        }

        results.push(PageResult {
            reference: page,
            frames: frame_set.clone(),
            fault: !in_memory,
            evicted,
        });
    }

    summarize(results, faults, hits)
}

// ─────────────────────────────────────────
//  LRU
// ─────────────────────────────────────────
pub fn lru(frames: usize, refs: &[u32]) -> PageSummary {
    let mut frame_set: Vec<Option<u32>> = vec![None; frames];
    let mut last_used: HashMap<u32, usize> = HashMap::new(); // page → last time index
    let mut results = Vec::new();
    let mut faults = 0;
    let mut hits = 0;

    for (time, &page) in refs.iter().enumerate() {
        let in_memory = frame_set.contains(&Some(page));

        let evicted;
        if in_memory {
            hits += 1;
            evicted = None;
        } else {
            faults += 1;
            if let Some(slot) = frame_set.iter().position(|f| f.is_none()) {
                // Still has free space
                frame_set[slot] = Some(page);
                evicted = None;
            } else {
                // Find page used longest ago (smallest last_used)
                let victim = frame_set
                    .iter()
                    .filter_map(|f| *f)
                    .min_by_key(|p| last_used.get(p).copied().unwrap_or(0))
                    .unwrap();
                let slot = frame_set.iter().position(|f| *f == Some(victim)).unwrap();
                frame_set[slot] = Some(page);
                evicted = Some(victim);
            }
        }

        last_used.insert(page, time);

        results.push(PageResult {
            reference: page,
            frames: frame_set.clone(),
            fault: !in_memory,
            evicted,
        });
    }

    summarize(results, faults, hits)
}

// ─────────────────────────────────────────
//  Optimal
// ─────────────────────────────────────────
pub fn optimal(frames: usize, refs: &[u32]) -> PageSummary {
    let mut frame_set: Vec<Option<u32>> = vec![None; frames];
    let mut results = Vec::new();
    let mut faults = 0;
    let mut hits = 0;

    for (i, &page) in refs.iter().enumerate() {
        let in_memory = frame_set.contains(&Some(page));

        let evicted;
        if in_memory {
            hits += 1;
            evicted = None;
        } else {
            faults += 1;
            if let Some(slot) = frame_set.iter().position(|f| f.is_none()) {
                frame_set[slot] = Some(page);
                evicted = None;
            } else {
                // Find page used furthest in future (or never used again)
                let future = &refs[i + 1..];
                let victim = frame_set
                    .iter()
                    .filter_map(|f| *f)
                    .max_by_key(|p| future.iter().position(|r| r == p).unwrap_or(usize::MAX))
                    .unwrap();
                let slot = frame_set.iter().position(|f| *f == Some(victim)).unwrap();
                frame_set[slot] = Some(page);
                evicted = Some(victim);
            }
        }

        results.push(PageResult {
            reference: page,
            frames: frame_set.clone(),
            fault: !in_memory,
            evicted,
        });
    }

    summarize(results, faults, hits)
}

// ─────────────────────────────────────────
//  Display
// ─────────────────────────────────────────
pub fn print_page_result(alg: &str, frames: usize, summary: &PageSummary) {
    println!("[RUN] PageReplacement={} Frames={}", alg, frames);
    println!("  {:<5} | {:<30} | {}", "Ref", "Frames", "Result");
    println!("  {}", "─".repeat(55));

    for r in &summary.results {
        let frames_str: Vec<String> = r
            .frames
            .iter()
            .map(|f| match f {
                Some(p) => p.to_string(),
                None => "-".to_string(),
            })
            .collect();
        let frame_display = format!("[{}]", frames_str.join(", "));

        let result_str = if r.fault {
            match r.evicted {
                Some(e) => format!("FAULT (evict {})", e),
                None => "FAULT".to_string(),
            }
        } else {
            "HIT".to_string()
        };

        println!(
            "  {:<5} | {:<30} | {}",
            r.reference, frame_display, result_str
        );
    }

    println!("\nSummary:");
    println!("  Page Faults = {}", summary.faults);
    println!("  Hits        = {}", summary.hits);
    println!("  Hit Ratio   = {:.2}%", summary.hit_ratio);
}

fn summarize(results: Vec<PageResult>, faults: usize, hits: usize) -> PageSummary {
    let total = faults + hits;
    let hit_ratio = if total > 0 {
        hits as f64 / total as f64 * 100.0
    } else {
        0.0
    };
    PageSummary {
        results,
        faults,
        hits,
        hit_ratio,
    }
}
