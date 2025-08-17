// Interactive CLI for WalDB Store
// Provides a shell interface to test all features

use std::io::{self, Write};
use std::path::Path;
use std::time::Instant;

mod waldb_store {
    include!("waldb.rs");
}

use waldb_store::Store;

fn main() -> io::Result<()> {
    println!("🗄️ WalDB CLI v0.1.0");
    println!("Type 'help' for commands, 'quit' to exit\n");
    
    // Open or create store in current directory
    let store_path = "./waldb_data";
    println!("Opening store at: {}", store_path);
    let store = Store::open(Path::new(store_path))?;
    println!("Store ready!\n");
    
    let mut input = String::new();
    
    loop {
        print!("waldb> ");
        io::stdout().flush()?;
        
        input.clear();
        io::stdin().read_line(&mut input)?;
        
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        
        let start = Instant::now();
        
        match parts[0] {
            "help" | "h" | "?" => {
                print_help();
            }
            
            "set" | "s" => {
                if parts.len() < 3 {
                    println!("Usage: set <key> <value> [replace_subtree]");
                    continue;
                }
                let key = parts[1];
                let value = parts[2..].join(" ");
                let replace = parts.len() > 3 && parts[parts.len() - 1] == "true";
                
                match store.set(key, &value, replace) {
                    Ok(_) => println!("✓ Set '{}' = '{}'", key, value),
                    Err(e) => println!("✗ Error: {}", e),
                }
            }
            
            "get" | "g" => {
                if parts.len() < 2 {
                    println!("Usage: get <key>");
                    continue;
                }
                let key = parts[1];
                
                match store.get(key) {
                    Ok(Some(value)) => {
                        if key.ends_with('/') {
                            // It's a subtree query, pretty print JSON
                            println!("{}", pretty_json(&value));
                        } else {
                            println!("{}", value);
                        }
                    }
                    Ok(None) => println!("(not found)"),
                    Err(e) => println!("✗ Error: {}", e),
                }
            }
            
            "delete" | "del" | "d" => {
                if parts.len() < 2 {
                    println!("Usage: delete <key>");
                    continue;
                }
                let key = parts[1];
                
                match store.delete(key) {
                    Ok(_) => println!("✓ Deleted '{}'", key),
                    Err(e) => println!("✗ Error: {}", e),
                }
            }
            
            "delete-subtree" | "delst" => {
                if parts.len() < 2 {
                    println!("Usage: delete-subtree <prefix>");
                    continue;
                }
                let prefix = parts[1];
                
                match store.delete_subtree(prefix) {
                    Ok(_) => println!("✓ Deleted subtree '{}'", prefix),
                    Err(e) => println!("✗ Error: {}", e),
                }
            }
            
            "pattern" | "p" => {
                if parts.len() < 2 {
                    println!("Usage: pattern <pattern>");
                    println!("  Wildcards: * = any chars, ? = single char");
                    continue;
                }
                let pattern = parts[1];
                
                match store.get_pattern(pattern) {
                    Ok(results) => {
                        println!("Found {} matches:", results.len());
                        for (k, v) in results.iter().take(20) {
                            println!("  {} = {}", k, truncate(v, 50));
                        }
                        if results.len() > 20 {
                            println!("  ... and {} more", results.len() - 20);
                        }
                    }
                    Err(e) => println!("✗ Error: {}", e),
                }
            }
            
            "delete-pattern" | "delp" => {
                if parts.len() < 2 {
                    println!("Usage: delete-pattern <pattern>");
                    continue;
                }
                let pattern = parts[1];
                
                match store.delete_pattern(pattern) {
                    Ok(count) => println!("✓ Deleted {} keys matching '{}'", count, pattern),
                    Err(e) => println!("✗ Error: {}", e),
                }
            }
            
            "range" | "r" => {
                if parts.len() < 3 {
                    println!("Usage: range <start> <end> [limit]");
                    continue;
                }
                let start = parts[1];
                let end = parts[2];
                let limit = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(20);
                
                match store.get_range_limit(start, end, limit) {
                    Ok(results) => {
                        println!("Range [{}..{}) - {} results:", start, end, results.len());
                        for (k, v) in &results {
                            println!("  {} = {}", k, truncate(v, 50));
                        }
                    }
                    Err(e) => println!("✗ Error: {}", e),
                }
            }
            
            "scan" => {
                if parts.len() < 2 {
                    println!("Usage: scan <prefix> [limit]");
                    continue;
                }
                let prefix = parts[1];
                let limit = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(20);
                
                match store.scan_prefix(prefix, limit) {
                    Ok(results) => {
                        println!("Prefix '{}' - {} results:", prefix, results.len());
                        for (k, v) in &results {
                            println!("  {} = {}", k, truncate(v, 50));
                        }
                    }
                    Err(e) => println!("✗ Error: {}", e),
                }
            }
            
            "flush" | "f" => {
                match store.flush() {
                    Ok(_) => println!("✓ Flushed to disk"),
                    Err(e) => println!("✗ Error: {}", e),
                }
            }
            
            "stats" => {
                let (l0, l1, l2) = store.segment_counts();
                println!("Segment counts:");
                println!("  L0: {} segments", l0);
                println!("  L1: {} segments", l1);
                println!("  L2: {} segments", l2);
            }
            
            "bench" => {
                run_benchmark(&store);
            }
            
            "load" => {
                if parts.len() < 2 {
                    println!("Usage: load <prefix> [count]");
                    continue;
                }
                let prefix = parts[1];
                let count = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(100);
                
                println!("Loading {} test keys with prefix '{}'...", count, prefix);
                for i in 0..count {
                    let key = format!("{}/key{:04}", prefix, i);
                    let value = format!("value{}", i);
                    store.set(&key, &value, false)?;
                }
                println!("✓ Loaded {} keys", count);
            }
            
            "tree" => {
                if parts.len() < 2 {
                    println!("Usage: tree <prefix>");
                    continue;
                }
                let prefix = parts[1];
                print_tree(&store, prefix, 0, 3);
            }
            
            "clear" => {
                print!("\x1B[2J\x1B[1;1H");
                println!("🦌 Antler Store CLI v0.1.0\n");
            }
            
            "quit" | "exit" | "q" => {
                println!("Goodbye!");
                break;
            }
            
            _ => {
                println!("Unknown command: '{}'. Type 'help' for available commands.", parts[0]);
            }
        }
        
        let elapsed = start.elapsed();
        if elapsed.as_millis() > 0 {
            println!("({}ms)", elapsed.as_millis());
        }
        println!();
    }
    
    Ok(())
}

