use crate::client::ZkClientImpl;
use crate::output::DiffResult;
use anyhow::Result;
use std::collections::HashMap;

pub async fn run(client1: &ZkClientImpl, client2: &ZkClientImpl, path: &str) -> Result<DiffResult> {
    let path = crate::client::normalize_path(path);
    let tree1 = dump_subtree(client1, &path).await?;
    let tree2 = dump_subtree(client2, &path).await?;

    let keys1: std::collections::HashSet<_> = tree1.keys().cloned().collect();
    let keys2: std::collections::HashSet<_> = tree2.keys().cloned().collect();

    let added: Vec<String> = keys2.difference(&keys1).cloned().collect();
    let removed: Vec<String> = keys1.difference(&keys2).cloned().collect();
    let modified: Vec<String> = keys1.intersection(&keys2).filter(|k| tree1[*k] != tree2[*k]).cloned().collect();

    Ok(DiffResult {
        path,
        added,
        removed,
        modified,
    })
}

pub fn format_human(r: &DiffResult) -> String {
    let mut lines = Vec::new();
    for k in &r.removed { lines.push(format!("- {} (only in host1)", k)); }
    for k in &r.added { lines.push(format!("+ {} (only in host2)", k)); }
    for k in &r.modified { lines.push(format!("~ {} (data differs)", k)); }
    if lines.is_empty() {
        "No differences found".to_string()
    } else {
        lines.join("\n")
    }
}

fn dump_subtree<'a>(
    client: &'a ZkClientImpl,
    path: &'a str,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<HashMap<String, Vec<u8>>>> + Send + 'a>> {
    Box::pin(async move {
        let mut map = HashMap::new();
        collect(client, path, &mut map).await?;
        Ok(map)
    })
}

fn collect<'a>(
    client: &'a ZkClientImpl,
    path: &'a str,
    map: &'a mut HashMap<String, Vec<u8>>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(async move {
        match client.get(path).await {
            Ok((data, _stat)) => {
                map.insert(path.to_string(), data);
                if let Ok(children) = client.ls(path).await {
                    for child in &children {
                        let full = format!("{}/{}", path, child).replace("//", "/");
                        collect(client, &full, map).await?;
                    }
                }
            }
            Err(_) => {}
        }
        Ok(())
    })
}
