# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

### Rust Core
```bash
# Build the library
cargo build --release

# Run Rust tests (custom test runner, not cargo test)
rustc --edition 2021 tests.rs -o /tmp/test_runner && /tmp/test_runner

# Run benchmarks
rustc --edition 2021 -O benchmarks.rs -o /tmp/bench_runner && /tmp/bench_runner

# Build CLI
cargo build --bin waldb-cli

# Quick test via Makefile
make test
make bench
```

### Node.js Bindings
```bash
cd bindings/node

# Build native module
npm run build-release

# Run all Node.js tests
npm test

# Run individual test suites
npm run test-fundamentals  # Basic operations
npm run test-types         # Type encoding/decoding
npm run test-complete      # Complex scenarios
npm run test-vector        # Vector/text search

# Development build
npm run build-debug
```

## Architecture Overview

WalDB is a monolithic Rust database (`waldb.rs`) with clean separation between core, FFI, and language bindings:

### Core Design Decisions
1. **No JSON reconstruction in core** - The Rust core returns flat key-value entries. Language bindings (like Node.js) handle object reconstruction.
2. **io::Error everywhere** - Simple error handling using standard `io::Result` instead of custom error types.
3. **Monolithic waldb.rs** - All core logic in a single file for easier navigation at this project size.
4. **include! in bindings** - Node.js FFI uses `include!("../../../waldb.rs")` to avoid dependency complexity.

### Key Components in waldb.rs

- **Store** - Main database interface with RwLock protection
- **StoreInner** - Protected state containing memtable, segments, and metadata
- **GroupCommitWAL** - Write-ahead log with batched commits for performance
- **Segment** - Immutable sorted string table with hash index
- **SegmentCache** - LRU block cache for segment reads
- **Manifest** - Tracks active segments for crash recovery

### Tree Semantics
- Cannot write under scalar parents (e.g., if `a/b` is a scalar, cannot set `a/b/c`)
- `replace_subtree` flag allows overwriting entire subtrees
- Delete operations remove entire subtrees atomically

### Performance Features
- Group commit batches WAL writes every 10ms
- Background compaction thread merges segments (L0→L1→L2)
- Block-level caching with 100MB default cache
- Hash indexes for O(1) segment lookups

### Node.js Integration
- Neon bindings in `bindings/node/src/lib.rs`
- JavaScript wrapper in `bindings/node/index.js` handles:
  - Type encoding (prefixes: `n:` for numbers, `b:` for booleans, etc.)
  - Object reconstruction from flat entries
  - Async/Promise wrapping of native calls

### Vector/Text Search
- Vector storage with `set_vector()`/`get_vector()`
- Cosine similarity search
- Text tokenization and fuzzy matching
- Hybrid scoring combining vector, text, and filter signals
- Exposed via `advancedSearch()` in Node.js

## Common Development Tasks

### Adding a New Core Method
1. Add method to `impl Store` in waldb.rs
2. Export in Node.js FFI (`bindings/node/src/lib.rs`)
3. Add TypeScript definitions (`bindings/node/index.d.ts`)
4. Wrap in JavaScript API (`bindings/node/index.js`)
5. Add tests to appropriate test file

### Running a Single Test
```bash
# Rust - modify tests.rs to run only specific test
rustc --edition 2021 tests.rs -o /tmp/test_runner && /tmp/test_runner

# Node.js - run specific test file
cd bindings/node
node test-fundamentals.js
```

### Debugging Compaction
Compaction runs in background thread, catches errors but continues. Check `compact_l0_to_l1()` and `compact_l1_to_l2()` in waldb.rs. Errors are suppressed to maintain availability.

### CI Workflow
The CI (`/.github/workflows/ci.yml`) compiles tests directly with `rustc --edition 2021` rather than using `cargo test`. This is intentional to match the custom test runner approach.

## Important Notes

- Test directories are created in `/tmp/waldb_test_*` and cleaned up automatically
- The `errors.rs` file was removed - all error handling uses `io::Error`
- Vector/text search structs are defined but not used directly from Rust tests (used via FFI)
- Lock poisoning uses `.expect()` which is standard practice - if a thread panics while holding a lock, subsequent acquisitions should fail
- The two non-test `unwrap()` calls are logically safe (checked with `is_none()`/`is_some()` first)