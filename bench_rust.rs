use std::path::Path;
use std::time::Instant;

// Include waldb directly
include!("waldb.rs");

fn main() -> std::io::Result<()> {
    let store = Store::open(Path::new("/tmp/bench-rust-waldb"))?;
    
    println!("Running Rust native benchmark...\n");
    
    // Write benchmark
    let start = Instant::now();
    for i in 0..10000 {
        store.set(&format!("key/{}", i), &format!("value-{}", i), false)?;
    }
    let write_time = start.elapsed();
    let write_ops = 10000.0 / write_time.as_secs_f64();
    println!("Write 10,000 keys: {:?} ({:.0} ops/sec)", write_time, write_ops);
    
    // Flush to ensure data is on disk
    store.flush()?;
    
    // Read benchmark
    let start = Instant::now();
    for i in 0..10000 {
        let _ = store.get(&format!("key/{}", i))?;
    }
    let read_time = start.elapsed();
    let read_ops = 10000.0 / read_time.as_secs_f64();
    println!("Read 10,000 keys: {:?} ({:.0} ops/sec)", read_time, read_ops);
    
    // Pattern matching benchmark
    let start = Instant::now();
    let results = store.get_pattern("key/*")?;
    let pattern_time = start.elapsed();
    println!("Pattern match {} keys: {:?}", results.len(), pattern_time);
    
    // Range query benchmark
    let start = Instant::now();
    let results = store.get_range("key/0", "key/9999")?;
    let range_time = start.elapsed();
    println!("Range query {} keys: {:?}", results.len(), range_time);
    
    println!("\n✅ Raw Rust performance (no FFI overhead)");
    
    Ok(())
}