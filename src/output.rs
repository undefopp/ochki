use serde::Serialize;
use zookeeper_client::Stat;

#[derive(Serialize)]
pub struct StatJson {
    pub czxid: i64,
    pub mzxid: i64,
    pub ctime: i64,
    pub mtime: i64,
    pub version: i32,
    pub cversion: i32,
    pub aversion: i32,
    pub ephemeral_owner: i64,
    pub data_length: i32,
    pub num_children: i32,
    pub pzxid: i64,
}

impl From<&Stat> for StatJson {
    fn from(s: &Stat) -> Self {
        Self {
            czxid: s.czxid,
            mzxid: s.mzxid,
            ctime: s.ctime,
            mtime: s.mtime,
            version: s.version,
            cversion: s.cversion,
            aversion: s.aversion,
            ephemeral_owner: s.ephemeral_owner,
            data_length: s.data_length,
            num_children: s.num_children,
            pzxid: s.pzxid,
        }
    }
}

impl From<Stat> for StatJson {
    fn from(s: Stat) -> Self {
        Self::from(&s)
    }
}

#[derive(Serialize)]
pub struct LsResult {
    pub path: String,
    pub children: Vec<String>,
}

#[derive(Serialize)]
pub struct LsDetailedResult {
    pub path: String,
    pub children: Vec<ChildInfo>,
}

#[derive(Serialize)]
pub struct ChildInfo {
    pub name: String,
    pub ephemeral: bool,
    pub data_length: i32,
    pub num_children: i32,
}

#[derive(Serialize)]
pub struct GetResult {
    pub path: String,
    pub data: String,
    pub data_encoding: String,
    pub stat: StatJson,
}

#[derive(Serialize)]
pub struct SetResult {
    pub path: String,
    pub version: i32,
    pub stat: StatJson,
}

#[derive(Serialize)]
pub struct CreateResult {
    pub path: String,
    pub mode: String,
}

#[derive(Serialize)]
pub struct DeleteResult {
    pub path: String,
    pub recursive: bool,
}

#[derive(Serialize)]
pub struct ExistsResult {
    pub path: String,
    pub exists: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stat: Option<StatJson>,
}

#[derive(Serialize)]
pub struct TreeResult {
    pub path: String,
    pub tree: Vec<TreeEntry>,
}

#[derive(Serialize)]
pub struct TreeEntry {
    pub path: String,
    pub depth: usize,
}

#[derive(Serialize)]
pub struct FindResult {
    pub path: String,
    pub pattern: String,
    pub matches: Vec<String>,
}

#[derive(Serialize)]
pub struct CpResult {
    pub src: String,
    pub dst: String,
    pub recursive: bool,
}

#[derive(Serialize)]
pub struct MvResult {
    pub src: String,
    pub dst: String,
}

#[derive(Serialize)]
pub struct AclResult {
    pub path: String,
    pub acls: Vec<AclEntry>,
}

#[derive(Serialize)]
pub struct AclEntry {
    pub scheme: String,
    pub id: String,
    pub permissions: String,
}

#[derive(Serialize)]
pub struct HealthResult {
    pub command: String,
    pub host: String,
    pub output: String,
}

#[derive(Serialize)]
pub struct DiffResult {
    pub path: String,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub modified: Vec<String>,
}

pub enum OutputFormat {
    Human,
    Json,
}

#[allow(dead_code)]
impl OutputFormat {
    pub fn format(&self, value: &impl Serialize) -> String {
        match self {
            OutputFormat::Json => {
                serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string())
            }
            OutputFormat::Human => unreachable!("use command-specific formatting"),
        }
    }
}
