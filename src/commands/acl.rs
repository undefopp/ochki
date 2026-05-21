use crate::client::ZkClientImpl;
use crate::output::{AclEntry, AclResult};
use anyhow::Result;
use zookeeper_client::{Acl, AuthId, Permission};

pub async fn get_acl(client: &ZkClientImpl, path: &str) -> Result<AclResult> {
    let path = crate::client::normalize_path(path);
    let (acls, _stat) = client.get_acl(&path).await?;
    let entries: Vec<AclEntry> = acls
        .iter()
        .map(|acl| AclEntry {
            scheme: acl.scheme().to_string(),
            id: acl.id().to_string(),
            permissions: format_perms(acl.permission()),
        })
        .collect();
    Ok(AclResult {
        path,
        acls: entries,
    })
}

pub async fn set_acl(
    client: &ZkClientImpl,
    path: &str,
    acl_str: &str,
    version: Option<i32>,
) -> Result<AclResult> {
    let path = crate::client::normalize_path(path);
    let acl = parse_acl(acl_str)?;
    let acl_entry = AclEntry {
        scheme: acl.scheme().to_string(),
        id: acl.id().to_string(),
        permissions: format_perms(acl.permission()),
    };
    client.set_acl(&path, &[acl], version).await?;
    Ok(AclResult {
        path,
        acls: vec![acl_entry],
    })
}

pub fn format_human(r: &AclResult) -> String {
    r.acls
        .iter()
        .map(|a| format!("{}:{}:{}", a.scheme, a.id, a.permissions))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_perms(perms: Permission) -> String {
    let mut s = String::new();
    if perms.has(Permission::CREATE) {
        s.push('c');
    }
    if perms.has(Permission::DELETE) {
        s.push('d');
    }
    if perms.has(Permission::READ) {
        s.push('r');
    }
    if perms.has(Permission::WRITE) {
        s.push('w');
    }
    if perms.has(Permission::ADMIN) {
        s.push('a');
    }
    s
}

fn parse_acl(s: &str) -> Result<Acl> {
    let parts: Vec<&str> = s.splitn(3, ':').collect();
    if parts.len() != 3 {
        anyhow::bail!("ACL format: scheme:id:perms (e.g. world:anyone:cdrwa)");
    }
    let perms = parse_perms(parts[2])?;
    let id = AuthId::new(parts[0], parts[1]);
    Ok(Acl::new(perms, id))
}

fn parse_perms(s: &str) -> Result<Permission> {
    let mut perms = Permission::NONE;
    for c in s.chars() {
        match c {
            'c' => perms = perms | Permission::CREATE,
            'd' => perms = perms | Permission::DELETE,
            'r' => perms = perms | Permission::READ,
            'w' => perms = perms | Permission::WRITE,
            'a' => perms = perms | Permission::ADMIN,
            _ => anyhow::bail!("Invalid permission char: '{}'. Use: cdrwa", c),
        }
    }
    Ok(perms)
}
