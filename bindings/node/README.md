# WalDB Node.js Bindings

High-performance async Node.js bindings for WalDB, providing a Firebase Realtime Database-like API with tree semantics.

## Installation

```bash
npm install @elkyn/waldb
```

## Features

- 🚀 **High Performance**: 12,000+ writes/sec, 40,000+ reads/sec
- 🌲 **Tree Semantics**: Firebase-like hierarchical data structure
- 🔄 **Async/Await**: All operations are async for non-blocking I/O
- 🎯 **Type Preservation**: Maintains JavaScript types (numbers, booleans, arrays, etc.)
- 📦 **Three-tier API**: Flexible data access patterns

## Usage

### Basic Operations

```javascript
const WalDB = require('@elkyn/waldb');

// Open database (async)
const db = await WalDB.open('./my_database');

// Set values (async)
await db.set('users/alice/name', 'Alice Smith');
await db.set('users/alice/profile', {
  age: 30,
  city: 'New York',
  interests: ['coding', 'music']
});

// Get values - three different methods
const entries = await db.get('users/alice');        // [[key, value], ...] array
const raw = await db.getRaw('users/alice');         // Raw entries with type prefixes
const obj = await db.getObject('users/alice');      // Reconstructed object

// Check existence
if (await db.exists('users/alice/email')) {
  console.log('Alice has an email');
}

// Delete
await db.delete('users/alice/temp_data');
```

### Three-tier API

WalDB provides three methods for reading data, each optimized for different use cases:

```javascript
// 1. get() - Returns decoded entries array
const entries = await db.get('users');
// Result: [['users/alice/name', 'Alice'], ['users/alice/age', 30], ...]

// 2. getRaw() - Returns raw entries with type prefixes
const raw = await db.getRaw('users');  
// Result: [['users/alice/name', 's:Alice'], ['users/alice/age', 'n:30'], ...]

// 3. getObject() - Returns reconstructed JavaScript object
const obj = await db.getObject('users');
// Result: { alice: { name: 'Alice', age: 30 }, ... }
```

### Firebase-style Reference API

```javascript
// Create references
const usersRef = db.ref('users');
const aliceRef = usersRef.child('alice');

// Use references (async)
await aliceRef.set({ name: 'Alice', age: 30 });
const data = await aliceRef.get();
await aliceRef.child('email').set('alice@example.com');
await aliceRef.remove();

// Navigate references
const parentRef = aliceRef.parent();  // users
```

### Pattern Matching & Range Queries

```javascript
// Pattern matching with wildcards
const results = await db.getPattern('users/*/email');
// Returns all user emails as object

const entries = await db.getPatternEntries('logs/2024-*');
// Returns all 2024 logs as entries array

// Range queries
const range = await db.getRange('users/a', 'users/d');
// Returns users starting with a, b, c

const rangeEntries = await db.getRangeEntries('events/2024-01', 'events/2024-02');
// Returns January events as entries array
```

### Type Preservation

WalDB automatically preserves JavaScript types:

```javascript
await db.set('config', {
  version: 1.5,           // Number preserved
  enabled: true,          // Boolean preserved
  name: 'MyApp',         // String preserved
  tags: ['v1', 'prod'],  // Array preserved
  metadata: null         // Null preserved
});

const config = await db.getObject('config');
console.log(typeof config.version);  // 'number'
console.log(typeof config.enabled);  // 'boolean'
console.log(Array.isArray(config.tags)); // true
```

### Advanced Features

```javascript
// Force overwrite (replaces parent nodes if needed)
await db.set('path/to/value', 'data', true);

// Flush to disk manually
await db.flush();

// Atomic subtree replacement
await db.set('users/alice', {
  name: 'Alice',
  profile: { age: 31, city: 'Boston' }
});
// Atomically replaces entire alice subtree
```

## Performance

- **Writes**: 12,000+ operations per second
- **Reads**: 40,000+ operations per second  
- **Pattern matching**: ~3ms for 1000 keys
- No async overhead - uses native threads efficiently

## Architecture

The bindings use a three-layer architecture:
- **Rust Core**: Handles raw storage with tree semantics
- **FFI Layer**: Minimal bridge passing string pairs
- **JavaScript API**: Rich type handling and convenience methods

No tokio, no unnecessary async complexity - just efficient std::thread usage with RwLock protection.

## License

MIT