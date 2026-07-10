use std::thread;
use std::time::Duration;
use std::sync::mpsc::Sender;
use sha2::{Digest, Sha256};
use image::{ImageBuffer, Rgba};
use crate::db::{self, EntryKind};
use chrono::{DateTime, Utc};

fn compute_text_hash(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect::<String>()
}

fn compute_image_hash(width: usize, height: usize, bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(&width.to_ne_bytes());
    hasher.update(&height.to_ne_bytes());
    hasher.update(bytes);
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect::<String>()
}

/// Spawns a background thread to poll the OS clipboard every 400ms.
///
/// If new text or image data is detected, it is hashed, stored in the SQLite database,
/// and a message is sent over the channel to notify the GUI to refresh.
pub fn start_clipboard_poller(refresh_tx: Sender<()>) {
    thread::spawn(move || {
        // Open a separate database connection for the background thread
        let conn = match db::init_db() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to open DB in poller thread: {}", e);
                return;
            }
        };

        let mut clipboard = match arboard::Clipboard::new() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to initialize clipboard in background thread: {}", e);
                return;
            }
        };

        // Initialize the last seen hash and timestamp from the latest entry in the database
        let mut last_seen_time = Utc::now();
        let mut last_seen_hash = String::new();

        if let Ok((h, time_str)) = conn.query_row::<(String, String), _, _>(
            "SELECT content_hash, created_at FROM clippy_history ORDER BY created_at DESC LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ) {
            if let Ok(dt) = DateTime::parse_from_rfc3339(&time_str) {
                last_seen_hash = h;
                last_seen_time = dt.with_timezone(&Utc);
            }
        }

        println!("Clipboard poller thread started. Initial hash: {}", last_seen_hash);

        loop {
            // Sync last_seen_hash with the absolute latest database entry (ignoring pinned ordering)
            // Only sync if the database entry is strictly newer than our last seen time (prevents syncing back to older pinned items on Clear all)
            if let Ok((h, time_str)) = conn.query_row::<(String, String), _, _>(
                "SELECT content_hash, created_at FROM clippy_history ORDER BY created_at DESC LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            ) {
                if let Ok(dt) = DateTime::parse_from_rfc3339(&time_str) {
                    let dt_utc = dt.with_timezone(&Utc);
                    if dt_utc > last_seen_time {
                        last_seen_hash = h;
                        last_seen_time = dt_utc;
                    }
                }
            }

            let mut changed = false;

            // Try reading clipboard text
            match clipboard.get_text() {
                Ok(text) if !text.trim().is_empty() => {
                    let hash = compute_text_hash(&text);
                    if hash != last_seen_hash {
                        println!("New text copied: {}", text.chars().take(30).collect::<String>());
                        match db::insert_entry(&conn, EntryKind::Text, Some(&text), None, &hash) {
                            Ok(_) => {
                                let _ = db::prune_entries(&conn, 200);
                                last_seen_hash = hash;
                                last_seen_time = Utc::now();
                                changed = true;
                            }
                            Err(e) => eprintln!("Failed to insert text entry: {}", e),
                        }
                    }
                }
                _ => {
                    // Try reading clipboard image if text is not available or empty
                    match clipboard.get_image() {
                        Ok(image_data) => {
                            let hash = compute_image_hash(image_data.width, image_data.height, &image_data.bytes);
                            if hash != last_seen_hash {
                                println!("New image copied ({}x{})", image_data.width, image_data.height);
                                
                                let images_dir = db::get_images_dir();
                                let image_filename = format!("{}.png", hash);
                                let image_path = images_dir.join(&image_filename);
                                
                                // Save raw RGBA pixel data to disk as PNG
                                let buffer: Option<ImageBuffer<Rgba<u8>, Vec<u8>>> = 
                                    ImageBuffer::from_raw(image_data.width as u32, image_data.height as u32, image_data.bytes.to_vec());
                                
                                if let Some(buf) = buffer {
                                    if let Err(e) = buf.save(&image_path) {
                                        eprintln!("Failed to save image file: {}", e);
                                    } else {
                                        let image_path_str = image_path.to_string_lossy();
                                        match db::insert_entry(&conn, EntryKind::Image, None, Some(&image_path_str), &hash) {
                                            Ok(_) => {
                                                let _ = db::prune_entries(&conn, 200);
                                                last_seen_hash = hash;
                                                last_seen_time = Utc::now();
                                                changed = true;
                                            }
                                            Err(e) => eprintln!("Failed to insert image entry: {}", e),
                                        }
                                    }
                                } else {
                                    eprintln!("Failed to create ImageBuffer from raw clipboard bytes");
                                }
                            }
                        }
                        Err(_) => {
                            // Clipboard is empty or contains unsupported types
                        }
                    }
                }
            }

            if changed {
                // Send refresh signal to UI
                if let Err(e) = refresh_tx.send(()) {
                    eprintln!("Failed to send refresh signal to main thread: {}", e);
                }
            }

            thread::sleep(Duration::from_millis(400));
        }
    });
}
