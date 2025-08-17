/**
 * WalDB - High-performance write-ahead log database with tree semantics
 * TypeScript definitions
 */

declare module '@elkyn/waldb' {
  /**
   * Main database class providing Firebase-like API
   */
  export class WalDB {
    constructor(path: string);
    
    /**
     * Open a WalDB database
     */
    static open(path: string): WalDB;
    
    /**
     * Set a value at the given path
     */
    set(key: string, value: any, force?: boolean): void;
    
    /**
     * Get a value or subtree at the given path
     */
    get(key: string): any;
    
    /**
     * Delete a path and all its children
     */
    delete(key: string): void;
    
    /**
     * Check if a path exists
     */
    exists(key: string): boolean;
    
    /**
     * Get all values matching a pattern with * and ? wildcards
     */
    getPattern(pattern: string): Record<string, any>;
    
    /**
     * Get all values in a range
     */
    getRange(start: string, end: string): Record<string, any>;
    
    /**
     * List all keys with a given prefix
     */
    listKeys(prefix: string): string[];
    
    /**
     * Flush pending writes to disk
     */
    flush(): void;
    
    /**
     * Create a Firebase RTDB-style reference
     */
    ref(path?: string): Reference;
  }

  /**
   * Firebase RTDB-style reference class
   */
  export class Reference {
    /**
     * Get child reference
     */
    child(childPath: string): Reference;
    
    /**
     * Set value at this reference
     */
    set(value: any, force?: boolean): void;
    
    /**
     * Get value at this reference
     */
    get(): any;
    
    /**
     * Remove value at this reference
     */
    remove(): void;
    
    /**
     * Check if this reference exists
     */
    exists(): boolean;
    
    /**
     * Get the path of this reference
     */
    toString(): string;
  }

  /**
   * Open a WalDB store
   */
  export function open(path: string): WalDB;
}