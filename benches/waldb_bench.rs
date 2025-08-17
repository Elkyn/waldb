// Comprehensive benchmark suite for WalDB
#![feature(test)]

extern crate test;

use test::Bencher;
use std::path::Path;
use tempfile::tempdir;

mod waldb_store {
    include!("../waldb.rs");
}

use waldb_store::Store;

// Helper to create a temporary store
fn temp_store() -> (Store, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    (store, dir)
}

// ==================== WRITE BENCHMARKS ====================

#[bench]
fn bench_write_sequential(b: &mut Bencher) {
    let (store, _dir) = temp_store();
    let mut i = 0;
    
    b.iter(|| {
        let key = format!("key{:06}", i);
        store.set(&key, "benchmark_value", false).unwrap();
        i += 1;
    });
}

#[bench]
fn bench_write_random(b: &mut Bencher) {
    let (store, _dir) = temp_store();
    
    b.iter(|| {
        let key = format!("key{:06}", rand::random::<u32>() % 1000000);
        store.set(&key, "benchmark_value", false).unwrap();
    });
}

#[bench]
fn bench_write_nested(b: &mut Bencher) {
    let (store, _dir) = temp_store();
    let mut i = 0;
    
    b.iter(|| {
        let key = format!("users/{}/data/field{}", i / 100, i % 100);
        store.set(&key, "value", false).unwrap();
        i += 1;
    });
}

#[bench]
fn bench_write_large_values(b: &mut Bencher) {
    let (store, _dir) = temp_store();
    let large_value = "x".repeat(1000); // 1KB value
    let mut i = 0;
    
    b.iter(|| {
        let key = format!("large{:06}", i);
        store.set(&key, &large_value, false).unwrap();
        i += 1;
    });
}

// ==================== READ BENCHMARKS ====================

#[bench]
fn bench_read_hot(b: &mut Bencher) {
    let (store, _dir) = temp_store();
    
    // Pre-populate with data
    for i in 0..1000 {
        store.set(&format!("key{:04}", i), "value", false).unwrap();
    }
    
    b.iter(|| {
        store.get("key0500").unwrap();
    });
}

#[bench]
fn bench_read_cold(b: &mut Bencher) {
    let (store, _dir) = temp_store();
    
    // Pre-populate and flush to disk
    for i in 0..10000 {
        store.set(&format!("key{:06}", i), "value", false).unwrap();
    }
    store.flush().unwrap();
    
    // Read different keys to avoid cache
    let mut i = 0;
    b.iter(|| {
        let key = format!("key{:06}", i % 10000);
        store.get(&key).unwrap();
        i += 1;
    });
}

#[bench]
fn bench_read_missing(b: &mut Bencher) {
    let (store, _dir) = temp_store();
    
    // Pre-populate with some data
    for i in 0..1000 {
        store.set(&format!("exists{:04}", i), "value", false).unwrap();
    }
    
    b.iter(|| {
        store.get("missing_key").unwrap();
    });
}

#[bench]
fn bench_read_subtree(b: &mut Bencher) {
    let (store, _dir) = temp_store();
    
    // Create a subtree
    for i in 0..100 {
        store.set(&format!("tree/node{:03}/value", i), "data", false).unwrap();
        store.set(&format!("tree/node{:03}/metadata", i), "meta", false).unwrap();
    }
    store.flush().unwrap();
    
    b.iter(|| {
        store.get("tree/").unwrap();
    });
}

// ==================== RANGE QUERY BENCHMARKS ====================

#[bench]
fn bench_range_query_small(b: &mut Bencher) {
    let (store, _dir) = temp_store();
    
    // Pre-populate
    for i in 0..1000 {
        store.set(&format!("item{:04}", i), "value", false).unwrap();
    }
    store.flush().unwrap();
    
    b.iter(|| {
        store.get_range_limit("item0100", "item0200", 10).unwrap();
    });
}

#[bench]
fn bench_range_query_large(b: &mut Bencher) {
    let (store, _dir) = temp_store();
    
    // Pre-populate
    for i in 0..10000 {
        store.set(&format!("item{:05}", i), "value", false).unwrap();
    }
    store.flush().unwrap();
    
    b.iter(|| {
        store.get_range("item01000", "item02000").unwrap();
    });
}

