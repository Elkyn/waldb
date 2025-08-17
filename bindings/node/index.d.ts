/**
 * WalDB - High-performance write-ahead log database with tree semantics
 * TypeScript definitions
 */

declare module '@elkyn/waldb' {
  /**
   * Main database class providing Firebase-like API with async operations
   */
  export class WalDB {
    private constructor(store: any);
    
    /**
     * Open a WalDB database (async)
     * @param path Path to the database directory
     */
    static open(path: string): Promise<WalDB>;
    
    /**
     * Set a value at the given path (async)
     * @param key The path to set
     * @param value The value to set (objects will be flattened)
     * @param force Whether to force overwrite parent nodes
     */
    set(key: string, value: any, force?: boolean): Promise<void>;
    
    /**
     * Get entries with decoded values (default) (async)
     * Returns array of [key, value] pairs with decoded values
     * @param key The path to get
     */
    get(key: string): Promise<Array<[string, any]>>;
    
    /**
     * Get raw entries with prefixed strings (async)
     * Returns array of [key, value] pairs with raw prefixed values like "n:42", "s:hello"
     * @param key The path to get
     */
    getRaw(key: string): Promise<Array<[string, string]>>;
    
    /**
     * Get value or subtree as reconstructed object (async)
     * Returns the value or reconstructed object, null if not found
     * @param key The path to get
     */
    getObject(key: string): Promise<any>;
    
    /**
     * Delete a path and all its children (async)
     * @param key The path to delete
     */
    delete(key: string): Promise<void>;
    
    /**
     * Check if a path exists (async)
     * @param key The path to check
     */
    exists(key: string): Promise<boolean>;
    
    /**
     * Get all values matching a pattern with * and ? wildcards (async)
     * @param pattern Pattern with wildcards
     */
    getPattern(pattern: string): Promise<Record<string, any>>;
    
    /**
     * Get all values in a range (async)
     * @param start Start key (inclusive)
     * @param end End key (exclusive)
     */
    getRange(start: string, end: string): Promise<Record<string, any>>;
    
    /**
     * Get all key-value pairs matching a pattern as entries array (async)
     * @param pattern Pattern with * and ? wildcards
     */
    getPatternEntries(pattern: string): Promise<Array<[string, any]>>;
    
    /**
     * Get all key-value pairs in a range as entries array (async)
     * @param start Start key (inclusive)
     * @param end End key (exclusive)
     */
    getRangeEntries(start: string, end: string): Promise<Array<[string, any]>>;
    
    /**
     * Flush pending writes to disk (async)
     */
    flush(): Promise<void>;
    
    /**
     * Create a Firebase RTDB-style reference
     * @param path The path to reference
     */
    ref(path: string): Reference;
  }

  /**
   * Firebase RTDB-style reference class
   */
  export class Reference {
    private constructor(db: WalDB, path: string);
    
    /**
     * Get child reference
     * @param childPath Child path to append
     */
    child(childPath: string): Reference;
    
    /**
     * Get parent reference
     */
    parent(): Reference;
    
    /**
     * Set value at this reference (async)
     * @param value Value to set
     */
    set(value: any): Promise<void>;
    
    /**
     * Get value at this reference (async)
     * Returns reconstructed object/value
     */
    get(): Promise<any>;
    
    /**
     * Remove value at this reference (async)
     */
    remove(): Promise<void>;
  }

  // Default export
  export default WalDB;
  
  // Named exports for convenience
  export { WalDB, Reference };
}