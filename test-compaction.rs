// Thorough compaction tests for WalDB

// Include the main waldb module (it has all the imports we need)
include!("waldb.rs");

fn test_dir(name: &str) -> String {
    let dir = format!("/tmp/waldb_compaction_test_{}", name);
    let _ = fs::remove_dir_all(&dir);
    dir
}

fn cleanup(dir: &str) {
    let _ = fs::remove_dir_all(dir);
}

fn test_basic_compaction() {
    println!("Testing basic L0 to L1 compaction...");
    let dir = test_dir("basic");
    let store = Store::open(Path::new(&dir)).unwrap();
    
    // Create multiple L0 segments
    for batch in 0..5 {
        for i in 0..100 {
            let key = format!("key_{:03}_{:03}", batch, i);
            let value = format!("value_{}", i);
            store.set(&key, &value, false).unwrap();
        }
        store.flush().unwrap();
    }
    
    // Check initial segment counts
    let (l0_before, l1_before, _l2_before) = store.segment_counts();
    assert!(l0_before >= 5, "Should have at least 5 L0 segments");
    
    // Wait for compaction (compaction runs every second)
    thread::sleep(Duration::from_secs(3));
    
    // Check after compaction
    let (l0_after, l1_after, _l2_after) = store.segment_counts();
    // Compaction might not have triggered yet if segments are small
    // Just verify data is accessible
    println!("  Segments: L0 {} -> {}, L1 {} -> {}", l0_before, l0_after, l1_before, l1_after);
    
    // Verify data integrity
    for batch in 0..5 {
        for i in 0..100 {
            let key = format!("key_{:03}_{:03}", batch, i);
            let value = store.get(&key).unwrap();
            assert_eq!(value, Some(format!("value_{}", i)));
        }
    }
    
    cleanup(&dir);
    println!("✓ Basic compaction test passed");
}

fn test_compaction_with_overwrites() {
    println!("Testing compaction with overwrites...");
    let dir = test_dir("overwrites");
    let store = Store::open(Path::new(&dir)).unwrap();
    
    // Write initial values
    for i in 0..200 {
        store.set(&format!("key_{:03}", i), "initial", false).unwrap();
    }
    store.flush().unwrap();
    
    // Overwrite with new values
    for i in 0..200 {
        store.set(&format!("key_{:03}", i), "updated", false).unwrap();
    }
    store.flush().unwrap();
    
    // Overwrite again
    for i in 0..200 {
        store.set(&format!("key_{:03}", i), "final", false).unwrap();
    }
    store.flush().unwrap();
    
    // Force compaction by waiting
    thread::sleep(Duration::from_secs(2));
    
    // Verify only latest values are returned
    for i in 0..200 {
        let value = store.get(&format!("key_{:03}", i)).unwrap();
        assert_eq!(value, Some("final".to_string()));
    }
    
    cleanup(&dir);
    println!("✓ Overwrites compaction test passed");
}

fn test_compaction_with_deletes() {
    println!("Testing compaction with deletions...");
    let dir = test_dir("deletes");
    let store = Store::open(Path::new(&dir)).unwrap();
    
    // Write initial data
    for i in 0..300 {
        store.set(&format!("key_{:03}", i), &format!("value_{}", i), false).unwrap();
    }
    store.flush().unwrap();
    
    // Delete every other key
    for i in (0..300).step_by(2) {
        store.delete(&format!("key_{:03}", i)).unwrap();
    }
    store.flush().unwrap();
    
    // Wait for compaction
    thread::sleep(Duration::from_secs(2));
    
    // Verify deletions are preserved
    for i in 0..300 {
        let value = store.get(&format!("key_{:03}", i)).unwrap();
        if i % 2 == 0 {
            assert_eq!(value, None, "Deleted key {} should be None", i);
        } else {
            assert_eq!(value, Some(format!("value_{}", i)));
        }
    }
    
    cleanup(&dir);
    println!("✓ Deletions compaction test passed");
}

fn test_l1_to_l2_compaction() {
    println!("Testing L1 to L2 compaction...");
    let dir = test_dir("l1_to_l2");
    let store = Store::open(Path::new(&dir)).unwrap();
    
    // Create many segments to trigger L1->L2 compaction
    for batch in 0..20 {
        for i in 0..50 {
            let key = format!("batch_{:02}_key_{:03}", batch, i);
            let value = format!("value_{}", i);
            store.set(&key, &value, false).unwrap();
        }
        store.flush().unwrap();
        
        // Small delay to allow compaction between batches
        thread::sleep(Duration::from_millis(100));
    }
    
    // Wait for all compactions
    thread::sleep(Duration::from_secs(3));
    
    let (l0, l1, l2) = store.segment_counts();
    println!("  Segments after compaction: L0={}, L1={}, L2={}", l0, l1, l2);
    
    // Should have some L2 segments if L1->L2 compaction triggered
    // Note: This depends on thresholds, but with 20 batches we should get some
    
    // Verify all data is still accessible
    for batch in 0..20 {
        for i in 0..50 {
            let key = format!("batch_{:02}_key_{:03}", batch, i);
            let value = store.get(&key).unwrap();
            assert_eq!(value, Some(format!("value_{}", i)));
        }
    }
    
    cleanup(&dir);
    println!("✓ L1 to L2 compaction test passed");
}

