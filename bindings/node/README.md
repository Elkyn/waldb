# WalDB Node.js Bindings

Node.js bindings for WalDB using the Neon framework, providing a Firebase Realtime Database-like API.

## Installation

```bash
npm install @elkyn/waldb
```

## Usage

### Basic Operations

```javascript
const WalDB = require('@elkyn/waldb');

// Open database
const db = WalDB.open('./my_database');

// Set values
db.set('users/alice/name', 'Alice Smith');
db.set('users/alice/profile', {
  age: 30,
  city: 'New York',
  interests: ['coding', 'music']
});

// Get values
const name = db.get('users/alice/name');        // 'Alice Smith'
const profile = db.get('users/alice/profile');  // { age: 30, city: 'New York', ... }
const users = db.get('users/');                 // Full users subtree

// Check existence
if (db.exists('users/alice/email')) {
  console.log('Alice has an email');
}

// Delete
db.delete('users/alice/temp_data');
```

### Firebase-style Reference API

```javascript
// Create references
const usersRef = db.ref('users');
const aliceRef = usersRef.child('alice');

// Use references
aliceRef.set({ name: 'Alice Updated', age: 31 });
const aliceData = aliceRef.get();

// Check reference
if (aliceRef.exists()) {
  console.log('Alice exists at:', aliceRef.toString());
}

// Remove via reference
aliceRef.remove();
```

### Pattern Matching

```javascript
// Find all user names
const names = db.getPattern('users/*/name');
// Returns: { 'users/alice/name': 'Alice', 'users/bob/name': 'Bob' }

// Find all settings
const settings = db.getPattern('users/*/settings/*');
```

### Range Queries

```javascript
// Get all users between alice and charlie
const someUsers = db.getRange('users/alice', 'users/charlie');

// Get all products in category A
const productA = db.getRange('products/A000', 'products/A999');
```

### Advanced Features

```javascript
// List all keys with prefix
const userKeys = db.listKeys('users/');
console.log(userKeys); // ['users/alice', 'users/bob', ...]

// Force overwrite parent nodes
db.set('data/path/child', 'child value');
// This would normally fail:
// db.set('data/path', 'parent value');
// But with force it works:
db.set('data/path', 'parent value', true);

// Manual flush to disk
db.flush();
```

## API Reference

### WalDB Class

#### Static Methods

- `WalDB.open(path)` - Open or create a database at the given path

#### Instance Methods

- `set(path, value, force?)` - Set a value at path
- `get(path)` - Get value or subtree at path
- `delete(path)` - Delete path and all children
- `exists(path)` - Check if path exists
- `getPattern(pattern)` - Get all values matching pattern (* and ? wildcards)
- `getRange(start, end)` - Get all values in range [start, end)
- `listKeys(prefix)` - List all keys with given prefix
- `flush()` - Force flush to disk
- `ref(path?)` - Create a reference to the given path

### Reference Class

- `child(path)` - Get child reference
- `set(value, force?)` - Set value at this reference
- `get()` - Get value at this reference
- `remove()` - Delete this reference
- `exists()` - Check if this reference exists
- `toString()` - Get the path of this reference

## Performance

The Node.js bindings provide excellent performance through native Rust implementation:

- **Writes**: 10,000+ operations/sec
- **Reads**: 100,000+ operations/sec
- **Memory**: Efficient with built-in LRU cache
- **Storage**: LSM tree with automatic compaction

## Building from Source

```bash
# Install dependencies
npm install

# Build native module
npm run build-release

# Run tests
npm test
```

## Error Handling

All operations that can fail will throw JavaScript errors:

```javascript
try {
  db.set('invalid/path', 'value');
} catch (error) {
  console.error('Database error:', error.message);
}
```

## Tree Semantics

WalDB enforces Firebase RTDB tree rules:

```javascript
// ✅ Valid - creates intermediate nodes
db.set('a/b/c', 'value');

// ❌ Invalid - can't overwrite parent with scalar
db.set('a/b', 'scalar'); // Throws error

// ✅ Valid - force overwrite destroys subtree  
db.set('a/b', 'scalar', true);
```

## License

MIT License - see [LICENSE](../../LICENSE) for details.