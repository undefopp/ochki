use crate::client::ZkClientImpl;
use crate::style;
use anyhow::Result;

pub struct AddAuthResult {
    pub scheme: String,
}

pub async fn run(client: &ZkClientImpl, scheme: &str, credential: &str) -> Result<AddAuthResult> {
    client.addauth(scheme, credential).await?;
    Ok(AddAuthResult {
        scheme: scheme.to_string(),
    })
}

pub fn format_human(r: &AddAuthResult) -> String {
    format!("{} Authenticated via {}", style::success("\u{2713}"), style::path(&r.scheme))
}
