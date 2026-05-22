use anyhow::Result;
use zookeeper_client::{CreateMode, Stat};

#[derive(Clone)]
#[allow(dead_code)]
pub struct ZkClientImpl {
    inner: zookeeper_client::Client,
}

#[allow(dead_code)]
impl ZkClientImpl {
    pub async fn connect(addr: &str) -> Result<Self> {
        let inner = zookeeper_client::Client::connect(addr).await?;
        Ok(Self { inner })
    }

    pub async fn ls(&self, path: &str) -> Result<Vec<String>> {
        let children = self.inner.list_children(path).await?;
        Ok(children)
    }

    pub async fn ls_with_stat(&self, path: &str) -> Result<Vec<(String, Stat)>> {
        let (children, _stat) = self.inner.get_children(path).await?;
        let mut result = Vec::with_capacity(children.len());
        for child in &children {
            let child_path = format!("{}/{}", path, child).replace("//", "/");
            match self.inner.check_stat(&child_path).await {
                Ok(Some(stat)) => result.push((child.clone(), stat)),
                Ok(None) => continue,
                Err(_) => continue,
            }
        }
        Ok(result)
    }

    pub async fn get(&self, path: &str) -> Result<(Vec<u8>, Stat)> {
        let data = self.inner.get_data(path).await?;
        Ok(data)
    }

    pub async fn set(&self, path: &str, data: &[u8], version: Option<i32>) -> Result<Stat> {
        let stat = self.inner.set_data(path, data, version).await?;
        Ok(stat)
    }

    pub async fn create(
        &self,
        path: &str,
        data: &[u8],
        mode: CreateMode,
    ) -> Result<(Stat, String)> {
        let options = mode.with_acls(zookeeper_client::Acls::anyone_all());
        let (stat, seq) = self.inner.create(path, data, &options).await?;
        let created_path = match mode {
            CreateMode::PersistentSequential | CreateMode::EphemeralSequential => {
                format!("{}{}", path, seq)
            }
            _ => path.to_string(),
        };
        Ok((stat, created_path))
    }

    pub async fn mkdir(&self, path: &str, _data: &[u8], mode: CreateMode) -> Result<()> {
        let options = mode.with_acls(zookeeper_client::Acls::anyone_all());
        self.inner.mkdir(path, &options).await?;
        Ok(())
    }

    pub async fn delete(&self, path: &str, version: Option<i32>) -> Result<()> {
        self.inner.delete(path, version).await?;
        Ok(())
    }

    pub async fn stat(&self, path: &str) -> Result<Option<Stat>> {
        let stat = self.inner.check_stat(path).await?;
        Ok(stat)
    }

    pub async fn exists(&self, path: &str) -> Result<bool> {
        match self.inner.check_stat(path).await {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn get_acl(&self, path: &str) -> Result<(Vec<zookeeper_client::Acl>, Stat)> {
        let acl = self.inner.get_acl(path).await?;
        Ok(acl)
    }

    pub async fn set_acl(
        &self,
        path: &str,
        acl: &[zookeeper_client::Acl],
        version: Option<i32>,
    ) -> Result<()> {
        self.inner.set_acl(path, acl, version).await?;
        Ok(())
    }

    pub async fn watch(
        &self,
        path: &str,
        mode: zookeeper_client::AddWatchMode,
    ) -> Result<zookeeper_client::PersistentWatcher> {
        let watcher = self.inner.watch(path, mode).await?;
        Ok(watcher)
    }

    pub async fn watch_data(
        &self,
        path: &str,
    ) -> Result<(Vec<u8>, Stat, zookeeper_client::OneshotWatcher)> {
        let result = self.inner.get_and_watch_data(path).await?;
        Ok(result)
    }

    pub async fn watch_children(
        &self,
        path: &str,
    ) -> Result<(Vec<String>, zookeeper_client::OneshotWatcher)> {
        let result = self.inner.list_and_watch_children(path).await?;
        Ok(result)
    }

    pub async fn create_recursive(
        &self,
        path: &str,
        data: &[u8],
        mode: CreateMode,
    ) -> Result<(Stat, String)> {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = String::new();
        for (i, part) in parts.iter().enumerate() {
            current.push('/');
            current.push_str(part);
            if i == parts.len() - 1 {
                return self.create(&current, data, mode).await;
            }
            if !self.exists(&current).await? {
                self.create(&current, b"", CreateMode::Persistent).await?;
            }
        }
        anyhow::bail!("empty path")
    }

    pub fn inner(&self) -> &zookeeper_client::Client {
        &self.inner
    }

    pub async fn addauth(&self, scheme: &str, credential: &str) -> Result<()> {
        self.inner.auth(scheme, credential.as_bytes()).await?;
        Ok(())
    }
}

pub fn normalize_path(path: &str) -> String {
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    }
}
