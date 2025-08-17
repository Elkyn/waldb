// Example: Real-time Chat Application using WalDB
// Shows how to build a chat system with rooms, messages, and users

use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

mod waldb_store {
    include!("../waldb.rs");
}

use waldb_store::Store;

struct ChatApp {
    store: Store,
    current_user: String,
    current_room: String,
}

impl ChatApp {
    fn new(store_path: &Path) -> io::Result<Self> {
        let store = Store::open(store_path)?;
        Ok(ChatApp {
            store,
            current_user: String::new(),
            current_room: "general".to_string(),
        })
    }
    
    fn login(&mut self, username: &str) -> io::Result<()> {
        self.current_user = username.to_string();
        
        // Record user login
        let timestamp = Self::timestamp();
        self.store.set(
            &format!("users/{}/last_seen", username),
            &timestamp.to_string(),
            false
        )?;
        
        // Add user to active users
        self.store.set(
            &format!("active_users/{}", username),
            "online",
            false
        )?;
        
        println!("✓ Logged in as {}", username);
        Ok(())
    }
    
    fn logout(&mut self) -> io::Result<()> {
        if !self.current_user.is_empty() {
            // Remove from active users
            self.store.delete(&format!("active_users/{}", self.current_user))?;
            println!("✓ Logged out {}", self.current_user);
            self.current_user.clear();
        }
        Ok(())
    }
    
    fn send_message(&mut self, text: &str) -> io::Result<()> {
        if self.current_user.is_empty() {
            println!("❌ Please login first");
            return Ok(());
        }
        
        let timestamp = Self::timestamp();
        let msg_id = format!("msg_{}", timestamp);
        
        // Store message
        let msg_path = format!("rooms/{}/messages/{}", self.current_room, msg_id);
        self.store.set(&format!("{}/text", msg_path), text, false)?;
        self.store.set(&format!("{}/user", msg_path), &self.current_user, false)?;
        self.store.set(&format!("{}/timestamp", msg_path), &timestamp.to_string(), false)?;
        
        println!("✓ Message sent");
        Ok(())
    }
    
    fn list_messages(&self, limit: usize) -> io::Result<()> {
        println!("\n=== Room: {} ===", self.current_room);
        
        let room_path = format!("rooms/{}/messages/", self.current_room);
        let messages = self.store.scan_prefix(&room_path, limit * 3)?; // 3 fields per message
        
        let mut current_msg = String::new();
        let mut msg_data: (String, String, u64) = (String::new(), String::new(), 0);
        
        for (key, value) in messages {
            let relative = &key[room_path.len()..];
            let parts: Vec<&str> = relative.split('/').collect();
            
            if parts.len() == 2 {
                let msg_id = parts[0];
                let field = parts[1];
                
                if msg_id != current_msg {
                    // Print previous message if exists
                    if !current_msg.is_empty() && !msg_data.0.is_empty() {
                        println!("[{}] {}: {}", 
                            Self::format_time(msg_data.2),
                            msg_data.1,
                            msg_data.0
                        );
                    }
                    current_msg = msg_id.to_string();
                    msg_data = (String::new(), String::new(), 0);
                }
                
                match field {
                    "text" => msg_data.0 = value,
                    "user" => msg_data.1 = value,
                    "timestamp" => msg_data.2 = value.parse().unwrap_or(0),
                    _ => {}
                }
            }
        }
        
        // Print last message
        if !msg_data.0.is_empty() {
            println!("[{}] {}: {}", 
                Self::format_time(msg_data.2),
                msg_data.1,
                msg_data.0
            );
        }
        
        Ok(())
    }
    
    fn switch_room(&mut self, room: &str) -> io::Result<()> {
        self.current_room = room.to_string();
        
        // Record room entry
        if !self.current_user.is_empty() {
            self.store.set(
                &format!("rooms/{}/members/{}", room, self.current_user),
                &Self::timestamp().to_string(),
                false
            )?;
        }
        
        println!("✓ Switched to room: {}", room);
        Ok(())
    }
    
    fn list_rooms(&self) -> io::Result<()> {
        println!("\n=== Available Rooms ===");
        
        let rooms = self.store.get_pattern("rooms/*/messages/")?;
        let mut room_names = std::collections::HashSet::new();
        
        for (key, _) in rooms {
            let parts: Vec<&str> = key.split('/').collect();
            if parts.len() > 1 {
                room_names.insert(parts[1].to_string());
            }
        }
        
        if room_names.is_empty() {
            println!("No rooms yet. Send a message to create one!");
        } else {
            for room in room_names {
                let marker = if room == self.current_room { " (current)" } else { "" };
                println!("  - {}{}", room, marker);
            }
        }
        
        Ok(())
    }
    
    fn list_users(&self) -> io::Result<()> {
        println!("\n=== Active Users ===");
        
        let users = self.store.get("active_users/")?;
        
        if let Some(json) = users {
            // Parse the JSON (simplified)
            let users_list: Vec<&str> = json
                .trim_matches('{').trim_matches('}')
                .split(',')
                .filter_map(|pair| {
                    pair.split(':').next().map(|k| k.trim_matches('"'))
                })
                .collect();
                
            for user in users_list {
                let marker = if user == self.current_user { " (you)" } else { "" };
                println!("  - {}{}", user, marker);
            }
        } else {
            println!("No active users");
        }
        
        Ok(())
    }
    
    fn timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
    
    fn format_time(timestamp: u64) -> String {
        let now = Self::timestamp();
        let diff = now - timestamp;
        
        if diff < 60 {
            format!("{}s ago", diff)
        } else if diff < 3600 {
            format!("{}m ago", diff / 60)
        } else if diff < 86400 {
            format!("{}h ago", diff / 3600)
        } else {
            format!("{}d ago", diff / 86400)
        }
    }
}

fn main() -> io::Result<()> {
    println!("🦌 Antler Chat Example");
    println!("======================\n");
    
    let mut app = ChatApp::new(Path::new("./chat_data"))?;
    
    println!("Commands:");
    println!("  login <username>  - Login as user");
    println!("  send <message>    - Send a message");
    println!("  list              - List recent messages");
    println!("  room <name>       - Switch to room");
    println!("  rooms             - List all rooms");
    println!("  users             - List active users");
    println!("  logout            - Logout");
    println!("  quit              - Exit\n");
    
    let mut input = String::new();
    
    loop {
        print!("> ");
        io::stdout().flush()?;
        
        input.clear();
        io::stdin().read_line(&mut input)?;
        
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        
        match parts[0] {
            "login" => {
                if parts.len() < 2 {
                    println!("Usage: login <username>");
                } else {
                    app.login(parts[1])?;
                }
            }
            "send" => {
                if parts.len() < 2 {
                    println!("Usage: send <message>");
                } else {
                    let message = parts[1..].join(" ");
                    app.send_message(&message)?;
                }
            }
            "list" => {
                app.list_messages(20)?;
            }
            "room" => {
                if parts.len() < 2 {
                    println!("Usage: room <name>");
                } else {
                    app.switch_room(parts[1])?;
                }
            }
            "rooms" => {
                app.list_rooms()?;
            }
            "users" => {
                app.list_users()?;
            }
            "logout" => {
                app.logout()?;
            }
            "quit" | "exit" => {
                app.logout()?;
                println!("Goodbye!");
                break;
            }
            _ => {
                println!("Unknown command: {}", parts[0]);
            }
        }
        
        println!();
    }
    
    Ok(())
}