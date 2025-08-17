// WalDB Node.js API
// High-level JavaScript interface that provides Firebase RTDB-like API

const { open, set, get, delete: del, exists, getPattern, getRange, listKeys, flush } = require('./index.node');

/**
 * WalDB Database class providing Firebase-like API
 */
class WalDB {
    constructor(store) {
        this._store = store;
    }

    /**
     * Open a WalDB database
     * @param {string} path - Path to the database directory
     * @returns {WalDB} Database instance
     */
    static open(path) {
        const store = open(path);
        return new WalDB(store);
    }

    /**
     * Set a value at the given path
     * @param {string} path - The path to set
     * @param {any} value - The value to set (will be JSON stringified if object)
     * @param {boolean} [force=false] - Whether to force overwrite parent nodes
     */
    set(path, value, force = false) {
        const stringValue = typeof value === 'string' ? value : JSON.stringify(value);
        set(this._store, path, stringValue, force);
    }

    /**
     * Get a value or subtree at the given path
     * @param {string} path - The path to get
     * @returns {any} The value or null if not found
     */
    get(path) {
        return get(this._store, path);
    }

    /**
     * Delete a path and all its children
     * @param {string} path - The path to delete
     */
    delete(path) {
        del(this._store, path);
    }

    /**
     * Check if a path exists
     * @param {string} path - The path to check
     * @returns {boolean} True if the path exists
     */
    exists(path) {
        return exists(this._store, path);
    }

    /**
     * Get all values matching a pattern
     * @param {string} pattern - Pattern with * and ? wildcards
     * @returns {Object} Object with matching key-value pairs
     */
    getPattern(pattern) {
        return getPattern(this._store, pattern);
    }

    /**
     * Get all values in a range
     * @param {string} start - Start key (inclusive)
     * @param {string} end - End key (exclusive)
     * @returns {Object} Object with matching key-value pairs
     */
    getRange(start, end) {
        return getRange(this._store, start, end);
    }

    /**
     * List all keys with a given prefix
     * @param {string} prefix - The prefix to match
     * @returns {string[]} Array of matching keys
     */
    listKeys(prefix) {
        return listKeys(this._store, prefix);
    }

    /**
     * Flush pending writes to disk
     */
    flush() {
        flush(this._store);
    }

    /**
     * Firebase RTDB-style reference API
     * @param {string} path - Path to create reference for
     * @returns {Reference} Reference object
     */
    ref(path = '') {
        return new Reference(this, path);
    }
}

/**
 * Firebase RTDB-style reference class
 */
class Reference {
    constructor(db, path) {
        this._db = db;
        this._path = path;
    }

    /**
     * Get child reference
     * @param {string} childPath - Child path
     * @returns {Reference} Child reference
     */
    child(childPath) {
        const fullPath = this._path ? `${this._path}/${childPath}` : childPath;
        return new Reference(this._db, fullPath);
    }

    /**
     * Set value at this reference
     * @param {any} value - Value to set
     * @param {boolean} [force=false] - Whether to force overwrite
     */
    set(value, force = false) {
        this._db.set(this._path, value, force);
    }

    /**
     * Get value at this reference
     * @returns {any} The value
     */
    get() {
        return this._db.get(this._path);
    }

    /**
     * Remove value at this reference
     */
    remove() {
        this._db.delete(this._path);
    }

    /**
     * Check if this reference exists
     * @returns {boolean} True if exists
     */
    exists() {
        return this._db.exists(this._path);
    }

    /**
     * Get the path of this reference
     * @returns {string} The path
     */
    toString() {
        return this._path || '/';
    }
}

// Export both the class and a convenience function
module.exports = WalDB;
module.exports.open = (path) => WalDB.open(path);
module.exports.WalDB = WalDB;
module.exports.Reference = Reference;