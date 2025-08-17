const { spawn, spawnSync } = require('child_process');
const path = require('path');
const fs = require('fs');

/**
 * Simple FFI-style Node.js bindings for WalDB using CLI bridge
 * This provides a working implementation while native bindings are in development
 */
class WalDB {
    constructor(storePath) {
        this.storePath = path.resolve(storePath);
        this.waldbBinary = this._findWaldbBinary();
    }

    _findWaldbBinary() {
        // Try different locations for the waldb binary
        const candidates = [
            path.join(__dirname, '../../waldb'),
            path.join(__dirname, '../../target/release/waldb'),
            path.join(__dirname, '../../target/debug/waldb'),
            'waldb' // In PATH
        ];

        for (const candidate of candidates) {
            if (candidate === 'waldb') {
                // Check if it's in PATH
                try {
                    spawnSync('which', ['waldb'], { stdio: 'ignore' });
                    return 'waldb';
                } catch (e) {
                    continue;
                }
            } else if (fs.existsSync(candidate)) {
                return candidate;
            }
        }

        // Try to build it
        try {
            console.log('Building WalDB binary...');
            spawnSync('cargo', ['build', '--release'], { 
                cwd: path.join(__dirname, '../..'),
                stdio: 'inherit'
            });
            
            const builtBinary = path.join(__dirname, '../../target/release/waldb');
            if (fs.existsSync(builtBinary)) {
                return builtBinary;
            }
        } catch (e) {
            // Fall back to debug build
            try {
                spawnSync('cargo', ['build'], { 
                    cwd: path.join(__dirname, '../..'),
                    stdio: 'inherit'
                });
                
                const debugBinary = path.join(__dirname, '../../target/debug/waldb');
                if (fs.existsSync(debugBinary)) {
                    return debugBinary;
                }
            } catch (e2) {
                // Last resort - try rustc directly
                const srcBinary = path.join(__dirname, '../../waldb.rs');
                const outBinary = path.join(__dirname, '../../waldb');
                try {
                    spawnSync('rustc', ['-O', srcBinary, '-o', outBinary], {
                        stdio: 'inherit'
                    });
                    if (fs.existsSync(outBinary)) {
                        return outBinary;
                    }
                } catch (e3) {
                    throw new Error('WalDB binary not found and could not be built');
                }
            }
        }

        throw new Error('WalDB binary not found');
    }

    _runCommand(args) {
        const result = spawnSync(this.waldbBinary, [this.storePath, ...args], {
            encoding: 'utf8',
            stdio: ['pipe', 'pipe', 'pipe']
        });

        if (result.error) {
            throw new Error(`WalDB command failed: ${result.error.message}`);
        }

        if (result.status !== 0) {
            throw new Error(`WalDB command failed: ${result.stderr}`);
        }

        return result.stdout.trim();
    }

    /**
     * Set a value
     * @param {string} key - The key path
     * @param {any} value - The value to set
     * @param {boolean} force - Whether to force overwrite
     */
    set(key, value, force = false) {
        const stringValue = typeof value === 'string' ? value : JSON.stringify(value);
        const args = ['set', key, stringValue];
        if (force) args.push('--force');
        
        this._runCommand(args);
    }

    /**
     * Get a value
     * @param {string} key - The key path
     * @returns {any} The value or null if not found
     */
    get(key) {
        try {
            const result = this._runCommand(['get', key]);
            
            // Try to parse as JSON
            if (result.startsWith('{') || result.startsWith('[')) {
                try {
                    return JSON.parse(result);
                } catch (e) {
                    return result;
                }
            }
            
            return result || null;
        } catch (e) {
            if (e.message.includes('not found')) {
                return null;
            }
            throw e;
        }
    }

    /**
     * Delete a key
     * @param {string} key - The key path
     */
    delete(key) {
        this._runCommand(['delete', key]);
    }

    /**
     * Get pattern matches
     * @param {string} pattern - The pattern with wildcards
     * @returns {Object} Matching key-value pairs
     */
    getPattern(pattern) {
        const result = this._runCommand(['pattern', pattern]);
        const matches = {};
        
        if (result) {
            const lines = result.split('\n');
            for (const line of lines) {
                const match = line.match(/^(.+?) = (.+)$/);
                if (match) {
                    const [, key, value] = match;
                    // Try to parse JSON values
                    if (value.startsWith('{') || value.startsWith('[')) {
                        try {
                            matches[key] = JSON.parse(value);
                        } catch (e) {
                            matches[key] = value;
                        }
                    } else {
                        matches[key] = value;
                    }
                }
            }
        }
        
        return matches;
    }

    /**
     * Get range of keys
     * @param {string} start - Start key
     * @param {string} end - End key
     * @returns {Object} Key-value pairs in range
     */
    getRange(start, end) {
        const result = this._runCommand(['range', start, end]);
        const matches = {};
        
        if (result) {
            const lines = result.split('\n');
            for (const line of lines) {
                const match = line.match(/^(.+?) = (.+)$/);
                if (match) {
                    const [, key, value] = match;
                    // Try to parse JSON values
                    if (value.startsWith('{') || value.startsWith('[')) {
                        try {
                            matches[key] = JSON.parse(value);
                        } catch (e) {
                            matches[key] = value;
                        }
                    } else {
                        matches[key] = value;
                    }
                }
            }
        }
        
        return matches;
    }

    /**
     * Check if key exists
     * @param {string} key - The key to check
     * @returns {boolean} True if exists
     */
    exists(key) {
        return this.get(key) !== null;
    }

    /**
     * Flush pending writes
     */
    flush() {
        this._runCommand(['flush']);
    }

    /**
     * Firebase-style reference
     * @param {string} path - Path for reference
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

    child(childPath) {
        const fullPath = this._path ? `${this._path}/${childPath}` : childPath;
        return new Reference(this._db, fullPath);
    }

    set(value, force = false) {
        return this._db.set(this._path, value, force);
    }

    get() {
        return this._db.get(this._path);
    }

    remove() {
        return this._db.delete(this._path);
    }

    exists() {
        return this._db.exists(this._path);
    }

    toString() {
        return this._path || '/';
    }
}

/**
 * Open a WalDB store
 * @param {string} path - Path to store directory
 * @returns {WalDB} WalDB instance
 */
function open(path) {
    return new WalDB(path);
}

module.exports = {
    open,
    WalDB,
    Reference
};