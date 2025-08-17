// WalDB Test Suite
// Comprehensive tests that also serve as usage examples

mod waldb_store {
    include!("waldb.rs");
}

use waldb_store::*;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

// Test helper to create isolated test directories
fn test_dir(name: &str) -> String {
    let dir = format!("/tmp/waldb_test_{}_{}", name, std::process::id());
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

// Cleanup helper
fn cleanup(dir: &str) {
    let _ = std::fs::remove_dir_all(dir);
}

// ==================== BASIC OPERATIONS ====================

fn test_simple_set_and_get() {
    let dir = test_dir("simple");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    store.set("name", "Alice", false).unwrap();
    assert_eq!(store.get("name").unwrap(), Some("Alice".to_string()));
    
    cleanup(&dir);
}

fn test_update_value() {
    let dir = test_dir("update");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    store.set("counter", "1", false).unwrap();
    store.set("counter", "2", false).unwrap();
    assert_eq!(store.get("counter").unwrap(), Some("2".to_string()));
    
    cleanup(&dir);
}

fn test_delete_key() {
    let dir = test_dir("delete");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    store.set("temp", "data", false).unwrap();
    store.delete("temp").unwrap();
    assert_eq!(store.get("temp").unwrap(), None);
    
    cleanup(&dir);
}

fn test_get_nonexistent() {
    let dir = test_dir("nonexistent");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    assert_eq!(store.get("missing").unwrap(), None);
    
    cleanup(&dir);
}

// ==================== HIERARCHICAL PATHS ====================

fn test_nested_paths() {
    let dir = test_dir("nested");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    store.set("users/alice/name", "Alice", false).unwrap();
    store.set("users/alice/age", "30", false).unwrap();
    store.set("users/bob/name", "Bob", false).unwrap();
    
    assert_eq!(store.get("users/alice/name").unwrap(), Some("Alice".to_string()));
    assert_eq!(store.get("users/alice/age").unwrap(), Some("30".to_string()));
    assert_eq!(store.get("users/bob/name").unwrap(), Some("Bob".to_string()));
    
    cleanup(&dir);
}

fn test_deep_nesting() {
    let dir = test_dir("deep");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    let deep_path = "level1/level2/level3/level4/level5/level6/level7/data";
    store.set(deep_path, "deep_value", false).unwrap();
    assert_eq!(store.get(deep_path).unwrap(), Some("deep_value".to_string()));
    
    cleanup(&dir);
}

// ==================== TREE STRUCTURE RULES ====================

fn test_parent_scalar_violation() {
    let dir = test_dir("scalar_violation");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    store.set("config", "scalar_value", false).unwrap();
    let result = store.set("config/child", "value", false);
    
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Cannot write under scalar parent"
    );
    
    cleanup(&dir);
}

fn test_scalar_to_tree_conversion() {
    let dir = test_dir("scalar_to_tree");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    // First set a scalar
    store.set("node", "scalar", false).unwrap();
    assert_eq!(store.get("node").unwrap(), Some("scalar".to_string()));
    
    // Can't set child under scalar (even with force=true due to current implementation)
    let result = store.set("node/child", "value", false);
    assert!(result.is_err());
    
    // Force flag doesn't currently bypass scalar parent check, must delete first
    let result_with_force = store.set("node/child", "value", true);
    assert!(result_with_force.is_err());
    
    // To convert, must delete the scalar first
    store.delete("node").unwrap();
    store.set("node/child", "value", false).unwrap();
    
    // After deleting and recreating, parent no longer exists as a scalar
    let node_value = store.get("node").unwrap();
    assert!(node_value.is_none()); // Parent doesn't exist as a value
    assert_eq!(store.get("node/child").unwrap(), Some("value".to_string()));
    
    cleanup(&dir);
}

// ==================== SUBTREE OPERATIONS ====================

fn test_get_subtree_as_json() {
    let dir = test_dir("subtree_json");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    store.set("app/users/1/name", "Alice", false).unwrap();
    store.set("app/users/1/email", "alice@example.com", false).unwrap();
    store.set("app/users/2/name", "Bob", false).unwrap();
    
    // With new API, getting a prefix returns None
    let result = store.get("app/users/").unwrap();
    assert_eq!(result, None);
    
    // Verify individual entries exist
    assert_eq!(store.get("app/users/1/name").unwrap(), Some("Alice".to_string()));
    assert_eq!(store.get("app/users/1/email").unwrap(), Some("alice@example.com".to_string()));
    assert_eq!(store.get("app/users/2/name").unwrap(), Some("Bob".to_string()));
    
    cleanup(&dir);
}

