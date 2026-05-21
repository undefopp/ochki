use crate::client::ZkClientImpl;
use crate::output::{ChildInfo, LsDetailedResult, LsResult};
use anyhow::Result;

pub enum LsOutput {
    Simple(LsResult),
    Detailed(LsDetailedResult),
}

pub async fn run(client: &ZkClientImpl, path: &str, recursive: bool, detailed: bool) -> Result<LsOutput> {
    let path = crate::client::normalize_path(path);
    if recursive {
        let mut entries = Vec::new();
        collect_recursive(client, &path, &mut entries).await?;
        Ok(LsOutput::Simple(LsResult {
            path: path.clone(),
            children: entries,
        }))
    } else if detailed {
        let children_raw = client.ls_with_stat(&path).await?;
        let children: Vec<ChildInfo> = children_raw
            .iter()
            .map(|(name, stat)| ChildInfo {
                name: name.clone(),
                ephemeral: stat.ephemeral_owner != 0,
                data_length: stat.data_length,
                num_children: stat.num_children,
            })
            .collect();
        Ok(LsOutput::Detailed(LsDetailedResult { path, children }))
    } else {
        let children = client.ls(&path).await?;
        Ok(LsOutput::Simple(LsResult { path, children }))
    }
}

pub fn format_human(out: &LsOutput) -> String {
    match out {
        LsOutput::Simple(r) => r.children.join("\n"),
        LsOutput::Detailed(r) => {
            let mut lines = Vec::new();
            for c in &r.children {
                let e = if c.ephemeral { "E" } else { "P" };
                lines.push(format!("{}\t{}\t{}\t{}", e, c.data_length, c.num_children, c.name));
            }
            lines.join("\n")
        }
    }
}

pub fn format_json(out: &LsOutput) -> String {
    match out {
        LsOutput::Simple(r) => serde_json::to_string_pretty(r).unwrap(),
        LsOutput::Detailed(r) => serde_json::to_string_pretty(r).unwrap(),
    }
}

fn collect_recursive<'a>(
    client: &'a ZkClientImpl,
    path: &'a str,
    result: &'a mut Vec<String>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(async move {
        let children = client.ls(path).await?;
        for child in &children {
            let full = format!("{}/{}", path, child).replace("//", "/");
            result.push(full.clone());
            collect_recursive(client, &full, result).await?;
        }
        Ok(())
    })
}
