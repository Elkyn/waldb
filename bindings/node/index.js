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
    
    /**
     * Set a vector embedding
     * @param {string} path - Path to store the vector
     * @param {number[]} vector - Array of numbers representing the vector
     * @returns {Promise<void>}
     */
    async setVector(path, vector) {
        if (!Array.isArray(vector) || !vector.every(v => typeof v === 'number')) {
            throw new Error('Vector must be an array of numbers');
        }
        return native.setVector(this._store, path, vector);
    }
    
    /**
     * Get a vector embedding
     * @param {string} path - Path of the vector
     * @returns {Promise<number[]|null>} Vector array or null if not found
     */
    async getVector(path) {
        return native.getVector(this._store, path);
    }
    
    /**
     * Advanced search with vector similarity, text search, and hybrid scoring
     * @param {Object} options - Search options
     * @param {string} options.pattern - Pattern to match keys
     * @param {Array} [options.filters] - Filter conditions
     * @param {Object} [options.vector] - Vector search options
     * @param {number[]} options.vector.query - Query vector for similarity search
     * @param {string} options.vector.field - Field containing vectors to search
     * @param {number} [options.vector.threshold] - Minimum similarity threshold
     * @param {Object} [options.text] - Text search options
     * @param {string} options.text.query - Text query string
     * @param {string[]} options.text.fields - Fields to search in
     * @param {boolean} [options.text.caseSensitive] - Case sensitive search
     * @param {Object} [options.scoring] - Scoring weights for hybrid search
     * @param {number} [options.scoring.vector] - Weight for vector similarity (default: 1.0)
     * @param {number} [options.scoring.text] - Weight for text relevance (default: 1.0)
     * @param {number} [options.scoring.filter] - Weight for filter matches (default: 1.0)
     * @param {number} [options.limit] - Maximum number of results
     * @returns {Promise<Array<Array<[string, any]>>>} Array of groups, each group is array of [key, value] pairs
     */
    async advancedSearch(options) {
        // Validate required pattern
        if (!options.pattern || typeof options.pattern !== 'string') {
            throw new Error('Pattern is required and must be a string');
        }
        
        // Validate vector search if provided
        if (options.vector) {
            if (!Array.isArray(options.vector.query) || !options.vector.query.every(v => typeof v === 'number')) {
                throw new Error('Vector query must be an array of numbers');
            }
            if (!options.vector.field || typeof options.vector.field !== 'string') {
                throw new Error('Vector field must be a string');
            }
        }
        
        // Validate text search if provided
        if (options.text) {
            if (!options.text.query || typeof options.text.query !== 'string') {
                throw new Error('Text query must be a string');
            }
            if (!Array.isArray(options.text.fields) || !options.text.fields.every(f => typeof f === 'string')) {
                throw new Error('Text fields must be an array of strings');
            }
        }
        
        const results = await native.advancedSearch(this._store, options);
        
        // Decode values in the results
        return results.map(group => 
            group.map(([key, value]) => [key, WalDB._decodeValue(value)])
        );
    }
    
    /**
     * Advanced search that returns reconstructed objects
     * @param {Object} options - Same as advancedSearch()
     * @returns {Promise<Array<Object>>} Array of reconstructed objects with search metadata
     */
    async advancedSearchObjects(options) {
        const groups = await this.advancedSearch(options);
        
        return groups.map(entries => {
            // Find the group key (shortest key without metadata fields)
            const nonMetaEntries = entries.filter(([key]) => 
                !key.endsWith('_vector_score') && !key.endsWith('_text_score') && !key.endsWith('_total_score')
            );
            
            if (nonMetaEntries.length === 0) return {};
            
            const groupKey = nonMetaEntries.reduce((min, [key]) => 
                key.length < min.length ? key : min, nonMetaEntries[0][0]
            ).split('/').slice(0, -1).join('/');
            
            // Reconstruct object from entries
            const obj = this._reconstructFromEntries(entries, groupKey);
            
            // Add search metadata if present
            const vectorScore = entries.find(([key]) => key.endsWith('_vector_score'));
            const textScore = entries.find(([key]) => key.endsWith('_text_score'));
            const totalScore = entries.find(([key]) => key.endsWith('_total_score'));
            
            if (vectorScore || textScore || totalScore) {
                obj._searchMeta = {};
                if (vectorScore) obj._searchMeta.vectorScore = parseFloat(vectorScore[1]);
                if (textScore) obj._searchMeta.textScore = parseFloat(textScore[1]);
                if (totalScore) obj._searchMeta.totalScore = parseFloat(totalScore[1]);
            }
            
            return obj;
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