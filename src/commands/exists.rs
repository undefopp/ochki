use crate::client::ZkClientImpl;
use crate::output::ExistsResult;
use crate::style;
use anyhow::Result;

pub async fn run(client: &ZkClientImpl, path: &str, verbose: bool) -> Result<ExistsResult> {
    let path = crate::client::normalize_path(path);
    match client.stat(&path).await? {
        Some(stat) => {
            let s = if verbose {
                Some(crate::output::StatJson::from(stat))
            } else {
                None
            };
            Ok(ExistsResult {
                path,
                exists: true,
                stat: s,
            })
        }
        None => Ok(ExistsResult {
            path,
            exists: false,
            stat: None,
        }),
    }
}

pub fn format_human(r: &ExistsResult) -> Option<String> {
    r.stat.as_ref().map(|s| {
        format!(
            "{} {} {}",
            style::success("\u{2713}"),
            style::path(&r.path),
            style::dim(&format!("(version={}, dataLength={}, numChildren={})", s.version, s.data_length, s.num_children)),
        )
    })
}
