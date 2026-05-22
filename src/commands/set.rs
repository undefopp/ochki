use crate::client::ZkClientImpl;
use crate::output::{SetResult, StatJson};
use crate::style;
use anyhow::Result;

pub async fn run(
    client: &ZkClientImpl,
    path: &str,
    data: &str,
    version: Option<i32>,
) -> Result<SetResult> {
    let path = crate::client::normalize_path(path);
    let stat = client.set(&path, data.as_bytes(), version).await?;
    Ok(SetResult {
        path,
        version: stat.version,
        stat: StatJson::from(stat),
    })
}

pub fn format_human(r: &SetResult) -> String {
    format!("{} {} ({})", style::success("Updated"), style::path(&r.path), style::kv("version", &r.version.to_string()))
}
