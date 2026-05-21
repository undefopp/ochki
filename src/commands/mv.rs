use crate::client::ZkClientImpl;
use crate::output::MvResult;
use anyhow::Result;
use zookeeper_client::CreateMode;

pub async fn run(client: &ZkClientImpl, src: &str, dst: &str, dry_run: bool) -> Result<MvResult> {
    let src = crate::client::normalize_path(src);
    let dst = crate::client::normalize_path(dst);

    if dry_run {
        eprintln!(
            "Would move {} -> {} (non-atomic: read -> create -> delete)",
            src, dst
        );
        return Ok(MvResult { src, dst });
    }

    eprintln!("Warning: mv is non-atomic (read -> create -> delete). If interrupted, data may be duplicated.");
    let (data, stat) = client.get(&src).await?;
    if stat.ephemeral_owner != 0 {
        anyhow::bail!("Cannot move ephemeral node {}", src);
    }
    client.create(&dst, &data, CreateMode::Persistent).await?;
    client.delete(&src, None).await?;
    Ok(MvResult { src, dst })
}

pub fn format_human(r: &MvResult) -> String {
    format!("Moved {} -> {}", r.src, r.dst)
}
