//! Check smaps_rollup units

use procfs::process::Process;

fn main() {
    let me = Process::myself().unwrap();
    
    if let Ok(smaps) = me.smaps_rollup() {
        if let Some(rollup) = smaps.memory_map_rollup.0.first() {
            let ext = &rollup.extension.map;
            println!("smaps_rollup values:");
            for (key, value) in ext.iter() {
                println!("  {}: {}", key, value);
            }
        }
    }
    
    // Compare with raw
    println!("\nRaw /proc/self/smaps_rollup:");
    if let Ok(raw) = std::fs::read_to_string("/proc/self/smaps_rollup") {
        for line in raw.lines().take(15) {
            println!("  {}", line);
        }
    }
}
