use crate::client::ZkClientImpl;
use crate::output::StatJson;
use crate::style;
use anyhow::Result;

pub async fn run(client: &ZkClientImpl, path: &str) -> Result<StatJson> {
    let path = crate::client::normalize_path(path);
    let stat = client
        .stat(&path)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Node not found: {}", path))?;
    Ok(StatJson::from(stat))
}

pub fn format_human(s: &StatJson) -> String {
    [
        style::kv("cZxid", &s.czxid.to_string()),
        style::kv("mZxid", &s.mzxid.to_string()),
        style::kv("cTime", &s.ctime.to_string()),
        style::kv("mTime", &s.mtime.to_string()),
        style::kv("Version", &s.version.to_string()),
        style::kv("cVersion", &s.cversion.to_string()),
        style::kv("aVersion", &s.aversion.to_string()),
        style::kv("EphemeralOwner", &s.ephemeral_owner.to_string()),
        style::kv("DataLength", &s.data_length.to_string()),
        style::kv("NumChildren", &s.num_children.to_string()),
        style::kv("pZxid", &s.pzxid.to_string()),
    ].join("\n")
}