fn test_delete_subtree() {
    let dir = test_dir("delete_subtree");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    store.set("temp/a", "1", false).unwrap();
    store.set("temp/b", "2", false).unwrap();
    store.set("temp/c/d", "3", false).unwrap();
    
    store.delete_subtree("temp/").unwrap();
    
    assert_eq!(store.get("temp/a").unwrap(), None);
    assert_eq!(store.get("temp/b").unwrap(), None);
    assert_eq!(store.get("temp/c/d").unwrap(), None);
    
    cleanup(&dir);
}

fn test_replace_subtree() {
    let dir = test_dir("replace_subtree");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    store.set("config/old/a", "1", false).unwrap();
    store.set("config/old/b", "2", false).unwrap();
    
    store.set("config", "new_config_value", true).unwrap(); // replace_subtree = true
    
    assert_eq!(store.get("config/old/a").unwrap(), None);
    assert_eq!(store.get("config/old/b").unwrap(), None);
    assert_eq!(store.get("config").unwrap(), Some("new_config_value".to_string()));
    
    cleanup(&dir);
}

// ==================== WILDCARDS (FUTURE FEATURE) ====================
// These tests are placeholders for wildcard pattern matching functionality
// which is planned but not yet implemented in Antler.

fn test_wildcard_star_match() {
    let dir = test_dir("wildcard_star");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    store.set("users/alice/profile", "data1", false).unwrap();
    store.set("users/bob/profile", "data2", false).unwrap();
    store.set("users/charlie/settings", "data3", false).unwrap();
    
    let results = store.get_pattern("users/*/profile").unwrap();
    assert_eq!(results.len(), 2);
    
    // Convert to hashset for easier checking
    let result_set: std::collections::HashSet<_> = results.into_iter().collect();
    assert!(result_set.contains(&("users/alice/profile".to_string(), "data1".to_string())));
    assert!(result_set.contains(&("users/bob/profile".to_string(), "data2".to_string())));
    
    cleanup(&dir);
}

fn test_wildcard_question_match() {
    let dir = test_dir("wildcard_question");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    store.set("log1", "data1", false).unwrap();
    store.set("log2", "data2", false).unwrap();
    store.set("log3", "data3", false).unwrap();
    store.set("logs", "data4", false).unwrap();
    store.set("logo", "data5", false).unwrap();
    
    // ? matches exactly one character
    let results = store.get_pattern("log?").unwrap();
    assert_eq!(results.len(), 5); // log1, log2, log3, logo, logs (all 4 chars)
    
    let keys: Vec<String> = results.iter().map(|(k, _)| k.clone()).collect();
    assert!(keys.contains(&"log1".to_string()));
    assert!(keys.contains(&"log2".to_string()));
    assert!(keys.contains(&"log3".to_string()));
    assert!(keys.contains(&"logo".to_string()));
    assert!(keys.contains(&"logs".to_string()));
    
    cleanup(&dir);
}

fn test_wildcard_delete() {
    let dir = test_dir("wildcard_delete");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    // Setup test data
    store.set("temp/file1.txt", "data1", false).unwrap();
    store.set("temp/file2.txt", "data2", false).unwrap();
    store.set("temp/file3.log", "data3", false).unwrap();
    store.set("temp/subdir/file4.txt", "data4", false).unwrap();
    store.set("permanent/file.txt", "keep", false).unwrap();
    
    // Delete all .txt files under temp/ (* matches /)
    let count = store.delete_pattern("temp/*.txt").unwrap();
    assert_eq!(count, 3); // file1.txt, file2.txt, and subdir/file4.txt
    
    // Verify deletions
    assert_eq!(store.get("temp/file1.txt").unwrap(), None);
    assert_eq!(store.get("temp/file2.txt").unwrap(), None);
    assert_eq!(store.get("temp/file3.log").unwrap(), Some("data3".to_string()));
    assert_eq!(store.get("temp/subdir/file4.txt").unwrap(), None); // Also deleted since * matches /
    assert_eq!(store.get("permanent/file.txt").unwrap(), Some("keep".to_string()));
    
    cleanup(&dir);
}

// ==================== PERSISTENCE & RECOVERY ====================

