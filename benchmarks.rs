// WalDB Benchmark Suite
// Performance measurements and regression tests

mod waldb_store {
    include!("waldb.rs");
}

use waldb_store::*;
use std::time::{Duration, Instant};
use std::sync::Arc;
use std::thread;

// Benchmark result structure
#[derive(Debug)]
struct BenchmarkResult {
    name: String,
    operations: usize,
    duration: Duration,
    ops_per_sec: f64,
    notes: String,
}

impl BenchmarkResult {
    fn new(name: &str, operations: usize, duration: Duration) -> Self {
        BenchmarkResult {
            name: name.to_string(),
            operations,
            duration,
            ops_per_sec: operations as f64 / duration.as_secs_f64(),
            notes: String::new(),
        }
    }
    
    fn with_note(mut self, note: &str) -> Self {
        self.notes = note.to_string();
        self
    }
}

// Helper to create temp directories
fn bench_dir(name: &str) -> String {
    let dir = format!("/tmp/antler_bench_{}_{}", name, std::process::id());
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn cleanup(dir: &str) {
    let _ = std::fs::remove_dir_all(dir);
}

// ==================== WRITE BENCHMARKS ====================

fn bench_sequential_writes() -> BenchmarkResult {
    let dir = bench_dir("seq_writes");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    let operations = 10000;
    let start = Instant::now();
    
    for i in 0..operations {
        store.set(&format!("key{:08}", i), "value", false).unwrap();
    }
    store.flush().unwrap();
    
    let duration = start.elapsed();
    cleanup(&dir);
    
    BenchmarkResult::new("Sequential Writes", operations, duration)
        .with_note("Keys in order (best case)")
}

fn bench_random_writes() -> BenchmarkResult {
    let dir = bench_dir("rand_writes");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    let operations = 10000;
    let start = Instant::now();
    
    // Use simple hash to randomize
    for i in 0..operations {
        let key = format!("key{:08}", (i * 7919) % operations);
        store.set(&key, "value", false).unwrap();
    }
    store.flush().unwrap();
    
    let duration = start.elapsed();
    cleanup(&dir);
    
    BenchmarkResult::new("Random Writes", operations, duration)
        .with_note("Keys randomized (worst case)")
}

fn bench_batch_writes() -> BenchmarkResult {
    let dir = bench_dir("batch_writes");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    let operations = 10000;
    let start = Instant::now();
    
    // Write in batches of 100
    for batch in 0..100 {
        for i in 0..100 {
            let key = format!("batch{}/key{}", batch, i);
            store.set(&key, "value", false).unwrap();
        }
    }
    store.flush().unwrap();
    
    let duration = start.elapsed();
    cleanup(&dir);
    
    BenchmarkResult::new("Batch Writes", operations, duration)
        .with_note("100 batches of 100 keys")
}

fn bench_large_values() -> BenchmarkResult {
    let dir = bench_dir("large_values");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    let operations = 1000;
    let large_value = "x".repeat(10000); // 10KB values
    let start = Instant::now();
    
    for i in 0..operations {
        store.set(&format!("key{}", i), &large_value, false).unwrap();
    }
    store.flush().unwrap();
    
    let duration = start.elapsed();
    cleanup(&dir);
    
    BenchmarkResult::new("Large Value Writes", operations, duration)
        .with_note("10KB values")
}

// ==================== READ BENCHMARKS ====================

fn bench_sequential_reads() -> BenchmarkResult {
    let dir = bench_dir("seq_reads");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    // Prepare data
    let operations = 10000;
    for i in 0..operations {
        store.set(&format!("key{:08}", i), "value", false).unwrap();
    }
    store.flush().unwrap();
    
    let start = Instant::now();
    for i in 0..operations {
        store.get(&format!("key{:08}", i)).unwrap();
    }
    let duration = start.elapsed();
    
    cleanup(&dir);
    
    BenchmarkResult::new("Sequential Reads", operations, duration)
        .with_note("Keys in order (cache friendly)")
}

fn bench_random_reads() -> BenchmarkResult {
    let dir = bench_dir("rand_reads");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    // Prepare data
    let operations = 10000;
    for i in 0..operations {
        store.set(&format!("key{:08}", i), "value", false).unwrap();
    }
    store.flush().unwrap();
    
    let start = Instant::now();
    for i in 0..operations {
        let key = format!("key{:08}", (i * 7919) % operations);
        store.get(&key).unwrap();
    }
    let duration = start.elapsed();
    
    cleanup(&dir);
    
    BenchmarkResult::new("Random Reads", operations, duration)
        .with_note("Randomized access pattern")
}

fn bench_cache_hit_rate() -> BenchmarkResult {
    let dir = bench_dir("cache_hits");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    // Prepare small dataset that fits in cache
    for i in 0..100 {
        store.set(&format!("cached{}", i), "value", false).unwrap();
    }
    store.flush().unwrap();
    
    let operations = 100000; // Read same 100 keys 1000 times
    let start = Instant::now();
    
    for i in 0..operations {
        store.get(&format!("cached{}", i % 100)).unwrap();
    }
    let duration = start.elapsed();
    
    cleanup(&dir);
    
    BenchmarkResult::new("Cache Hit Rate", operations, duration)
        .with_note("100 keys read 1000 times each")
}

fn bench_miss_reads() -> BenchmarkResult {
    let dir = bench_dir("miss_reads");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    // Prepare sparse data
    for i in 0..1000 {
        if i % 10 == 0 {
            store.set(&format!("key{}", i), "value", false).unwrap();
        }
    }
    store.flush().unwrap();
    
    let operations = 9000; // Read mostly missing keys
    let start = Instant::now();
    
    for i in 0..1000 {
        if i % 10 != 0 {
            for _ in 0..9 {
                store.get(&format!("key{}", i)).unwrap();
            }
        }
    }
    let duration = start.elapsed();
    
    cleanup(&dir);
    
    BenchmarkResult::new("Miss Reads", operations, duration)
        .with_note("90% missing keys (bloom filter test)")
}

// ==================== SUBTREE BENCHMARKS ====================

fn bench_subtree_operations() -> BenchmarkResult {
    let dir = bench_dir("subtree");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    // Create tree structure
    for user in 0..100 {
        for field in 0..10 {
            let key = format!("users/{}/field{}", user, field);
            store.set(&key, "data", false).unwrap();
        }
    }
    store.flush().unwrap();
    
    let operations = 100;
    let start = Instant::now();
    
    for user in 0..operations {
        store.get(&format!("users/{}/", user)).unwrap();
    }
    let duration = start.elapsed();
    
    cleanup(&dir);
    
    BenchmarkResult::new("Subtree Gets", operations, duration)
        .with_note("100 subtrees with 10 keys each")
}

fn bench_subtree_deletes() -> BenchmarkResult {
    let dir = bench_dir("subtree_del");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    // Create tree structure
    for batch in 0..100 {
        for item in 0..100 {
            let key = format!("batch{}/{}", batch, item);
            store.set(&key, "data", false).unwrap();
        }
    }
    store.flush().unwrap();
    
    let operations = 100;
    let start = Instant::now();
    
    for batch in 0..operations {
        store.delete_subtree(&format!("batch{}/", batch)).unwrap();
    }
    store.flush().unwrap();
    
    let duration = start.elapsed();
    cleanup(&dir);
    
    BenchmarkResult::new("Subtree Deletes", operations, duration)
        .with_note("Delete 100 subtrees with 100 keys each")
}

// ==================== CONCURRENT BENCHMARKS ====================

fn bench_concurrent_writes() -> BenchmarkResult {
    let dir = bench_dir("concurrent_writes");
    let store = Arc::new(Store::open(std::path::Path::new(&dir)).unwrap());
    
    let threads = 8;
    let ops_per_thread = 1000;
    let total_ops = threads * ops_per_thread;
    
    let start = Instant::now();
    let mut handles = vec![];
    
    for thread_id in 0..threads {
        let store_clone = store.clone();
        let handle = thread::spawn(move || {
            for i in 0..ops_per_thread {
                let key = format!("thread{}/key{}", thread_id, i);
                store_clone.set(&key, "value", false).unwrap();
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    store.flush().unwrap();
    
    let duration = start.elapsed();
    cleanup(&dir);
    
    BenchmarkResult::new("Concurrent Writes", total_ops, duration)
        .with_note("8 threads writing 1000 keys each")
}

fn bench_concurrent_reads() -> BenchmarkResult {
    let dir = bench_dir("concurrent_reads");
    let store = Arc::new(Store::open(std::path::Path::new(&dir)).unwrap());
    
    // Prepare data
    for i in 0..1000 {
        store.set(&format!("key{}", i), "value", false).unwrap();
    }
    store.flush().unwrap();
    
    let threads = 8;
    let ops_per_thread = 10000;
    let total_ops = threads * ops_per_thread;
    
    let start = Instant::now();
    let mut handles = vec![];
    
    for _ in 0..threads {
        let store_clone = store.clone();
        let handle = thread::spawn(move || {
            for i in 0..ops_per_thread {
                store_clone.get(&format!("key{}", i % 1000)).unwrap();
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let duration = start.elapsed();
    cleanup(&dir);
    
    BenchmarkResult::new("Concurrent Reads", total_ops, duration)
        .with_note("8 threads reading from 1000 keys")
}

// ==================== RECOVERY BENCHMARKS ====================

fn bench_wal_replay() -> BenchmarkResult {
    let dir = bench_dir("wal_replay");
    
    // Write data without flushing
    {
        let store = Store::open(std::path::Path::new(&dir)).unwrap();
        for i in 0..1000 {
            store.set(&format!("wal{}", i), "value", false).unwrap();
        }
        // No flush - simulate crash
    }
    
    // Measure recovery time
    let start = Instant::now();
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    let duration = start.elapsed();
    
    // Verify data recovered
    for i in 0..1000 {
        assert!(store.get(&format!("wal{}", i)).unwrap().is_some());
    }
    
    cleanup(&dir);
    
    BenchmarkResult::new("WAL Replay", 1000, duration)
        .with_note("Recovery of 1000 unflushed operations")
}

fn bench_segment_loading() -> BenchmarkResult {
    let dir = bench_dir("segment_load");
    
    // Create multiple segments
    {
        let store = Store::open(std::path::Path::new(&dir)).unwrap();
        for batch in 0..10 {
            for i in 0..1000 {
                store.set(&format!("seg{}/key{}", batch, i), "value", false).unwrap();
            }
            store.flush().unwrap();
        }
    }
    
    // Measure startup time with segments
    let start = Instant::now();
    let _store = Store::open(std::path::Path::new(&dir)).unwrap();
    let duration = start.elapsed();
    
    cleanup(&dir);
    
    BenchmarkResult::new("Segment Loading", 10, duration)
        .with_note("Load 10 segments on startup")
}

// ==================== STRESS TESTS ====================

fn bench_stress_test() -> BenchmarkResult {
    let dir = bench_dir("stress");
    let store = Arc::new(Store::open(std::path::Path::new(&dir)).unwrap());
    
    let operations = 50000;
    let start = Instant::now();
    
    // Mix of operations
    let mut handles = vec![];
    
    // Writer thread
    let store_write = store.clone();
    handles.push(thread::spawn(move || {
        for i in 0..10000 {
            store_write.set(&format!("stress/{}", i), "value", false).unwrap();
        }
    }));
    
    // Reader threads
    for _ in 0..3 {
        let store_read = store.clone();
        handles.push(thread::spawn(move || {
            for i in 0..10000 {
                store_read.get(&format!("stress/{}", i % 1000));
            }
        }));
    }
    
    // Deleter thread
    let store_del = store.clone();
    handles.push(thread::spawn(move || {
        for i in 0..10000 {
            if i % 2 == 0 {
                store_del.delete(&format!("stress/{}", i));
            }
        }
    }));
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let duration = start.elapsed();
    cleanup(&dir);
    
    BenchmarkResult::new("Stress Test", operations, duration)
        .with_note("Mixed read/write/delete operations")
}

// ==================== REPORT GENERATION ====================

fn format_duration(d: Duration) -> String {
    if d.as_secs() > 0 {
        format!("{:.2}s", d.as_secs_f64())
    } else if d.as_millis() > 0 {
        format!("{}ms", d.as_millis())
    } else {
        format!("{}μs", d.as_micros())
    }
}

fn print_result(result: &BenchmarkResult) {
    let notes = if result.notes.is_empty() { 
        String::new() 
    } else { 
        format!("({})", result.notes) 
    };
    
    println!("  {:30} {:>10} ops in {:>8} = {:>12.0} ops/sec {}",
        result.name,
        result.operations,
        format_duration(result.duration),
        result.ops_per_sec,
        notes
    );
}

fn print_section(title: &str) {
    println!("\n{}", "=".repeat(80));
    println!("{:^80}", title);
    println!("{}", "=".repeat(80));
}

fn main() {
    println!("\n{:^80}", "ANTLER PERFORMANCE BENCHMARK");
    println!("{:^80}", "High-Performance Tree Store");
    println!("{}", "=".repeat(80));
    
    let mut results = Vec::new();
    
    // Run write benchmarks
    print_section("WRITE PERFORMANCE");
    let benchmarks = vec![
        bench_sequential_writes,
        bench_random_writes,
        bench_batch_writes,
        bench_large_values,
    ];
    
    for bench in benchmarks {
        let result = bench();
        print_result(&result);
        results.push(result);
    }
    
    // Run read benchmarks
    print_section("READ PERFORMANCE");
    let benchmarks = vec![
        bench_sequential_reads,
        bench_random_reads,
        bench_cache_hit_rate,
        bench_miss_reads,
    ];
    
    for bench in benchmarks {
        let result = bench();
        print_result(&result);
        results.push(result);
    }
    
    // Run subtree benchmarks
    print_section("SUBTREE OPERATIONS");
    let benchmarks = vec![
        bench_subtree_operations,
        bench_subtree_deletes,
    ];
    
    for bench in benchmarks {
        let result = bench();
        print_result(&result);
        results.push(result);
    }
    
    // Run concurrent benchmarks
    print_section("CONCURRENT PERFORMANCE");
    let benchmarks = vec![
        bench_concurrent_writes,
        bench_concurrent_reads,
    ];
    
    for bench in benchmarks {
        let result = bench();
        print_result(&result);
        results.push(result);
    }
    
    // Run recovery benchmarks
    print_section("RECOVERY & STARTUP");
    let benchmarks = vec![
        bench_wal_replay,
        bench_segment_loading,
    ];
    
    for bench in benchmarks {
        let result = bench();
        print_result(&result);
        results.push(result);
    }
    
    // Run stress test
    print_section("STRESS TEST");
    let result = bench_stress_test();
    print_result(&result);
    results.push(result);
    
    // Summary
    print_section("SUMMARY");
    
    let total_ops: usize = results.iter().map(|r| r.operations).sum();
    let total_time: Duration = results.iter().map(|r| r.duration).sum();
    
    println!("\n  Total Operations: {:>15}", total_ops);
    println!("  Total Time:       {:>15}", format_duration(total_time));
    println!("  Average Throughput: {:>13.0} ops/sec", total_ops as f64 / total_time.as_secs_f64());
    
    // Performance assertions
    println!("\n{}", "=".repeat(80));
    println!("{:^80}", "PERFORMANCE REQUIREMENTS");
    println!("{}", "=".repeat(80));
    
    let seq_writes = results.iter().find(|r| r.name == "Sequential Writes").unwrap();
    let seq_reads = results.iter().find(|r| r.name == "Sequential Reads").unwrap();
    
    let write_target = 3000.0;
    let read_target = 50000.0;
    
    println!("\n  Write Performance Target: {} ops/sec", write_target);
    println!("  Actual: {:.0} ops/sec - {}", 
        seq_writes.ops_per_sec,
        if seq_writes.ops_per_sec >= write_target { "✅ PASS" } else { "❌ FAIL" }
    );
    
    println!("\n  Read Performance Target: {} ops/sec", read_target);
    println!("  Actual: {:.0} ops/sec - {}", 
        seq_reads.ops_per_sec,
        if seq_reads.ops_per_sec >= read_target { "✅ PASS" } else { "❌ FAIL" }
    );
    
    println!("\n{}", "=".repeat(80));
    
    // Check if targets met
    if seq_writes.ops_per_sec < write_target || seq_reads.ops_per_sec < read_target {
        println!("\n⚠️  WARNING: Performance targets not met!");
        std::process::exit(1);
    } else {
        println!("\n✅ All performance targets met!");
    }
}