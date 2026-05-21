use crate::client::ZkClientImpl;
use crate::output::StatJson;
use anyhow::Result;

pub async fn run(client: &ZkClientImpl, path: &str) -> Result<StatJson> {
    let path = crate::client::normalize_path(path);
    let stat = client.stat(&path).await?.ok_or_else(|| anyhow::anyhow!("Node not found: {}", path))?;
    Ok(StatJson::from(stat))
}

pub fn format_human(s: &StatJson) -> String {
    format!(
        "cZxid: {}\nmZxid: {}\ncTime: {}\nmTime: {}\nVersion: {}\ncVersion: {}\naversion: {}\nEphemeralOwner: {}\nDataLength: {}\nNumChildren: {}\npZxid: {}",
        s.czxid, s.mzxid, s.ctime, s.mtime,
        s.version, s.cversion, s.aversion,
        s.ephemeral_owner, s.data_length, s.num_children, s.pzxid,
    )
}
