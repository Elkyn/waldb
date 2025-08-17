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
- 🔍 **Advanced Search** - Filter queries with multiple conditions and operators
- 🧮 **Vector Search** - Cosine similarity search on embeddings
- 📝 **Text Search** - Tokenization, fuzzy matching, case-insensitive search
- 🎯 **Hybrid Search** - Combine vector, text, and filter signals with custom scoring
- 📁 **File Storage** - Automatic compression and deduplication for blobs
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
const WalDB = require('waldb');

// Open database (async)
const db = await WalDB.open('./my_data');

// Firebase-like API
await db.set('users/alice/name', 'Alice Smith');
await db.set('users/alice/age', 30);  // Types preserved!

// Three ways to read data
const entries = await db.get('users/alice');      // [[key, value], ...]
const raw = await db.getRaw('users/alice');       // With type prefixes
const obj = await db.getObject('users/alice');    // Reconstructed object

// File storage with deduplication
await db.setFile('avatars/alice.jpg', imageBuffer);
const image = await db.getFile('avatars/alice.jpg');

// Advanced search with filters
const admins = await db.search({
  pattern: 'users/*',
  filters: [
    { field: 'role', op: '==', value: 'admin' },
    { field: 'age', op: '>', value: '25' }
  ]
});

// Pattern matching
const names = await db.getPattern('users/*/name');

// Range queries
const range = await db.getRange('users/alice', 'users/bob');

// Vector search
await db.setVector('docs/1/embedding', [0.1, 0.2, 0.3]);
await db.setVector('docs/2/embedding', [0.4, 0.5, 0.6]);

const similar = await db.advancedSearch({
  pattern: 'docs/*',
  vector: {
    query: [0.15, 0.25, 0.35],
    field: 'embedding',
    topK: 5
  },
  limit: 2
});

// Hybrid search (vector + text + filters)
const results = await db.advancedSearch({
  pattern: 'products/*',
  text: {
    query: 'red shirt',
    fields: ['name', 'description'],
    fuzzy: true
  },
  vector: {
    query: [0.8, 0.2, 0.1],  // "red" color embedding
    field: 'color_embedding'
  },
  filters: [
    { field: 'category', op: '==', value: 'clothing' },
    { field: 'price', op: '<', value: '50' }
  ],
  scoring: {
    vector: 0.4,
    text: 0.4,
    filter: 0.2
  }
});
```

### CLI

```bash
# Build from source
git clone https://github.com/elkyn/waldb
cd waldb
cargo build --release --bin waldb-cli

# Run interactive shell
./target/release/waldb-cli ./my_data

waldb> set users/alice/name "Alice Smith"
OK
waldb> get users/alice/name
Alice Smith
waldb> pattern users/*/name
Found 1 matches:
  users/alice/name = Alice Smith
waldb> exit
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

### Building from Source

```bash
git clone https://github.com/elkyn/waldb
cd waldb

# Build Rust library and CLI
cargo build --release

# Build Node.js bindings (optional)
cd bindings/node
npm install
npm run build-release
```

### Node.js

```bash
# Clone and build locally
git clone https://github.com/elkyn/waldb
cd waldb/bindings/node
npm install
npm run build-release

# Use in your project
npm link  # In waldb/bindings/node
npm link waldb  # In your project
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