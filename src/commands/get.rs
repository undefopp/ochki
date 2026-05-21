use crate::client::ZkClientImpl;
use crate::output::{GetResult, StatJson};
use anyhow::Result;

pub async fn run(client: &ZkClientImpl, path: &str) -> Result<GetResult> {
    let path = crate::client::normalize_path(path);
    let (data, stat) = client.get(&path).await?;
    let data_str = match String::from_utf8(data.clone()) {
        Ok(s) => s,
        Err(_) => {
            let hex: Vec<String> = data.iter().map(|b| format!("{:02x}", b)).collect();
            format!("<binary:{}:{}>", data.len(), hex.join(""))
        }
    };
    Ok(GetResult {
        path,
        data: data_str,
        data_encoding: if String::from_utf8(data).is_ok() {
            "utf8".to_string()
        } else {
            "hex".to_string()
        },
        stat: StatJson::from(stat),
    })
}

pub fn format_human(r: &GetResult) -> String {
    format!(
        "{}\n\ncZxid: {}\nmZxid: {}\ncTime: {}\nmTime: {}\nVersion: {}\ncVersion: {}\naversion: {}\nEphemeralOwner: {}\nDataLength: {}\nNumChildren: {}\npZxid: {}",
        r.data,
        r.stat.czxid, r.stat.mzxid, r.stat.ctime, r.stat.mtime,
        r.stat.version, r.stat.cversion, r.stat.aversion,
        r.stat.ephemeral_owner, r.stat.data_length, r.stat.num_children, r.stat.pzxid,
    )
}