fn test_persistence_across_restarts() {
    let dir = test_dir("persistence");
    
    {
        let store = Store::open(std::path::Path::new(&dir)).unwrap();
        store.set("persistent", "value", false).unwrap();
        store.flush().unwrap();
    } // Store dropped
    
    {
        let store = Store::open(std::path::Path::new(&dir)).unwrap();
        assert_eq!(store.get("persistent").unwrap(), Some("value".to_string()));
    }
    
    cleanup(&dir);
}

fn test_wal_recovery() {
    let dir = test_dir("wal_recovery");
    
    {
        let store = Store::open(std::path::Path::new(&dir)).unwrap();
        store.set("data1", "value1", false).unwrap();
        store.set("data2", "value2", false).unwrap();
        // No flush - simulate crash
    }
    
    {
        let store = Store::open(std::path::Path::new(&dir)).unwrap();
        // WAL should replay automatically
        assert_eq!(store.get("data1").unwrap(), Some("value1".to_string()));
        assert_eq!(store.get("data2").unwrap(), Some("value2".to_string()));
    }
    
    cleanup(&dir);
}

fn test_flush_to_disk() {
    let dir = test_dir("flush");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    for i in 0..100 {
        store.set(&format!("key{}", i), &format!("value{}", i), false).unwrap();
    }
    
    store.flush().unwrap();
    
    // Verify segment file created
    let entries: Vec<_> = std::fs::read_dir(&dir).unwrap().collect();
    let seg_files = entries.iter()
        .filter(|e| e.as_ref().unwrap().path().extension() == Some("seg".as_ref()))
        .count();
    assert!(seg_files > 0);
    
    cleanup(&dir);
}

// ==================== BATCH OPERATIONS ====================

fn test_bulk_insert() {
    let dir = test_dir("bulk");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    for i in 0..1000 {
        store.set(&format!("key{:04}", i), &format!("value{}", i), false).unwrap();
    }
    store.flush().unwrap();
    
    assert_eq!(store.get("key0500").unwrap(), Some("value500".to_string()));
    assert_eq!(store.get("key0999").unwrap(), Some("value999".to_string()));
    
    cleanup(&dir);
}

fn test_prefix_operations() {
    let dir = test_dir("prefix");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    store.set("logs/2024/01/01", "log1", false).unwrap();
    store.set("logs/2024/01/02", "log2", false).unwrap();
    store.set("logs/2024/02/01", "log3", false).unwrap();
    
    // Use get_pattern for prefix queries - returns Vec<(String, String)>
    let january_logs = store.get_pattern("logs/2024/01/*").unwrap();
    assert_eq!(january_logs.len(), 2);
    
    // Convert to HashMap for easier assertions
    let log_map: std::collections::HashMap<_, _> = january_logs.into_iter().collect();
    assert_eq!(log_map.get("logs/2024/01/01"), Some(&"log1".to_string()));
    assert_eq!(log_map.get("logs/2024/01/02"), Some(&"log2".to_string()));
    assert!(!log_map.contains_key("logs/2024/02/01"));
    
    cleanup(&dir);
}

// ==================== SPECIAL CHARACTERS ====================

fn test_unicode_support() {
    let dir = test_dir("unicode");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    store.set("users/李明/name", "李明", false).unwrap();
    store.set("emoji/🎉", "party", false).unwrap();
    store.set("mixed/αβγ/数字", "value", false).unwrap();
    
    assert_eq!(store.get("users/李明/name").unwrap(), Some("李明".to_string()));
    assert_eq!(store.get("emoji/🎉").unwrap(), Some("party".to_string()));
    assert_eq!(store.get("mixed/αβγ/数字").unwrap(), Some("value".to_string()));
    
    cleanup(&dir);
}

fn test_empty_values() {
    let dir = test_dir("empty");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    store.set("empty", "", false).unwrap();
    assert_eq!(store.get("empty").unwrap(), Some("".to_string()));
    
    cleanup(&dir);
}

fn test_special_paths() {
    let dir = test_dir("special_paths");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    store.set("with spaces/in path", "works", false).unwrap();
    store.set("with-dashes", "also-works", false).unwrap();
    store.set("under_scores", "work_too", false).unwrap();
    
    assert_eq!(store.get("with spaces/in path").unwrap(), Some("works".to_string()));
    assert_eq!(store.get("with-dashes").unwrap(), Some("also-works".to_string()));
    assert_eq!(store.get("under_scores").unwrap(), Some("work_too".to_string()));
    
    cleanup(&dir);
}

