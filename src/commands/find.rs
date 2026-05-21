use crate::client::ZkClientImpl;
use crate::output::FindResult;
use anyhow::Result;
use regex::Regex;

pub async fn run(
    client: &ZkClientImpl,
    path: &str,
    pattern: &str,
    type_filter: Option<&str>,
) -> Result<FindResult> {
    let path = crate::client::normalize_path(path);
    let re = Regex::new(pattern)?;
    let mut matches = Vec::new();
    find_recursive(client, &path, &re, &mut matches, type_filter).await?;
    Ok(FindResult {
        path,
        pattern: pattern.to_string(),
        matches,
    })
}

pub fn format_human(r: &FindResult) -> String {
    r.matches.join("\n")
}

fn find_recursive<'a>(
    client: &'a ZkClientImpl,
    path: &'a str,
    re: &'a Regex,
    matches: &'a mut Vec<String>,
    type_filter: Option<&'a str>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(async move {
        match client.stat(path).await {
            Ok(Some(stat)) => {
                let name = path.rsplit('/').next().unwrap_or(path);
                if re.is_match(name) {
                    if let Some(tf) = type_filter {
                        let is_ephemeral = stat.ephemeral_owner != 0;
                        match tf {
                            "ephemeral" if is_ephemeral => matches.push(path.to_string()),
                            "persistent" if !is_ephemeral => matches.push(path.to_string()),
                            _ => {}
                        }
                    } else {
                        matches.push(path.to_string());
                    }
                }
            }
            Ok(None) | Err(_) => return Ok(()),
        }
        if let Ok(children) = client.ls(path).await {
            for child in &children {
                let full = format!("{}/{}", path, child).replace("//", "/");
                find_recursive(client, &full, re, matches, type_filter).await?;
            }
        }
        Ok(())
    })
}
