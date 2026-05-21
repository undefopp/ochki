use crate::client::ZkClientImpl;
use crate::output::{TreeEntry, TreeResult};
use anyhow::Result;

pub async fn run(client: &ZkClientImpl, path: &str, max_depth: Option<usize>) -> Result<TreeResult> {
    let path = crate::client::normalize_path(path);
    let mut entries = Vec::new();
    build_tree(client, &path, &mut entries, 0, max_depth).await?;
    Ok(TreeResult { path, tree: entries })
}

pub fn format_human(r: &TreeResult) -> String {
    let mut lines = Vec::new();
    for entry in &r.tree {
        let name = if entry.depth == 0 {
            r.path.clone()
        } else {
            entry.path.rsplit('/').next().unwrap_or(&entry.path).to_string()
        };
        let prefix = if entry.depth == 0 {
            "".to_string()
        } else {
            "│ ".repeat(entry.depth - 1) + "├─"
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
        if max_depth.map_or(false, |m| depth > m) {
            return Ok(());
        }
        entries.push(TreeEntry { path: path.to_string(), depth });
        match client.ls(path).await {
            Ok(children) => {
                for child in &children {
                    let full = format!("{}/{}", path, child).replace("//", "/");
                    build_tree(client, &full, entries, depth + 1, max_depth).await?;
                }
            }
            Err(_) => {}
        }
        Ok(())
    })
}
