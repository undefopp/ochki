use crate::client::ZkClientImpl;
use crate::output::{GetResult, StatJson};
use crate::style;
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
    let data_display = if r.data_encoding == "hex" {
        style::warn(&r.data).to_string()
    } else {
        r.data.clone()
    };
    format!(
        "{}\n\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        data_display,
        style::kv("cZxid", &r.stat.czxid.to_string()),
        style::kv("mZxid", &r.stat.mzxid.to_string()),
        style::kv("cTime", &r.stat.ctime.to_string()),
        style::kv("mTime", &r.stat.mtime.to_string()),
        style::kv("Version", &r.stat.version.to_string()),
        style::kv("cVersion", &r.stat.cversion.to_string()),
        style::kv("aVersion", &r.stat.aversion.to_string()),
        style::kv("EphemeralOwner", &r.stat.ephemeral_owner.to_string()),
        style::kv("DataLength", &r.stat.data_length.to_string()),
        style::kv("NumChildren", &r.stat.num_children.to_string()),
        style::kv("pZxid", &r.stat.pzxid.to_string()),
    )
}