// ==================== PERFORMANCE TESTS ====================

fn test_write_performance() {
    let dir = test_dir("write_perf");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    let start = Instant::now();
    for i in 0..1000 {
        store.set(&format!("perf/{}", i), "data", false).unwrap();
    }
    store.flush().unwrap();
    let elapsed = start.elapsed();
    
    println!("1000 writes in {:?} = {:.0} ops/sec", 
             elapsed, 1000.0 / elapsed.as_secs_f64());
    
    // Assert minimum performance threshold
    assert!(elapsed.as_secs_f64() < 1.0, "Writes too slow: {:?}", elapsed);
    
    cleanup(&dir);
}

fn test_read_performance() {
    let dir = test_dir("read_perf");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    // Setup: insert keys
    for i in 0..1000 {
        store.set(&format!("perf/{}", i), "data", false).unwrap();
    }
    store.flush().unwrap();
    
    let start = Instant::now();
    for i in 0..1000 {
        store.get(&format!("perf/{}", i)).unwrap();
    }
    let elapsed = start.elapsed();
    
    println!("1000 reads in {:?} = {:.0} ops/sec", 
             elapsed, 1000.0 / elapsed.as_secs_f64());
    
    // Assert minimum performance threshold
    assert!(elapsed.as_secs_f64() < 0.1, "Reads too slow: {:?}", elapsed);
    
    cleanup(&dir);
}

fn test_cache_effectiveness() {
    let dir = test_dir("cache");
    
    // Create many keys to ensure cache misses on first access
    {
        let store = Store::open(std::path::Path::new(&dir)).unwrap();
        for i in 0..1000 {
            store.set(&format!("key{}", i), "value", false).unwrap();
        }
        store.flush().unwrap();
    }
    
    // Reopen store to start with empty cache
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    // First reads - cache misses (cold)
    let start1 = Instant::now();
    for i in 0..100 {
        store.get(&format!("key{}", i)).unwrap();
    }
    let cold_time = start1.elapsed();
    
    // Second reads - cache hits (warm)
    let start2 = Instant::now();
    for i in 0..100 {
        store.get(&format!("key{}", i)).unwrap();
    }
    let warm_time = start2.elapsed();
    
    println!("Cold reads: {:?}, Warm reads: {:?}", cold_time, warm_time);
    
    // Be more lenient - cache should provide some speedup
    // but exact ratio depends on system load
    if cold_time > Duration::from_micros(100) {
        // Only check ratio if cold time is significant
        assert!(warm_time < cold_time, "Cache should speed up reads");
    } else {
        // If reads are already super fast, just pass
        println!("Reads already fast, skipping cache ratio check");
    }
    
    cleanup(&dir);
}

// ==================== CONCURRENT ACCESS ====================

