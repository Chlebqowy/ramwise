//! Debug tool to check procfs values

use procfs::process::all_processes;
use procfs::{Current, Meminfo};

fn main() {
    println!("=== System Memory ===");
    let mem = Meminfo::current().unwrap();
    println!("Total: {} bytes = {} MB", mem.mem_total, mem.mem_total / 1024 / 1024);
    println!("Available: {} bytes = {} MB", mem.mem_available.unwrap_or(0), mem.mem_available.unwrap_or(0) / 1024 / 1024);
    
    println!("\n=== Processes with RSS > 10MB ===");
    let mut count = 0;
    let mut big_procs = Vec::new();
    
    for proc in all_processes().unwrap() {
        count += 1;
        if let Ok(p) = proc {
            if let Ok(status) = p.status() {
                if let Some(rss_kb) = status.vmrss {
                    // procfs returns kB, convert to bytes
                    let rss_bytes = rss_kb * 1024;
                    if rss_bytes > 10_000_000 { // > 10MB
                        let name = p.stat().map(|s| s.comm).unwrap_or_else(|_| "?".to_string());
                        big_procs.push((p.pid(), name, rss_bytes));
                    }
                }
            }
        }
    }
    
    // Sort by RSS descending
    big_procs.sort_by(|a, b| b.2.cmp(&a.2));
    
    for (pid, name, rss) in big_procs.iter().take(30) {
        println!("  PID {:6}: {:20} - RSS: {:>12} bytes = {:>6} MB", 
            pid, name, rss, rss / 1024 / 1024);
    }
    
    println!("\nTotal processes scanned: {}", count);
    println!("Processes with RSS > 10MB: {}", big_procs.len());
}
