//! Check procfs unit conversion

use procfs::process::Process;
use procfs::{Current, Meminfo};

fn main() {
    println!("=== Checking procfs units ===\n");
    
    // Check system memory
    let mem = Meminfo::current().unwrap();
    println!("Meminfo::mem_total = {} (if bytes: {} GB, if kB: {} GB)", 
        mem.mem_total,
        mem.mem_total / 1024 / 1024 / 1024,
        mem.mem_total / 1024 / 1024);
    
    // Check current process
    let me = Process::myself().unwrap();
    let status = me.status().unwrap();
    
    println!("\nCurrent process (PID {}):", me.pid());
    println!("  status.vmrss = {:?}", status.vmrss);
    println!("  status.vmsize = {:?}", status.vmsize);
    
    // Read raw /proc/self/status to compare
    let raw = std::fs::read_to_string("/proc/self/status").unwrap();
    for line in raw.lines() {
        if line.starts_with("VmRSS:") || line.starts_with("VmSize:") {
            println!("  raw: {}", line.trim());
        }
    }
    
    // Now check a browser process if it exists
    println!("\n=== Looking for large processes ===");
    for proc in procfs::process::all_processes().unwrap().flatten() {
        if let Ok(status) = proc.status() {
            if let Some(rss) = status.vmrss {
                // Check if rss > 100MB when treated as kB
                if rss > 100_000 {  // 100MB in kB
                    let name = proc.stat().map(|s| s.comm).unwrap_or_default();
                    println!("PID {:6} {:20}: vmrss={} (as kB: {}MB, as bytes: {}MB)",
                        proc.pid(), name, rss, rss/1024, rss/1024/1024);
                }
            }
        }
    }
}
