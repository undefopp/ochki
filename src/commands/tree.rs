use crate::client::ZkClientImpl;
use crate::output::{TreeEntry, TreeResult};
use crate::style;
use anyhow::Result;

pub async fn run(
    client: &ZkClientImpl,
    path: &str,
    max_depth: Option<usize>,
) -> Result<TreeResult> {
    let path = crate::client::normalize_path(path);
    let mut entries = Vec::new();
    build_tree(client, &path, &mut entries, 0, max_depth).await?;
    Ok(TreeResult {
        path,
        tree: entries,
    })
}

pub fn format_human(r: &TreeResult) -> String {
    let mut lines = Vec::new();
    for entry in &r.tree {
        let name = if entry.depth == 0 {
            style::path(&r.path).to_string()
        } else {
            let raw = entry.path.rsplit('/').next().unwrap_or(&entry.path);
            style::path(raw).to_string()
        };
        let prefix = if entry.depth == 0 {
            String::new()
        } else {
            style::dim(&format!("{}{}", "\u{2502} ".repeat(entry.depth - 1), "\u{251c}\u{2500}")).to_string()
        };
        lines.push(format!("{}{}", prefix, name));
    }
    lines.join("\n")
}

fn build_tree<'a>(
    client: &'a ZkClientImpl,
    path: &'a str,
    entries: &'a mut Vec<TreeEntry>,
    depth: usize,
    max_depth: Option<usize>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(async move {
        if max_depth.is_some_and(|m| depth > m) {
            return Ok(());
        }
        entries.push(TreeEntry {
            path: path.to_string(),
            depth,
        });
        if let Ok(children) = client.ls(path).await {
            for child in &children {
                let full = format!("{}/{}", path, child).replace("//", "/");
                build_tree(client, &full, entries, depth + 1, max_depth).await?;
            }
        }
        Ok(())
    })
}