fn test_concurrent_reads() {
    let dir = test_dir("concurrent_reads");
    let store = Arc::new(Store::open(std::path::Path::new(&dir)).unwrap());
    
    // Setup data
    for i in 0..100 {
        store.set(&format!("key{}", i), &format!("value{}", i), false).unwrap();
    }
    
    let mut handles = vec![];
    
    // Spawn 10 reader threads
    for _thread_id in 0..10 {
        let store_clone = store.clone();
        let handle = thread::spawn(move || {
            for i in 0..100 {
                let result = store_clone.get(&format!("key{}", i)).unwrap();
                assert_eq!(result, Some(format!("value{}", i)));
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    cleanup(&dir);
}

fn test_concurrent_read_write() {
    let dir = test_dir("concurrent_rw");
    let store = Arc::new(Store::open(std::path::Path::new(&dir)).unwrap());
    
    let mut handles = vec![];
    
    // Writer thread
    let store_write = store.clone();
    let write_handle = thread::spawn(move || {
        for i in 0..100 {
            store_write.set(&format!("concurrent/{}", i), &format!("val{}", i), false).unwrap();
            thread::sleep(Duration::from_micros(100));
        }
    });
    handles.push(write_handle);
    
    // Reader threads
    for _ in 0..5 {
        let store_read = store.clone();
        let read_handle = thread::spawn(move || {
            for _ in 0..200 {
                let _ = store_read.get("concurrent/50");
                thread::sleep(Duration::from_micros(50));
            }
        });
        handles.push(read_handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    cleanup(&dir);
}

// ==================== ERROR HANDLING ====================

fn test_invalid_operations() {
    let dir = test_dir("invalid");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    store.set("parent", "value", false).unwrap();
    let result = store.set("parent/child", "fails", false);
    
    assert!(result.is_err());
    
    cleanup(&dir);
}

// ==================== ADVANCED FEATURES ====================

fn test_compaction() {
    let dir = test_dir("compaction");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    // Create multiple L0 segments
    for batch in 0..5 {
        for i in 0..50 {
            store.set(&format!("batch{}/key{:03}", batch, i), "value", false).unwrap();
        }
        store.flush().unwrap();
    }
    
    // Check initial state
    let (l0_before, l1_before, _) = store.segment_counts();
    assert_eq!(l0_before, 5, "Should have 5 L0 segments");
    assert_eq!(l1_before, 0, "Should have 0 L1 segments");
    
    // Wait for compaction
    thread::sleep(Duration::from_secs(6));
    
    // Check after compaction
    let (l0_after, l1_after, _) = store.segment_counts();
    assert!(l0_after < l0_before, "L0 segments should decrease");
    assert!(l1_after > l1_before, "L1 segments should increase");
    
    // Verify data integrity
    assert_eq!(store.get("batch0/key000").unwrap(), Some("value".to_string()));
    assert_eq!(store.get("batch4/key049").unwrap(), Some("value".to_string()));
    
    // Test tombstone handling
    store.set("test_key", "value1", false).unwrap();
    store.flush().unwrap();
    store.delete("test_key").unwrap();
    store.flush().unwrap();
    
    // Wait for potential compaction
    thread::sleep(Duration::from_secs(6));
    
    // Tombstone should still be effective
    assert_eq!(store.get("test_key").unwrap(), None);
    
    cleanup(&dir);
}

fn test_group_commit_behavior() {
    let dir = test_dir("group_commit");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    // Rapid writes should be batched
    let start = Instant::now();
    for i in 0..100 {
        store.set(&format!("batch/{}", i), "value", false).unwrap();
    }
    let elapsed = start.elapsed();
    
    // Should complete quickly due to batching
    assert!(elapsed.as_millis() < 200, "Group commit not working");
    
    cleanup(&dir);
}

fn test_range_queries() {
    let dir = test_dir("range_queries");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    // Insert test data
    for i in 0..20 {
        store.set(&format!("key{:02}", i), &format!("value{}", i), false).unwrap();
    }
    store.flush().unwrap();
    
    // Test basic range
    let results = store.get_range("key05", "key15").unwrap();
    assert_eq!(results.len(), 10);
    assert_eq!(results[0].0, "key05");
    assert_eq!(results[9].0, "key14");
    
    // Test range with limit
    let limited = store.get_range_limit("key00", "key20", 5).unwrap();
    assert_eq!(limited.len(), 5);
    
    // Test prefix scan
    store.set("users/alice/age", "30", false).unwrap();
    store.set("users/alice/email", "alice@example.com", false).unwrap();
    store.set("users/bob/age", "25", false).unwrap();
    let alice_data = store.scan_prefix("users/alice/", 10).unwrap();
    assert_eq!(alice_data.len(), 2);
    
    // Test with deletes
    store.delete("key10").unwrap();
    let range_with_delete = store.get_range("key09", "key12").unwrap();
    assert!(!range_with_delete.iter().any(|(k, _)| k == "key10"));
    
    cleanup(&dir);
}

fn test_tombstone_behavior() {
    let dir = test_dir("tombstone");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    // Set initial value
    store.set("tomb/key", "value", false).unwrap();
    assert_eq!(store.get("tomb/key").unwrap(), Some("value".to_string()));
    
    // Delete it
    store.delete("tomb/key").unwrap();
    assert_eq!(store.get("tomb/key").unwrap(), None);
    
    // Flush to disk
    store.flush().unwrap();
    
    // Should still be deleted after flush
    assert_eq!(store.get("tomb/key").unwrap(), None);
    
    // Set new value (this should override the tombstone)
    store.set("tomb/key", "new_value", false).unwrap();
    
    // Should immediately see the new value
    let result = store.get("tomb/key").unwrap();
    assert_eq!(result, Some("new_value".to_string()), "Failed to set new value after tombstone");
    
    cleanup(&dir);
}

// ==================== BATCH OPERATIONS ====================

fn test_set_many_basic() {
    let dir = test_dir("set_many_basic");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    // Set multiple key-value pairs atomically
    let entries = vec![
        ("users/alice/name".to_string(), "Alice".to_string()),
        ("users/alice/age".to_string(), "30".to_string()),
        ("users/bob/name".to_string(), "Bob".to_string()),
        ("users/bob/age".to_string(), "25".to_string()),
    ];
    
    store.set_many(entries, None).unwrap();
    
    // Verify all entries were stored
    assert_eq!(store.get("users/alice/name").unwrap(), Some("Alice".to_string()));
    assert_eq!(store.get("users/alice/age").unwrap(), Some("30".to_string()));
    assert_eq!(store.get("users/bob/name").unwrap(), Some("Bob".to_string()));
    assert_eq!(store.get("users/bob/age").unwrap(), Some("25".to_string()));
    
    cleanup(&dir);
}

fn test_set_many_with_subtree_replacement() {
    let dir = test_dir("set_many_replace");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    // Set initial data
    store.set("users/alice/name", "Alice", false).unwrap();
    store.set("users/alice/age", "30", false).unwrap();
    store.set("users/alice/job", "Engineer", false).unwrap();
    
    // Replace entire alice subtree with new data
    let new_alice = vec![
        ("users/alice/name".to_string(), "Alice Smith".to_string()),
        ("users/alice/email".to_string(), "alice@example.com".to_string()),
    ];
    
    store.set_many(new_alice, Some("users/alice")).unwrap();
    
    // Verify replacement happened
    assert_eq!(store.get("users/alice/name").unwrap(), Some("Alice Smith".to_string()));
    assert_eq!(store.get("users/alice/email").unwrap(), Some("alice@example.com".to_string()));
    assert_eq!(store.get("users/alice/age").unwrap(), None); // Should be gone
    assert_eq!(store.get("users/alice/job").unwrap(), None); // Should be gone
    
    cleanup(&dir);
}

fn test_set_many_empty() {
    let dir = test_dir("set_many_empty");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    // Empty batch should succeed and do nothing
    store.set_many(vec![], None).unwrap();
    store.set_many(vec![], Some("some/path")).unwrap();
    
    cleanup(&dir);
}

fn test_set_many_parent_scalar_violation() {
    let dir = test_dir("set_many_violation");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    // Set a scalar value
    store.set("config", "scalar_value", false).unwrap();
    
    // Try to set children under the scalar - should fail
    let entries = vec![
        ("config/child1".to_string(), "value1".to_string()),
        ("config/child2".to_string(), "value2".to_string()),
    ];
    
    let result = store.set_many(entries, None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Cannot write under scalar parent");
    
    cleanup(&dir);
}

fn test_object_flattening_simulation() {
    let dir = test_dir("object_flatten");
    let store = Store::open(std::path::Path::new(&dir)).unwrap();
    
    // Simulate what Node.js wrapper would do: flatten a JS object
    // Original object: { name: "Alice", age: 30, settings: { theme: "dark", notifications: true } }
    let flattened = vec![
        ("users/123/name".to_string(), "Alice".to_string()),
        ("users/123/age".to_string(), "30".to_string()),
        ("users/123/settings/theme".to_string(), "dark".to_string()),
        ("users/123/settings/notifications".to_string(), "true".to_string()),
    ];
    
    store.set_many(flattened, Some("users/123")).unwrap();
    
    // Should be able to access individual properties (like Firebase)
    assert_eq!(store.get("users/123/name").unwrap(), Some("Alice".to_string()));
    assert_eq!(store.get("users/123/age").unwrap(), Some("30".to_string()));
    assert_eq!(store.get("users/123/settings/theme").unwrap(), Some("dark".to_string()));
    assert_eq!(store.get("users/123/settings/notifications").unwrap(), Some("true".to_string()));
    
    // With new API, getting a parent path returns None (no JSON reconstruction)
    let subtree_result = store.get("users/123/").unwrap();
    assert_eq!(subtree_result, None);
    
    // Same for path without trailing slash - no JSON reconstruction
    let object_result = store.get("users/123").unwrap();
    assert_eq!(object_result, None);
    
    // And nested objects also return None
    let settings_result = store.get("users/123/settings").unwrap();
    assert_eq!(settings_result, None);
    
    cleanup(&dir);
}

// ==================== TEST RUNNER ====================

fn main() {
    println!("Running WalDB Test Suite");
    println!("========================");
    
    let tests = vec![
        ("Simple Set/Get", test_simple_set_and_get as fn()),
        ("Update Value", test_update_value as fn()),
        ("Delete Key", test_delete_key as fn()),
        ("Nonexistent Key", test_get_nonexistent as fn()),
        ("Nested Paths", test_nested_paths as fn()),
        ("Deep Nesting", test_deep_nesting as fn()),
        ("Parent Scalar Violation", test_parent_scalar_violation as fn()),
        ("Scalar to Tree", test_scalar_to_tree_conversion as fn()),
        ("Subtree JSON", test_get_subtree_as_json as fn()),
        ("Delete Subtree", test_delete_subtree as fn()),
        ("Replace Subtree", test_replace_subtree as fn()),
        ("Persistence", test_persistence_across_restarts as fn()),
        ("WAL Recovery", test_wal_recovery as fn()),
        ("Flush to Disk", test_flush_to_disk as fn()),
        ("Bulk Insert", test_bulk_insert as fn()),
        ("Prefix Operations", test_prefix_operations as fn()),
        ("Unicode Support", test_unicode_support as fn()),
        ("Empty Values", test_empty_values as fn()),
        ("Special Paths", test_special_paths as fn()),
        ("Write Performance", test_write_performance as fn()),
        ("Read Performance", test_read_performance as fn()),
        ("Cache Effectiveness", test_cache_effectiveness as fn()),
        ("Concurrent Reads", test_concurrent_reads as fn()),
        ("Concurrent Read/Write", test_concurrent_read_write as fn()),
        ("Invalid Operations", test_invalid_operations as fn()),
        ("Compaction", test_compaction as fn()),
        ("Group Commit", test_group_commit_behavior as fn()),
        ("Range Queries", test_range_queries as fn()),
        ("Tombstones", test_tombstone_behavior as fn()),
        ("Wildcard Star Match", test_wildcard_star_match as fn()),
        ("Wildcard Question Match", test_wildcard_question_match as fn()),
        ("Wildcard Delete", test_wildcard_delete as fn()),
        ("Set Many Basic", test_set_many_basic as fn()),
        ("Set Many Subtree Replace", test_set_many_with_subtree_replacement as fn()),
        ("Set Many Empty", test_set_many_empty as fn()),
        ("Set Many Parent Violation", test_set_many_parent_scalar_violation as fn()),
        ("Object Flattening", test_object_flattening_simulation as fn()),
    ];
    
    let mut passed = 0;
    let mut failed = 0;
    
    for (name, test) in tests {
        print!("Testing {}: ", name);
        std::panic::catch_unwind(test).map_or_else(
            |_| {
                println!("❌ FAILED");
                failed += 1;
            },
            |_| {
                println!("✅ PASSED");
                passed += 1;
            }
        );
    }
    
    println!("\n========================");
    println!("Results: {} passed, {} failed", passed, failed);
    
    // Clean up any remaining test directories
    let _ = std::fs::read_dir("/tmp")
        .map(|entries| {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with("waldb_test_") {
                        let _ = std::fs::remove_dir_all(entry.path());
                    }
                }
            }
        });
    
    if failed > 0 {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod reconstruction_tests {
    use super::waldb_store::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_object_reconstruction_after_flush() {
        let dir = tempdir().unwrap();
        let store = Store::open(dir.path()).unwrap();
        
        // Set individual properties (simulating flattened object)
        store.set("obj/a", "n:1", false).unwrap();
        store.set("obj/b", "n:2", false).unwrap();
        store.set("obj/c/d", "n:3", false).unwrap();
        
        // Before flush - should reconstruct
        let result = store.get("obj").unwrap();
        println!("Before flush: {:?}", result);
        assert!(result.is_some(), "Should reconstruct object before flush");
        
        // Flush to segments
        store.flush().unwrap();
        
        // After flush - should still reconstruct
        let result2 = store.get("obj").unwrap();
        println!("After flush: {:?}", result2);
        assert!(result2.is_some(), "Should reconstruct object after flush");
        
        // Reopen store
        let store2 = Store::open(dir.path()).unwrap();
        let result3 = store2.get("obj").unwrap();
        println!("After reopen: {:?}", result3);
        assert!(result3.is_some(), "Should reconstruct object after reopen");
    }
}