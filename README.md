# 🗄️ WalDB

> High-performance write-ahead log database with tree semantics

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Performance](https://img.shields.io/badge/writes-17k%2Fsec-green.svg)]()
[![Performance](https://img.shields.io/badge/reads-6M%2Fsec-green.svg)]()

WalDB is a blazingly fast embedded database with Firebase Realtime Database tree semantics. Built for applications requiring hierarchical data with path-based operations and real-time updates.

## ✨ Features

- 🚀 **Extreme Performance** - 12,000+ writes/sec, 40,000+ reads/sec (Node.js)
- 🌲 **Tree Structure** - Native hierarchical data like Firebase RTDB
- 🔍 **Pattern Matching** - Wildcards (`*`, `?`) for flexible queries
- 📚 **Range Queries** - Efficient pagination and scanning
- 💾 **LSM Tree Architecture** - Log-structured merge tree with compaction
- 🔄 **Thread-Safe** - RwLock protection, no async complexity
- 🎯 **Minimal Dependencies** - Pure Rust core, simple FFI
- 💪 **ACID Properties** - Atomic writes, crash recovery via WAL
- 📦 **Clean Architecture** - Separated core, FFI, and language APIs

## 🚀 Quick Start

### Rust

```rust
use waldb::Store;
use std::path::Path;

// Open or create a store
let store = Store::open(Path::new("./my_data"))?;

// Set values
store.set("users/alice/name", "Alice Smith", false)?;
store.set("users/alice/age", "30", false)?;

// Get values  
let name = store.get("users/alice/name")?; // Some("Alice Smith")

// Pattern matching
let names = store.get_pattern("users/*/name")?; // All user names

// Range queries
let range = store.get_range("users/alice", "users/bob")?;
```

### Node.js

```javascript
const WalDB = require('@elkyn/waldb');

// Open database (async)
const db = await WalDB.open('./my_data');

// Firebase-like API
await db.set('users/alice/name', 'Alice Smith');
await db.set('users/alice/age', 30);  // Types preserved!

// Three ways to read data
const entries = await db.get('users/alice');      // [[key, value], ...]
const raw = await db.getRaw('users/alice');       // With type prefixes
const obj = await db.getObject('users/alice');    // Reconstructed object

// Pattern matching
const names = await db.getPattern('users/*/name');

// Range queries
const range = await db.getRange('users/alice', 'users/bob');
```

### CLI

```bash
# Install CLI
cargo install waldb

# Interactive shell
waldb

waldb> set users/alice/name "Alice Smith"
waldb> get users/
{
  "alice": {
    "name": "Alice Smith"
  }
}
waldb> pattern users/*/name
Found 1 matches:
  users/alice/name = Alice Smith
```

## 🏗️ Architecture

WalDB uses a sophisticated LSM (Log-Structured Merge) tree architecture:

```
┌──────────────┐
│  Write Path  │
└──────┬───────┘
       ▼
┌──────────────┐
│     WAL      │ ← Write-Ahead Log (Durability)
└──────┬───────┘
       ▼
┌──────────────┐
│   MemTable   │ ← In-memory balanced tree
└──────┬───────┘
       ▼
┌──────────────┐
│   L0 SSTs    │ ← Immutable sorted files
└──────┬───────┘
       ▼
┌──────────────┐
│   L1 SSTs    │ ← Compacted, non-overlapping
└──────┬───────┘
       ▼
┌──────────────┐
│   L2 SSTs    │ ← Further compacted
└──────────────┘
```

### Key Components

- **WAL (Write-Ahead Log)**: Ensures durability, supports group commit
- **MemTable**: In-memory sorted structure for recent writes
- **SST Files**: Immutable sorted string tables with bloom filters
- **Compaction**: Background process merging and organizing data
- **Block Cache**: LRU cache for frequently accessed blocks

## 🎯 Architecture

WalDB uses a clean three-layer architecture:

### Core Layer (Rust)
- Handles raw key-value storage with tree semantics
- LSM-tree implementation with WAL for durability
- Thread-safe with RwLock protection
- No JSON reconstruction in core - just raw entries

### FFI Layer
- Minimal bridge between core and language bindings
- Passes only string pairs - no complex types
- Uses std::thread for async operations (no tokio needed)

### Language API Layer
- Each language gets its own idiomatic API
- JavaScript handles type encoding/decoding
- Provides convenience methods like `getObject()`

## 📊 Performance

| Operation | Node.js (via FFI) | Raw Rust | Notes |
|-----------|------------------|----------|-------|
| Write | 12,000/sec | 13,000/sec | Similar due to I/O bound |
| Read | 40,000/sec | 270,000/sec | 6.7x faster without FFI |
| Pattern Match | 1000 keys in ~3ms | 10,000 keys in ~6ms | Scales linearly |
| Range Query | - | 10,000 keys in ~4ms | Very efficient |

## 🌲 Tree Semantics

WalDB enforces Firebase RTDB tree rules:

```rust
// ✅ Valid - Creates intermediate nodes
store.set("a/b/c", "value", false)?;
// Tree: {"a": {"b": {"c": "value"}}}

// ❌ Invalid - Can't overwrite parent with scalar
store.set("a/b", "scalar", false)?; // Error: would destroy children

// ✅ Valid - Force overwrite destroys subtree
store.set("a/b", "scalar", true)?; // Replaces entire subtree
// Tree: {"a": {"b": "scalar"}}

// ✅ Deleting removes entire subtree
store.delete("a")?;
// Tree: {}
```

## 📚 API Reference

### Core Operations

```rust
// Open a store
let store = Store::open(path)?;

// Write operations
store.set(key, value, force)?;    // Set a value
store.delete(key)?;                // Delete key and subtree
store.flush()?;                    // Force WAL flush

// Read operations  
store.get(key)?;                   // Get raw value (no JSON reconstruction)
store.exists(key)?;               // Check if key exists

// Advanced queries
store.get_pattern(pattern)?;      // Pattern matching with * and ?
store.get_range(start, end)?;     // Range scan
store.list_keys(prefix)?;         // List all keys with prefix

// Metrics
let metrics = store.get_metrics();
println!("Writes: {}", metrics.total_writes());
println!("Cache hit rate: {:.2}%", metrics.cache_hit_rate() * 100.0);
```

## 🔧 Installation

### Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
waldb = "0.1"
```

### Building from Source

```bash
git clone https://github.com/elkyn/waldb
cd waldb
cargo build --release
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run benchmarks
cargo bench

# Run with logging
RUST_LOG=debug cargo test
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- Inspired by LevelDB/RocksDB architecture
- Tree semantics modeled after Firebase Realtime Database
- Built with love in Rust 🦀