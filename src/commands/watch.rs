use crate::client::ZkClientImpl;
use crate::style;
use anyhow::Result;
use serde::Serialize;
use zookeeper_client::AddWatchMode;

#[derive(Serialize)]
#[allow(dead_code)]
pub struct WatchEvent {
    pub path: String,
    pub event_type: String,
}

#[allow(dead_code)]
pub async fn run_oneshot(client: &ZkClientImpl, path: &str) -> Result<String> {
    let path = crate::client::normalize_path(path);
    let (_data, _stat, watcher) = client.watch_data(&path).await?;
    let event = watcher.changed().await;
    Ok(format!("{:?}", event))
}

pub async fn run_persistent(client: &ZkClientImpl, path: &str) -> Result<()> {
    let path = crate::client::normalize_path(path);
    let mut watcher = client
        .watch(&path, AddWatchMode::PersistentRecursive)
        .await?;
    loop {
        let event = watcher.changed().await;
        let event_type = format!("{:?}", event.event_type);
        let event_path = event.path.clone();
        println!("{} {}", style::warn("WatchEvent:"), style::path(&format!("{} {:?}", event_path, event_type)));
    }
}
