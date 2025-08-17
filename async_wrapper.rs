// Async wrapper for WalDB Store
// Provides non-blocking operations using tokio::task::spawn_blocking

use std::sync::Arc;
use std::io;
use tokio::task;

// Include the sync implementation with module isolation
mod sync_store {
    include!("waldb.rs");
}

use sync_store::Store;
use std::path::{Path, PathBuf};

/// Async wrapper around the synchronous Store
/// Uses tokio::task::spawn_blocking to run sync operations in a thread pool
pub struct AsyncStore {
    inner: Arc<Store>,
    _path: PathBuf,
}

impl AsyncStore {
    /// Open or create a store at the given path
    pub async fn open(path: &Path) -> io::Result<Self> {
        let path_buf = path.to_path_buf();
        let store = task::spawn_blocking(move || {
            Store::open(&path_buf)
        }).await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))??;
        
        Ok(AsyncStore {
            inner: Arc::new(store),
            _path: path.to_path_buf(),
        })
    }
    
    /// Set a key-value pair asynchronously
    pub async fn set(&self, key: &str, value: &str, replace_subtree: bool) -> io::Result<()> {
        let store = self.inner.clone();
        let key = key.to_string();
        let value = value.to_string();
        
        task::spawn_blocking(move || {
            store.set(&key, &value, replace_subtree)
        }).await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
    }
    
    /// Get a value by key asynchronously
    pub async fn get(&self, key: &str) -> io::Result<Option<String>> {
        let store = self.inner.clone();
        let key = key.to_string();
        
        task::spawn_blocking(move || {
            store.get(&key)
        }).await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
    }
    
    /// Delete a key asynchronously
    pub async fn delete(&self, key: &str) -> io::Result<()> {
        let store = self.inner.clone();
        let key = key.to_string();
        
        task::spawn_blocking(move || {
            store.delete(&key)
        }).await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
    }
    
    /// Delete an entire subtree asynchronously
    pub async fn delete_subtree(&self, prefix: &str) -> io::Result<()> {
        let store = self.inner.clone();
        let prefix = prefix.to_string();
        
        task::spawn_blocking(move || {
            store.delete_subtree(&prefix)
        }).await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
    }
    
    /// Flush memtable to disk asynchronously
    pub async fn flush(&self) -> io::Result<()> {
        let store = self.inner.clone();
        
        task::spawn_blocking(move || {
            store.flush()
        }).await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
    }
    
    /// Get all key-value pairs matching a wildcard pattern asynchronously
    pub async fn get_pattern(&self, pattern: &str) -> io::Result<Vec<(String, String)>> {
        let store = self.inner.clone();
        let pattern = pattern.to_string();
        
        task::spawn_blocking(move || {
            store.get_pattern(&pattern)
        }).await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
    }
    
    /// Delete all keys matching a wildcard pattern asynchronously
    pub async fn delete_pattern(&self, pattern: &str) -> io::Result<usize> {
        let store = self.inner.clone();
        let pattern = pattern.to_string();
        
        task::spawn_blocking(move || {
            store.delete_pattern(&pattern)
        }).await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
    }
    
    /// Get a range of key-value pairs asynchronously
    pub async fn get_range(&self, start: &str, end: &str) -> io::Result<Vec<(String, String)>> {
        let store = self.inner.clone();
        let start = start.to_string();
        let end = end.to_string();
        
        task::spawn_blocking(move || {
            store.get_range(&start, &end)
        }).await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
    }
    
    /// Get a limited range of key-value pairs asynchronously
    pub async fn get_range_limit(&self, start: &str, end: &str, limit: usize) -> io::Result<Vec<(String, String)>> {
        let store = self.inner.clone();
        let start = start.to_string();
        let end = end.to_string();
        
        task::spawn_blocking(move || {
            store.get_range_limit(&start, &end, limit)
        }).await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
    }
    
    /// Scan keys with a given prefix asynchronously
    pub async fn scan_prefix(&self, prefix: &str, limit: usize) -> io::Result<Vec<(String, String)>> {
        let store = self.inner.clone();
        let prefix = prefix.to_string();
        
        task::spawn_blocking(move || {
            store.scan_prefix(&prefix, limit)
        }).await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
    }
    
    /// Get segment counts for monitoring
    pub async fn segment_counts(&self) -> (usize, usize, usize) {
        let store = self.inner.clone();
        
        task::spawn_blocking(move || {
            store.segment_counts()
        }).await
        .unwrap_or((0, 0, 0))
    }
    
    /// Batch set operation - multiple sets in a single async call
    pub async fn batch_set(&self, operations: Vec<(String, String, bool)>) -> io::Result<()> {
        let store = self.inner.clone();
        
        task::spawn_blocking(move || {
            for (key, value, replace) in operations {
                store.set(&key, &value, replace)?;
            }
            Ok(())
        }).await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
    }
    
    /// Batch get operation - multiple gets in a single async call
    pub async fn batch_get(&self, keys: Vec<String>) -> io::Result<Vec<Option<String>>> {
        let store = self.inner.clone();
        
        task::spawn_blocking(move || {
            let mut results = Vec::new();
            for key in keys {
                results.push(store.get(&key)?);
            }
            Ok(results)
        }).await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_async_basic_operations() {
        let dir = tempdir().unwrap();
        let store = AsyncStore::open(dir.path()).await.unwrap();
        
        // Test set and get
        store.set("key1", "value1", false).await.unwrap();
        assert_eq!(store.get("key1").await.unwrap(), Some("value1".to_string()));
        
        // Test delete
        store.delete("key1").await.unwrap();
        assert_eq!(store.get("key1").await.unwrap(), None);
        
        // Test nonexistent key
        assert_eq!(store.get("nonexistent").await.unwrap(), None);
    }
    
    #[tokio::test]
    async fn test_async_concurrent_operations() {
        let dir = tempdir().unwrap();
        let store = Arc::new(AsyncStore::open(dir.path()).await.unwrap());
        
        let mut handles = vec![];
        
        // Spawn 10 concurrent tasks, each doing 100 operations
        for i in 0..10 {
            let store = store.clone();
            let handle = tokio::spawn(async move {
                for j in 0..100 {
                    let key = format!("task_{}_key_{}", i, j);
                    let value = format!("task_{}_value_{}", i, j);
                    store.set(&key, &value, false).await.unwrap();
                    let result = store.get(&key).await.unwrap();
                    assert_eq!(result, Some(value));
                }
            });
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Verify all data is present
        for i in 0..10 {
            for j in 0..100 {
                let key = format!("task_{}_key_{}", i, j);
                let value = format!("task_{}_value_{}", i, j);
                assert_eq!(store.get(&key).await.unwrap(), Some(value));
            }
        }
    }
    
    #[tokio::test]
    async fn test_async_batch_operations() {
        let dir = tempdir().unwrap();
        let store = AsyncStore::open(dir.path()).await.unwrap();
        
        // Batch set
        let operations = vec![
            ("batch1".to_string(), "value1".to_string(), false),
            ("batch2".to_string(), "value2".to_string(), false),
            ("batch3".to_string(), "value3".to_string(), false),
        ];
        store.batch_set(operations).await.unwrap();
        
        // Batch get
        let keys = vec!["batch1".to_string(), "batch2".to_string(), "batch3".to_string(), "missing".to_string()];
        let results = store.batch_get(keys).await.unwrap();
        
        assert_eq!(results[0], Some("value1".to_string()));
        assert_eq!(results[1], Some("value2".to_string()));
        assert_eq!(results[2], Some("value3".to_string()));
        assert_eq!(results[3], None);
    }
    
    #[tokio::test]
    async fn test_async_pattern_matching() {
        let dir = tempdir().unwrap();
        let store = AsyncStore::open(dir.path()).await.unwrap();
        
        // Set up test data
        store.set("users/alice/name", "Alice", false).await.unwrap();
        store.set("users/bob/name", "Bob", false).await.unwrap();
        store.set("users/charlie/name", "Charlie", false).await.unwrap();
        
        // Test pattern matching
        let results = store.get_pattern("users/*/name").await.unwrap();
        assert_eq!(results.len(), 3);
        
        // Delete pattern
        let count = store.delete_pattern("users/*/name").await.unwrap();
        assert_eq!(count, 3);
        
        // Verify deletion
        assert_eq!(store.get("users/alice/name").await.unwrap(), None);
        assert_eq!(store.get("users/bob/name").await.unwrap(), None);
        assert_eq!(store.get("users/charlie/name").await.unwrap(), None);
    }
    
    #[tokio::test]
    async fn test_async_range_queries() {
        let dir = tempdir().unwrap();
        let store = AsyncStore::open(dir.path()).await.unwrap();
        
        // Set up test data
        for i in 0..20 {
            let key = format!("key{:02}", i);
            let value = format!("value{}", i);
            store.set(&key, &value, false).await.unwrap();
        }
        store.flush().await.unwrap();
        
        // Test range query
        let results = store.get_range("key05", "key15").await.unwrap();
        assert_eq!(results.len(), 10);
        assert_eq!(results[0].0, "key05");
        assert_eq!(results[9].0, "key14");
        
        // Test limited range
        let limited = store.get_range_limit("key00", "key20", 5).await.unwrap();
        assert_eq!(limited.len(), 5);
        
        // Test prefix scan
        let prefix_results = store.scan_prefix("key1", 10).await.unwrap();
        assert_eq!(prefix_results.len(), 10); // key10-key19
    }
    
    #[tokio::test]
    async fn test_async_performance() {
        use std::time::Instant;
        
        let dir = tempdir().unwrap();
        let store = Arc::new(AsyncStore::open(dir.path()).await.unwrap());
        
        // Measure concurrent write performance
        let start = Instant::now();
        let mut handles = vec![];
        
        for i in 0..10 {
            let store = store.clone();
            let handle = tokio::spawn(async move {
                for j in 0..100 {
                    let key = format!("perf_{}_{}", i, j);
                    store.set(&key, "data", false).await.unwrap();
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.await.unwrap();
        }
        
        let elapsed = start.elapsed();
        let ops_per_sec = 1000.0 / elapsed.as_secs_f64();
        println!("Async writes: 1000 ops in {:?} = {:.0} ops/sec", elapsed, ops_per_sec);
        
        // Measure concurrent read performance
        let start = Instant::now();
        let mut handles = vec![];
        
        for i in 0..10 {
            let store = store.clone();
            let handle = tokio::spawn(async move {
                for j in 0..100 {
                    let key = format!("perf_{}_{}", i, j);
                    store.get(&key).await.unwrap();
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.await.unwrap();
        }
        
        let elapsed = start.elapsed();
        let ops_per_sec = 1000.0 / elapsed.as_secs_f64();
        println!("Async reads: 1000 ops in {:?} = {:.0} ops/sec", elapsed, ops_per_sec);
    }
}