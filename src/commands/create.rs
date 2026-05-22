use crate::client::ZkClientImpl;
use crate::output::CreateResult;
use crate::style;
use anyhow::Result;
use zookeeper_client::CreateMode;

pub async fn run(
    client: &ZkClientImpl,
    path: &str,
    data: Option<&str>,
    ephemeral: bool,
    sequential: bool,
    recursive: bool,
) -> Result<CreateResult> {
    let path = crate::client::normalize_path(path);
    let data = data.unwrap_or("").as_bytes();
    let mode = match (ephemeral, sequential) {
        (true, true) => CreateMode::EphemeralSequential,
        (true, false) => CreateMode::Ephemeral,
        (false, true) => CreateMode::PersistentSequential,
        (false, false) => CreateMode::Persistent,
    };
    let mode_str = match mode {
        CreateMode::Persistent => "persistent",
        CreateMode::Ephemeral => "ephemeral",
        CreateMode::PersistentSequential => "persistent_sequential",
        CreateMode::EphemeralSequential => "ephemeral_sequential",
        CreateMode::Container => "container",
    };

    let (_stat, created_path) = if recursive {
        client.create_recursive(&path, data, mode).await?
    } else {
        client.create(&path, data, mode).await?
    };
    Ok(CreateResult {
        path: created_path,
        mode: mode_str.to_string(),
    })
}

pub fn format_human(r: &CreateResult) -> String {
    format!("{} {} {}", style::success("Created"), style::path(&r.path), style::dim(&format!("({})", r.mode)))
}
