use crate::client::ZkClientImpl;
use crate::commands::dump::DumpNode;
use anyhow::Result;
use zookeeper_client::CreateMode;

pub async fn run(client: &ZkClientImpl, root: &str, file: &str, overwrite: bool) -> Result<String> {
    let root = crate::client::normalize_path(root);
    let content = std::fs::read_to_string(file)?;
    let tree: DumpNode = serde_json::from_str(&content)?;
    let mut count = 0usize;
    load_node(client, &root, &tree, overwrite, &mut count).await?;
    Ok(format!("Loaded {} nodes into {}", count, root))
}

fn load_node<'a>(
    client: &'a ZkClientImpl,
    root: &'a str,
    node: &'a DumpNode,
    overwrite: bool,
    count: &'a mut usize,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(async move {
        let data = base64_decode(&node.data)?;
        let path = format!("{}/{}", root, node.path).replace("//", "/");

        if overwrite && client.exists(&path).await.unwrap_or(false) {
            client.set(&path, &data, None).await?;
        } else {
            match client
                .create_recursive(&path, &data, CreateMode::Persistent)
                .await
            {
                Ok(_) => {}
                Err(_) if overwrite => {
                    client.set(&path, &data, None).await?;
                }
                Err(e) => return Err(e),
            }
        }
        *count += 1;

        for child in &node.children {
            load_node(client, root, child, overwrite, count).await?;
        }
        Ok(())
    })
}

fn base64_decode(input: &str) -> Result<Vec<u8>> {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut lookup = [0u8; 256];
    for (i, &c) in CHARS.iter().enumerate() {
        lookup[c as usize] = i as u8;
    }
    let input = input.trim_end_matches('=');
    let mut result = Vec::with_capacity(input.len() * 3 / 4);
    let bytes = input.as_bytes();
    for chunk in bytes.chunks(4) {
        let b0 = lookup[chunk[0] as usize] as u32;
        let b1 = if chunk.len() > 1 {
            lookup[chunk[1] as usize] as u32
        } else {
            0
        };
        let b2 = if chunk.len() > 2 {
            lookup[chunk[2] as usize] as u32
        } else {
            0
        };
        let b3 = if chunk.len() > 3 {
            lookup[chunk[3] as usize] as u32
        } else {
            0
        };
        let triple = (b0 << 18) | (b1 << 12) | (b2 << 6) | b3;
        result.push((triple >> 16) as u8);
        if chunk.len() > 2 {
            result.push((triple >> 8) as u8);
        }
        if chunk.len() > 3 {
            result.push(triple as u8);
        }
    }
    Ok(result)
}
