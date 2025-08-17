# WalDB Node.js Bindings

High-performance async Node.js bindings for WalDB, providing a Firebase Realtime Database-like API with tree semantics.

## Installation

```bash
# Clone and build from source
git clone https://github.com/elkyn/waldb
cd waldb/bindings/node
npm install
npm run build-release

# Link for local development
npm link

# In your project
npm link waldb
```

## Features

- 🚀 **High Performance**: 12,000+ writes/sec, 40,000+ reads/sec
- 🌲 **Tree Semantics**: Firebase-like hierarchical data structure
- 🔄 **Async/Await**: All operations are async for non-blocking I/O
- 🎯 **Type Preservation**: Maintains JavaScript types (numbers, booleans, arrays, etc.)
- 📦 **Three-tier API**: Flexible data access patterns
- 📁 **File Storage**: Automatic compression and deduplication for blobs
- 🔍 **Advanced Search**: Filter queries with multiple conditions
- 🧮 **Vector Search**: Store and search embeddings with cosine similarity
- 📝 **Text Search**: Full-text search with tokenization and fuzzy matching
- 🎯 **Hybrid Search**: Combine vector, text, and filters with custom scoring weights

## Usage

### Basic Operations

```javascript
const WalDB = require('waldb');

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

### File Storage

```javascript
// Store files with automatic compression and deduplication
const imageData = fs.readFileSync('photo.jpg');
await db.setFile('users/alice/avatar', imageData);

// Retrieve files
const avatar = await db.getFile('users/alice/avatar');
fs.writeFileSync('retrieved.jpg', avatar);

// Get file metadata without loading the file
const meta = await db.getFileMeta('users/alice/avatar');
console.log(meta); // { size: 45632, type: 'image/jpeg', hash: '...' }

// Files are automatically deduplicated - same content = stored once
await db.setFile('users/bob/avatar', imageData); // Reuses existing blob
```

### Advanced Search

```javascript
// Search with filters
const results = await db.search({
  pattern: 'users/*',
  filters: [
    { field: 'age', op: '>', value: '18' },
    { field: 'role', op: '==', value: 'admin' }
  ],
  limit: 50
});
// Returns: [[user1_entries], [user2_entries], ...] grouped by user

// Search and get as objects
const admins = await db.searchObjects({
  pattern: 'users/*',
  filters: [
    { field: 'role', op: '==', value: 'admin' },
    { field: 'active', op: '==', value: 'true' }
  ]
});
// Returns: [{ name: 'Alice', role: 'admin', ... }, ...]

// Supported operators: ==, !=, >, <, >=, <=
// Numeric comparisons work automatically
```

### Vector Search

```javascript
// Store embeddings
await db.setVector('docs/doc1/embedding', [0.1, 0.2, 0.3, 0.4, 0.5]);
await db.setVector('docs/doc2/embedding', [0.2, 0.3, 0.4, 0.5, 0.6]);
await db.setVector('docs/doc3/embedding', [0.3, 0.4, 0.5, 0.6, 0.7]);

// Find similar documents
const similar = await db.advancedSearch({
  pattern: 'docs/*',
  vector: {
    query: [0.15, 0.25, 0.35, 0.45, 0.55],
    field: 'embedding',
    topK: 2
  },
  limit: 2
});

// Get embedding
const embedding = await db.getVector('docs/doc1/embedding');
console.log(embedding); // [0.1, 0.2, 0.3, 0.4, 0.5]
```

### Text Search

```javascript
// Store searchable text
await db.set('articles/1/title', 'Introduction to WalDB');
await db.set('articles/1/content', 'WalDB is a high-performance database...');
await db.set('articles/2/title', 'Advanced WalDB Features');
await db.set('articles/2/content', 'Learn about vector search and more...');

// Search text fields
const results = await db.advancedSearch({
  pattern: 'articles/*',
  text: {
    query: 'WalDB features',
    fields: ['title', 'content'],
    fuzzy: true,
    caseInsensitive: true
  }
});
```

### Hybrid Search

```javascript
// Combine vector similarity, text search, and filters
const products = await db.advancedSearch({
  pattern: 'products/*',
  
  // Text search
  text: {
    query: 'comfortable running shoes',
    fields: ['name', 'description'],
    fuzzy: true
  },
  
  // Vector similarity
  vector: {
    query: [0.8, 0.2, 0.1, ...],  // Query embedding
    field: 'embedding',
    topK: 10
  },
  
  // Filters
  filters: [
    { field: 'category', op: '==', value: 'footwear' },
    { field: 'price', op: '<', value: '150' },
    { field: 'in_stock', op: '==', value: 'true' }
  ],
  
  // Scoring weights
  scoring: {
    vector: 0.4,   // 40% weight to vector similarity
    text: 0.4,     // 40% weight to text relevance
    filter: 0.2    // 20% weight to filter matches
  },
  
  limit: 5
});

// Get as reconstructed objects with scores
const productObjects = await db.advancedSearchObjects({
  pattern: 'products/*',
  // ... same options as above
});
// Returns objects with _vector_score, _text_score, _total_score fields
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