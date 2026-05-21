use crate::client::ZkClientImpl;
use crate::output::DeleteResult;
use anyhow::Result;

pub async fn run(client: &ZkClientImpl, path: &str, recursive: bool) -> Result<DeleteResult> {
    let path = crate::client::normalize_path(path);
    if recursive {
        delete_recursive(client, &path).await?;
    } else {
        client.delete(&path, None).await?;
    }
    Ok(DeleteResult {
        path,
        recursive,
    })
}

pub fn format_human(r: &DeleteResult) -> String {
    if r.recursive {
        format!("Deleted {} (recursive)", r.path)
    } else {
        format!("Deleted {}", r.path)
    }
}

fn delete_recursive<'a>(
    client: &'a ZkClientImpl,
    path: &'a str,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(async move {
        let children = client.ls(path).await?;
        for child in &children {
            let child_path = format!("{}/{}", path, child).replace("//", "/");
            delete_recursive(client, &child_path).await?;
        }
        client.delete(path, None).await
    })
}
