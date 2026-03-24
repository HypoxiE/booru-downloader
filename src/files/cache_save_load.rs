//write
//let mut cache = CACHE_COUNT_IMAGES.write().await;
//cache.insert("key1".to_string(), "value1".to_string());
//read
//let cache = CACHE_COUNT_IMAGES.read().await;
//println!("{:?}", cache.get("key1"));

use tokio::{fs::File, fs, io::AsyncWriteExt};
use serde_json;
use tokio::time::{sleep, Duration};

use tokio::sync::RwLock;
use std::collections::HashMap;
use once_cell::sync::Lazy;

use std::path::PathBuf;

use crate::files::fs_api;

/// url -> [timestamp, count]
pub static CACHE_COUNT_IMAGES: Lazy<RwLock<HashMap<u64, [u64; 2]>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub static CACHE_FILE: Lazy<PathBuf> = Lazy::new(|| fs_api::get_cache_path("cache.json"));

async fn dump_cache_to_file() -> std::io::Result<()> {

    let cache = CACHE_COUNT_IMAGES.read().await;
    let data = serde_json::to_string(&*cache).unwrap();

    let mut file = File::create(&*CACHE_FILE).await?;
    file.write_all(data.as_bytes()).await?;
    Ok(())
}

pub async fn auto_dump_cache(interval_secs: u64) {
    loop {
        sleep(Duration::from_secs(interval_secs)).await;
        if let Err(e) = dump_cache_to_file().await {
            eprintln!("Failed to dump cache: {:?}", e);
        }
    }
}

pub async fn load_cache_from_file() -> Result<(), Box<dyn std::error::Error>> {

	match fs::read(&*CACHE_FILE).await {
		Ok(data) => {
			let map: HashMap<u64, [u64; 2]> = serde_json::from_slice(&data)?;
			let mut cache = CACHE_COUNT_IMAGES.write().await;
			*cache = map;
			println!("Loaded {} entries from cache file", cache.len());
		}
		Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
			println!("Cache file '{:?}' not found. Starting with empty cache.", *CACHE_FILE);
		}
		Err(e) => {
			return Err(Box::new(e));
		}
	}
	Ok(())
}