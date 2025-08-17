/**
 * WalDB - Async-first Node.js bindings
 * High-performance write-ahead log database with Firebase-like tree semantics
 */

const native = require('./index.node');

class WalDB {
    constructor(store) {
        this._store = store;  // Native store handle
    }
    
    /**
     * Open a database (async)
     * @param {string} path - Path to the database directory
     * @returns {Promise<WalDB>} Database instance
     */
    static async open(path) {
        // Real async from Rust - returns a boxed store
        const store = await native.open(path);
        return new WalDB(store);
    }
    
    /**
     * Set a value at the given path (async)
     * @param {string} key - The path to set
     * @param {any} value - The value to set (objects will be flattened)
     * @param {boolean} [force=false] - Whether to force overwrite parent nodes
     */
    async set(key, value, force = false) {
        if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
            // Flatten object into multiple key-value pairs
            const flattened = this._flattenObject(key, value);
            const replaceAt = key === '' ? null : key;
            return native.setMany(this._store, flattened, replaceAt);
        } else {
            // Encode primitives and arrays with type prefixes
            const encodedValue = this._encodeValue(value);
            return native.set(this._store, key, encodedValue, force);
        }
    }
    
    /**
     * Get a value or subtree at the given path (async)
     * @param {string} key - The path to get
     * @returns {Promise<any>} The value or null if not found
     */
    async get(key) {
        const result = await native.get(this._store, key);
        
        // If result is a string, try to decode it
        if (typeof result === 'string') {
            // Check if it's JSON (reconstructed object from Rust)
            if (result.startsWith('{') || result.startsWith('[')) {
                try {
                    const parsed = JSON.parse(result);
                    return this._decodeObject(parsed);
                } catch(e) {
                    // Not JSON, decode as single value
                    return WalDB._decodeValue(result);
                }
            }
            return WalDB._decodeValue(result);
        }
        
        return result;
    }
    
    /**
     * Delete a path and all its children (async)
     * @param {string} key - The path to delete
     */
    async delete(key) {
        return native.delete(this._store, key);
    }
    
    /**
     * Flush memtable to disk (async)
     */
    async flush() {
        return native.flush(this._store);
    }
    
    /**
     * Get all values matching a pattern (async)
     * @param {string} pattern - Pattern with * and ? wildcards
     * @returns {Promise<Object>} Object with matching key-value pairs
     */
    async getPattern(pattern) {
        const results = await native.getPattern(this._store, pattern);
        if (results && typeof results === 'object') {
            return this._decodeObject(results);
        }
        return results;
    }
    
    /**
     * Get all values in a range (async)
     * @param {string} start - Start key (inclusive)
     * @param {string} end - End key (exclusive)
     * @returns {Promise<Object>} Object with matching key-value pairs
     */
    async getRange(start, end) {
        const results = await native.getRange(this._store, start, end);
        if (results && typeof results === 'object') {
            return this._decodeObject(results);
        }
        return results;
    }
    
    /**
     * Check if a key exists (async)
     * @param {string} key - The path to check
     * @returns {Promise<boolean>} True if the key exists
     */
    async exists(key) {
        const value = await this.get(key);
        return value !== null && value !== undefined;
    }
    
    /**
     * Create a reference to a path (Firebase-style API)
     * @param {string} path - The path to reference
     * @returns {Reference} A reference object
     */
    ref(path) {
        return new Reference(this, path);
    }
    
    
    // Private helper methods
    
    _flattenObject(basePath, obj, result = {}) {
        for (const [key, value] of Object.entries(obj)) {
            const fullPath = basePath ? `${basePath}/${key}` : key;
            
            if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
                this._flattenObject(fullPath, value, result);
            } else {
                result[fullPath] = this._encodeValue(value);
            }
        }
        return result;
    }
    
    _encodeValue(value) {
        if (value === null) {
            return 'z:null';
        } else if (typeof value === 'string') {
            return 's:' + value;
        } else if (typeof value === 'number') {
            return 'n:' + value;
        } else if (typeof value === 'boolean') {
            return 'b:' + value;
        } else if (Array.isArray(value)) {
            return 'a:' + JSON.stringify(value);
        } else if (typeof value === 'object') {
            return 's:' + JSON.stringify(value);
        }
    }
    
    static _decodeValue(encoded) {
        if (!encoded || typeof encoded !== 'string') {
            return encoded;
        }
        
        const colonIndex = encoded.indexOf(':');
        if (colonIndex === -1 || colonIndex === 0) {
            return encoded;
        }
        
        const type = encoded[0];
        const value = encoded.substring(2);
        
        switch (type) {
            case 's': return value;
            case 'n': return Number(value);
            case 'b': return value === 'true';
            case 'a':
                try {
                    return JSON.parse(value);
                } catch {
                    return value;
                }
            case 'z': return null;
            default: return encoded;
        }
    }
    
    _decodeObject(obj) {
        if (Array.isArray(obj)) {
            return obj.map(item => 
                typeof item === 'object' && item !== null ? this._decodeObject(item) : WalDB._decodeValue(item)
            );
        } else if (typeof obj === 'object' && obj !== null) {
            const decoded = {};
            for (const [key, value] of Object.entries(obj)) {
                if (typeof value === 'object' && value !== null) {
                    decoded[key] = this._decodeObject(value);
                } else {
                    decoded[key] = WalDB._decodeValue(value);
                }
            }
            return decoded;
        }
        return obj;
    }
}

/**
 * Firebase-style reference API
 */
class Reference {
    constructor(db, path) {
        this._db = db;
        this._path = path;
    }
    
    async set(value) {
        return this._db.set(this._path, value);
    }
    
    async get() {
        return this._db.get(this._path);
    }
    
    async remove() {
        return this._db.delete(this._path);
    }
    
    child(path) {
        const newPath = this._path ? `${this._path}/${path}` : path;
        return new Reference(this._db, newPath);
    }
    
    parent() {
        const idx = this._path.lastIndexOf('/');
        if (idx > 0) {
            return new Reference(this._db, this._path.substring(0, idx));
        }
        return new Reference(this._db, '');
    }
}

module.exports = WalDB;
module.exports.WalDB = WalDB;
module.exports.Reference = Reference;