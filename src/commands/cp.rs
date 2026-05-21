use crate::client::ZkClientImpl;
use crate::output::CpResult;
use anyhow::Result;
use zookeeper_client::CreateMode;

pub async fn run(client: &ZkClientImpl, src: &str, dst: &str, recursive: bool) -> Result<CpResult> {
    let src = crate::client::normalize_path(src);
    let dst = crate::client::normalize_path(dst);

    if recursive {
        copy_recursive(client, &src, &dst).await?;
    } else {
        copy_single(client, &src, &dst).await?;
    }
    Ok(CpResult {
        src,
        dst,
        recursive,
    })
}

pub fn format_human(r: &CpResult) -> String {
    if r.recursive {
        format!("Copied {} -> {} (recursive)", r.src, r.dst)
    } else {
        format!("Copied {} -> {}", r.src, r.dst)
    }
}

async fn copy_single(client: &ZkClientImpl, src: &str, dst: &str) -> Result<()> {
    let (data, stat) = client.get(src).await?;
    if stat.ephemeral_owner != 0 {
        eprintln!("Warning: {} is ephemeral, copying as persistent", src);
    }
    client.create(dst, &data, CreateMode::Persistent).await?;
    Ok(())
}

fn copy_recursive<'a>(
    client: &'a ZkClientImpl,
    src: &'a str,
    dst: &'a str,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(async move {
        let (data, stat) = client.get(src).await?;
        if stat.ephemeral_owner != 0 {
            eprintln!("Warning: {} is ephemeral, copying as persistent", src);
        }
        client.create(dst, &data, CreateMode::Persistent).await?;

        let children = client.ls(src).await?;
        for child in &children {
            let src_child = format!("{}/{}", src, child).replace("//", "/");
            let dst_child = format!("{}/{}", dst, child).replace("//", "/");
            copy_recursive(client, &src_child, &dst_child).await?;
        }
        Ok(())
    })
}