#[bench]
fn bench_prefix_scan(b: &mut Bencher) {
    let (store, _dir) = temp_store();
    
    // Pre-populate with prefixed data
    for i in 0..1000 {
        store.set(&format!("prefix/sub{:03}/data", i), "value", false).unwrap();
    }
    store.flush().unwrap();
    
    b.iter(|| {
        store.scan_prefix("prefix/sub1", 50).unwrap();
    });
}

// ==================== PATTERN MATCHING BENCHMARKS ====================

#[bench]
fn bench_pattern_star(b: &mut Bencher) {
    let (store, _dir) = temp_store();
    
    // Pre-populate
    for i in 0..100 {
        store.set(&format!("users/user{:03}/name", i), "name", false).unwrap();
        store.set(&format!("users/user{:03}/age", i), "25", false).unwrap();
    }
    store.flush().unwrap();
    
    b.iter(|| {
        store.get_pattern("users/*/name").unwrap();
    });
}

#[bench]
fn bench_pattern_question(b: &mut Bencher) {
    let (store, _dir) = temp_store();
    
    // Pre-populate
    for i in 0..100 {
        store.set(&format!("log{}", i), "data", false).unwrap();
    }
    store.flush().unwrap();
    
    b.iter(|| {
        store.get_pattern("log?").unwrap();
    });
}

// ==================== DELETE BENCHMARKS ====================

#[bench]
fn bench_delete_single(b: &mut Bencher) {
    let (store, _dir) = temp_store();
    let mut i = 0;
    
    // Pre-populate
    for j in 0..100000 {
        store.set(&format!("del{:06}", j), "value", false).unwrap();
    }
    
    b.iter(|| {
        store.delete(&format!("del{:06}", i)).unwrap();
        i += 1;
    });
}

#[bench]
fn bench_delete_subtree(b: &mut Bencher) {
    let (store, _dir) = temp_store();
    let mut i = 0;
    
    b.iter(|| {
        // Create subtree
        for j in 0..10 {
            store.set(&format!("temp{}/item{}", i, j), "value", false).unwrap();
        }
        // Delete entire subtree
        store.delete_subtree(&format!("temp{}/", i)).unwrap();
        i += 1;
    });
}

// ==================== MIXED WORKLOAD BENCHMARKS ====================

#[bench]
fn bench_mixed_workload(b: &mut Bencher) {
    let (store, _dir) = temp_store();
    let mut i = 0;
    
    // Pre-populate
    for j in 0..1000 {
        store.set(&format!("mixed{:04}", j), "value", false).unwrap();
    }
    
    b.iter(|| {
        match i % 4 {
            0 => {
                // Write
                store.set(&format!("mixed{:04}", i % 1000), "updated", false).unwrap();
            }
            1 => {
                // Read
                store.get(&format!("mixed{:04}", i % 1000)).unwrap();
            }
            2 => {
                // Range query
                let start = format!("mixed{:04}", (i % 900));
                let end = format!("mixed{:04}", (i % 900) + 10);
                store.get_range(&start, &end).unwrap();
            }
            _ => {
                // Delete
                store.delete(&format!("mixed{:04}", i % 1000)).unwrap();
            }
        }
        i += 1;
    });
}

// ==================== COMPACTION BENCHMARKS ====================

#[bench]
fn bench_write_with_compaction(b: &mut Bencher) {
    let (store, _dir) = temp_store();
    let mut i = 0;
    
    b.iter(|| {
        store.set(&format!("compact{:06}", i), "value", false).unwrap();
        
        // Force flush periodically to trigger compaction
        if i % 1000 == 0 {
            store.flush().unwrap();
        }
        i += 1;
    });
}

// ==================== CONCURRENT BENCHMARKS ====================

#[bench]
fn bench_concurrent_reads(b: &mut Bencher) {
    use std::sync::Arc;
    use std::thread;
    
    let (store, dir) = temp_store();
    let store = Arc::new(store);
    
    // Pre-populate
    for i in 0..1000 {
        store.set(&format!("concurrent{:04}", i), "value", false).unwrap();
    }
    
    b.iter(|| {
        let mut handles = vec![];
        
        for _ in 0..4 {
            let store = store.clone();
            let handle = thread::spawn(move || {
                for i in 0..10 {
                    store.get(&format!("concurrent{:04}", i * 100)).unwrap();
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
    });
}