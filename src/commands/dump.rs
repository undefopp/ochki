use crate::client::ZkClientImpl;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct DumpNode {
    pub path: String,
    pub data: String,
    pub stat: DumpStat,
    pub children: Vec<DumpNode>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DumpStat {
    pub version: i32,
    pub cversion: i32,
    pub aversion: i32,
    pub data_length: i32,
    pub num_children: i32,
    pub ephemeral_owner: i64,
}

pub async fn run(client: &ZkClientImpl, path: &str) -> Result<String> {
    let path = crate::client::normalize_path(path);
    let tree = dump_recursive(client, &path).await?;
    Ok(serde_json::to_string_pretty(&tree)?)
}

fn dump_recursive<'a>(
    client: &'a ZkClientImpl,
    path: &'a str,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<DumpNode>> + Send + 'a>> {
    Box::pin(async move {
        let (data, stat) = client.get(path).await?;
        let data_b64 = base64_encode(&data);
        let children = client.ls(path).await?;
        let mut child_nodes = Vec::new();
        for child in &children {
            let full = format!("{}/{}", path, child).replace("//", "/");
            child_nodes.push(dump_recursive(client, &full).await?);
        }
        Ok(DumpNode {
            path: path.to_string(),
            data: data_b64,
            stat: DumpStat {
                version: stat.version,
                cversion: stat.cversion,
                aversion: stat.aversion,
                data_length: stat.data_length,
                num_children: stat.num_children,
                ephemeral_owner: stat.ephemeral_owner,
            },
            children: child_nodes,
        })
    })
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        s.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        s.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        s.push(if chunk.len() > 1 { CHARS[((triple >> 6) & 0x3F) as usize] as char } else { '=' });
        s.push(if chunk.len() > 2 { CHARS[(triple & 0x3F) as usize] as char } else { '=' });
    }
    s
}