fn print_help() {
    println!("Available commands:");
    println!();
    println!("  Basic Operations:");
    println!("    set <key> <value> [replace]  - Set a key-value pair");
    println!("    get <key>                     - Get value by key (append / for subtree)");
    println!("    delete <key>                  - Delete a key");
    println!("    delete-subtree <prefix>       - Delete entire subtree");
    println!();
    println!("  Pattern Matching:");
    println!("    pattern <pattern>             - Find keys matching pattern (* and ? wildcards)");
    println!("    delete-pattern <pattern>      - Delete keys matching pattern");
    println!();
    println!("  Range Queries:");
    println!("    range <start> <end> [limit]   - Get keys in range");
    println!("    scan <prefix> [limit]         - Scan keys with prefix");
    println!();
    println!("  Management:");
    println!("    flush                         - Flush memtable to disk");
    println!("    stats                         - Show segment statistics");
    println!("    bench                         - Run performance benchmark");
    println!("    load <prefix> [count]         - Load test data");
    println!("    tree <prefix>                 - Show tree structure");
    println!();
    println!("  Other:");
    println!("    clear                         - Clear screen");
    println!("    help                          - Show this help");
    println!("    quit                          - Exit the CLI");
    println!();
    println!("  Shortcuts: s=set, g=get, d=delete, p=pattern, r=range, f=flush, q=quit");
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

fn pretty_json(json: &str) -> String {
    // Simple JSON pretty printer
    let mut result = String::new();
    let mut indent = 0;
    let mut in_string = false;
    let mut escape_next = false;
    
    for ch in json.chars() {
        if escape_next {
            result.push(ch);
            escape_next = false;
            continue;
        }
        
        match ch {
            '\\' if in_string => {
                result.push(ch);
                escape_next = true;
            }
            '"' => {
                result.push(ch);
                in_string = !in_string;
            }
            '{' | '[' if !in_string => {
                result.push(ch);
                result.push('\n');
                indent += 2;
                result.push_str(&" ".repeat(indent));
            }
            '}' | ']' if !in_string => {
                result.push('\n');
                indent -= 2;
                result.push_str(&" ".repeat(indent));
                result.push(ch);
            }
            ',' if !in_string => {
                result.push(ch);
                result.push('\n');
                result.push_str(&" ".repeat(indent));
            }
            ':' if !in_string => {
                result.push_str(": ");
            }
            ' ' if !in_string => {
                // Skip spaces outside strings
            }
            _ => {
                result.push(ch);
            }
        }
    }
    
    result
}

fn print_tree(store: &Store, prefix: &str, indent: usize, max_depth: usize) {
    if indent > max_depth {
        return;
    }
    
    let pattern = if prefix.ends_with('/') {
        format!("{}*", prefix)
    } else {
        format!("{}", prefix)
    };
    
    if let Ok(results) = store.get_pattern(&pattern) {
        let mut paths: Vec<_> = results.into_iter().map(|(k, v)| (k, v)).collect();
        paths.sort_by(|a, b| a.0.cmp(&b.0));
        
        let mut seen = std::collections::HashSet::new();
        
        for (key, value) in paths {
            let relative = if key.starts_with(prefix) {
                &key[prefix.len()..]
            } else {
                &key
            };
            
            let parts: Vec<&str> = relative.split('/').filter(|s| !s.is_empty()).collect();
            if parts.is_empty() {
                continue;
            }
            
            // Only show direct children
            let child = parts[0];
            if seen.contains(child) {
                continue;
            }
            seen.insert(child.to_string());
            
            let full_path = if prefix.ends_with('/') {
                format!("{}{}", prefix, child)
            } else {
                format!("{}/{}", prefix, child)
            };
            
            let is_leaf = parts.len() == 1;
            let indent_str = "  ".repeat(indent);
            let prefix_str = indent_str.as_str();
            
            if is_leaf {
                println!("{}├── {} = {}", prefix_str, child, truncate(&value, 30));
            } else {
                println!("{}├── {}/", prefix_str, child);
                print_tree(store, &format!("{}/", full_path), indent + 1, max_depth);
            }
        }
    }
}

fn run_benchmark(store: &Store) {
    println!("Running benchmark...");
    
    // Write benchmark
    let start = Instant::now();
    for i in 0..1000 {
        let key = format!("bench/key{:04}", i);
        let value = format!("value{}", i);
        let _ = store.set(&key, &value, false);
    }
    let write_elapsed = start.elapsed();
    let write_ops = 1000.0 / write_elapsed.as_secs_f64();
    
    // Read benchmark
    let start = Instant::now();
    for i in 0..1000 {
        let key = format!("bench/key{:04}", i);
        let _ = store.get(&key);
    }
    let read_elapsed = start.elapsed();
    let read_ops = 1000.0 / read_elapsed.as_secs_f64();
    
    // Clean up
    let _ = store.delete_pattern("bench/*");
    
    println!("Results:");
    println!("  Writes: 1000 ops in {:?} = {:.0} ops/sec", write_elapsed, write_ops);
    println!("  Reads:  1000 ops in {:?} = {:.0} ops/sec", read_elapsed, read_ops);
}