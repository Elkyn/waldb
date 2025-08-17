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
        if (typeof value === 'object' && value !== null) {
            // Flatten objects AND arrays into multiple key-value pairs
            const flattened = this._flattenObject(key, value);
            const replaceAt = key === '' ? null : key;
            return native.setMany(this._store, flattened, replaceAt);
        } else {
            // Encode primitives only
            const encodedValue = this._encodeValue(value);
            return native.set(this._store, key, encodedValue, force);
        }
    }
    
    /**
     * Get entries with decoded values (default) (async)
     * @param {string} key - The path to get
     * @returns {Promise<Array<[string, any]>>} Array of [key, value] pairs with decoded values
     */
    async get(key) {
        const entries = await native.getEntries(this._store, key);
        // Decode values in the entries
        return entries.map(([k, v]) => [k, WalDB._decodeValue(v)]);
    }
    
    /**
     * Get raw entries with prefixed strings (async)
     * @param {string} key - The path to get
     * @returns {Promise<Array<[string, string]>>} Array of [key, value] pairs with raw prefixed values
     */
    async getRaw(key) {
        return native.getEntries(this._store, key);
    }
    
    /**
     * Get value or subtree as reconstructed object (async)
     * @param {string} key - The path to get
     * @returns {Promise<any>} The value or reconstructed object, null if not found
     */
    async getObject(key) {
        const entries = await this.get(key);
        
        if (entries.length === 0) {
            return null;
        }
        
        // If single entry with exact key match, return the value directly
        if (entries.length === 1 && entries[0][0] === key) {
            return entries[0][1];
        }
        
        // Otherwise reconstruct object from entries
        return this._reconstructFromEntries(entries, key);
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
     * Get all key-value pairs matching a pattern as entries array (async)
     * @param {string} pattern - Pattern with * and ? wildcards
     * @returns {Promise<Array<[string, any]>>} Array of [key, value] pairs
     */
    async getPatternEntries(pattern) {
        const entries = await native.getPatternEntries(this._store, pattern);
        // Decode values in the entries
        return entries.map(([key, value]) => [key, WalDB._decodeValue(value)]);
    }
    
    /**
     * Get all key-value pairs in a range as entries array (async)
     * @param {string} start - Start key (inclusive)
     * @param {string} end - End key (exclusive)
     * @returns {Promise<Array<[string, any]>>} Array of [key, value] pairs
     */
    async getRangeEntries(start, end) {
        const entries = await native.getRangeEntries(this._store, start, end);
        // Decode values in the entries
        return entries.map(([key, value]) => [key, WalDB._decodeValue(value)]);
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
    
    _reconstructFromEntries(entries, basePath) {
        const result = {};
        const baseLen = basePath ? basePath.length + 1 : 0;
        
        for (const [key, value] of entries) {
            // Remove base path to get relative path
            const relativePath = key.substring(baseLen);
            
            // Split path into parts
            const parts = relativePath.split('/');
            
            // Navigate through object structure
            let current = result;
            for (let i = 0; i < parts.length - 1; i++) {
                if (!current[parts[i]]) {
                    current[parts[i]] = {};
                }
                current = current[parts[i]];
            }
            
            // Set the value at the final key
            current[parts[parts.length - 1]] = value;
        }
        
        // Convert objects with numeric keys to arrays
        return this._convertNumericObjectsToArrays(result);
    }
    
    _convertNumericObjectsToArrays(obj) {
        if (typeof obj !== 'object' || obj === null) {
            return obj;
        }
        
        // Check if all keys are numeric
        const keys = Object.keys(obj);
        const isArray = keys.length > 0 && keys.every(k => /^\d+$/.test(k));
        
        if (isArray) {
            // Convert to array
            const arr = [];
            for (const key of keys) {
                const index = parseInt(key, 10);
                arr[index] = this._convertNumericObjectsToArrays(obj[key]);
            }
            return arr;
        } else {
            // Recursively process object properties
            const result = {};
            for (const [key, value] of Object.entries(obj)) {
                result[key] = this._convertNumericObjectsToArrays(value);
            }
            return result;
        }
    }
    
    _flattenObject(basePath, obj, result = {}) {
        // Handle arrays
        if (Array.isArray(obj)) {
            obj.forEach((value, index) => {
                const fullPath = basePath ? `${basePath}/${index}` : String(index);
                
                if (typeof value === 'object' && value !== null) {
                    this._flattenObject(fullPath, value, result);
                } else {
                    result[fullPath] = this._encodeValue(value);
                }
            });
            return result;
        }
        
        // Handle objects
        for (const [key, value] of Object.entries(obj)) {
            const fullPath = basePath ? `${basePath}/${key}` : key;
            
            if (typeof value === 'object' && value !== null) {
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
        } else if (typeof value === 'object') {
            // This should rarely happen now since objects/arrays are flattened
            // Only for edge cases or direct set of complex values
            return 's:' + JSON.stringify(value);
        }
        return 's:' + String(value);
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
    
    // ==================== FILE/BLOB SUPPORT ====================
    
    /**
     * Store a file with automatic compression and deduplication
     * @param {string} path - The path where to store the file
     * @param {Buffer|ArrayBuffer|Uint8Array} data - The file data
     */
    async setFile(path, data) {
        // Convert to Buffer if needed
        const buffer = Buffer.isBuffer(data) ? data : Buffer.from(data);
        return native.setFile(this._store, path, buffer);
    }
    
    /**
     * Retrieve a file from blob storage
     * @param {string} path - The path of the file
     * @returns {Promise<Buffer>} The file data
     */
    async getFile(path) {
        return native.getFile(this._store, path);
    }
    
    /**
     * Delete a file and its metadata
     * @param {string} path - The path of the file to delete
     */
    async deleteFile(path) {
        return native.deleteFile(this._store, path);
    }
    
    /**
     * Get file metadata without retrieving the file
     * @param {string} path - The path of the file
     * @returns {Promise<Object>} File metadata (size, type, hash)
     */
    async getFileMeta(path) {
        const [size, type, hash] = await Promise.all([
            this.getObject(`${path}:size`),
            this.getObject(`${path}:type`),
            this.getObject(`${path}:hash`)
        ]);
        
        return { size: Number(size), type, hash };
    }
    
    // ==================== SEARCH FUNCTIONALITY ====================
    
    /**
     * Search with filters, grouping results by subroot
     * @param {Object} options - Search options
     * @param {string} options.pattern - Pattern to match (e.g., 'users/*')
     * @param {Array} [options.filters=[]] - Array of filters
     * @param {number} [options.limit=100] - Maximum results
     * @returns {Promise<Array>} Grouped search results
     */
    async search(options) {
        const { 
            pattern, 
            filters = [], 
            limit = 100 
        } = options;
        
        // Validate and normalize filters
        const normalizedFilters = filters.map(f => ({
            field: f.field,
            op: f.op || '==',
            value: String(f.value)
        }));
        
        // Call native search
        const results = await native.search(
            this._store, 
            pattern, 
            normalizedFilters, 
            limit
        );
        
        // Decode values in the results
        return results.map(group => 
            group.map(([key, value]) => [key, WalDB._decodeValue(value)])
        );
    }
    
    /**
     * Search and return as reconstructed objects
     * @param {Object} options - Same as search()
     * @returns {Promise<Array<Object>>} Array of reconstructed objects
     */
    async searchObjects(options) {
        const groups = await this.search(options);
        
        return groups.map(entries => {
            // Find the group key (shortest key)
            const groupKey = entries.reduce((min, [key]) => 
                key.length < min.length ? key : min, entries[0][0]
            ).split('/').slice(0, -1).join('/');
            
            // Reconstruct object from entries
            return this._reconstructFromEntries(entries, groupKey);
        });
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
        return this._db.getObject(this._path);
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