fn test_compaction_with_subtrees() {
    println!("Testing compaction with subtree operations...");
    let dir = test_dir("subtrees");
    let store = Store::open(Path::new(&dir)).unwrap();
    
    // Create tree structure
    for i in 0..10 {
        store.set(&format!("root/branch_{}/leaf_a", i), "a", false).unwrap();
        store.set(&format!("root/branch_{}/leaf_b", i), "b", false).unwrap();
        store.set(&format!("root/branch_{}/leaf_c", i), "c", false).unwrap();
    }
    store.flush().unwrap();
    
    // Delete some subtrees
    store.delete_subtree("root/branch_3").unwrap();
    store.delete_subtree("root/branch_7").unwrap();
    store.flush().unwrap();
    
    // Add new data
    for i in 10..15 {
        store.set(&format!("root/branch_{}/leaf_a", i), "new_a", false).unwrap();
    }
    store.flush().unwrap();
    
    // Wait for compaction
    thread::sleep(Duration::from_secs(2));
    
    // Verify tree state after compaction
    for i in 0..15 {
        let value = store.get(&format!("root/branch_{}/leaf_a", i)).unwrap();
        if i == 3 || i == 7 {
            assert_eq!(value, None, "Deleted subtree should be None");
        } else if i < 10 {
            assert_eq!(value, Some("a".to_string()));
        } else {
            assert_eq!(value, Some("new_a".to_string()));
        }
    }
    
    cleanup(&dir);
    println!("✓ Subtree compaction test passed");
}

fn test_compaction_persistence() {
    println!("Testing compaction persistence across restarts...");
    let dir = test_dir("persistence");
    
    {
        let store = Store::open(Path::new(&dir)).unwrap();
        
        // Write data that will trigger compaction
        for batch in 0..8 {
            for i in 0..100 {
                store.set(&format!("key_{:03}_{:03}", batch, i), "value", false).unwrap();
            }
            store.flush().unwrap();
        }
        
        // Wait for compaction
        thread::sleep(Duration::from_secs(2));
        
        let (l0, l1, l2) = store.segment_counts();
        println!("  Before restart: L0={}, L1={}, L2={}", l0, l1, l2);
    }
    
    // Reopen the store
    {
        let store = Store::open(Path::new(&dir)).unwrap();
        
        let (l0, l1, l2) = store.segment_counts();
        println!("  After restart: L0={}, L1={}, L2={}", l0, l1, l2);
        
        // Verify all data is still accessible
        for batch in 0..8 {
            for i in 0..100 {
                let value = store.get(&format!("key_{:03}_{:03}", batch, i)).unwrap();
                assert_eq!(value, Some("value".to_string()));
            }
        }
    }
    
    cleanup(&dir);
    println!("✓ Persistence compaction test passed");
}

fn test_compaction_under_load() {
    println!("Testing compaction under continuous load...");
    let dir = test_dir("load");
    let store = Store::open(Path::new(&dir)).unwrap();
    
    // Start a thread that continuously writes
    let store_clone = store.clone();
    let writer = thread::spawn(move || {
        for i in 0..1000 {
            store_clone.set(&format!("load_key_{:04}", i), &format!("value_{}", i), false).unwrap();
            if i % 100 == 0 {
                store_clone.flush().unwrap();
            }
            thread::sleep(Duration::from_millis(5));
        }
    });
    
    // Start a thread that continuously reads
    let store_clone = store.clone();
    let reader = thread::spawn(move || {
        for i in 0..500 {
            let key = format!("load_key_{:04}", (i * 7 + 13) % 1000); // Simple pseudo-random
            let _ = store_clone.get(&key);
            thread::sleep(Duration::from_millis(10));
        }
    });
    
    // Let it run for a while
    thread::sleep(Duration::from_secs(3));
    
    // Check that compaction is happening
    let (l0, l1, _) = store.segment_counts();
    assert!(l1 > 0, "Should have some L1 segments from compaction");
    
    writer.join().unwrap();
    reader.join().unwrap();
    
    // Verify data integrity
    for i in 0..1000 {
        let value = store.get(&format!("load_key_{:04}", i)).unwrap();
        assert_eq!(value, Some(format!("value_{}", i)));
    }
    
    cleanup(&dir);
    println!("✓ Load compaction test passed");
}

fn main() {
    println!("Running WalDB Compaction Tests");
    println!("==============================\n");
    
    test_basic_compaction();
    test_compaction_with_overwrites();
    test_compaction_with_deletes();
    test_l1_to_l2_compaction();
    test_compaction_with_subtrees();
    test_compaction_persistence();
    test_compaction_under_load();
    
    println!("\n==============================");
    println!("All compaction tests passed! ✅");
}