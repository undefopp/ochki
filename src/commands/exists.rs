use crate::client::ZkClientImpl;
use crate::output::ExistsResult;
use anyhow::Result;

pub async fn run(client: &ZkClientImpl, path: &str, verbose: bool) -> Result<ExistsResult> {
    let path = crate::client::normalize_path(path);
    match client.stat(&path).await? {
        Some(stat) => {
            let s = if verbose { Some(crate::output::StatJson::from(stat)) } else { None };
            Ok(ExistsResult { path, exists: true, stat: s })
        }
        None => Ok(ExistsResult { path, exists: false, stat: None }),
    }
}

pub fn format_human(r: &ExistsResult) -> Option<String> {
    if let Some(s) = &r.stat {
        Some(format!("Node {} exists\nVersion: {}, DataLength: {}, NumChildren: {}", r.path, s.version, s.data_length, s.num_children))
    } else {
        None
    }
}
