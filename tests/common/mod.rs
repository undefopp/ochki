use ochk::client::ZkClientImpl;
use testcontainers_modules::testcontainers::{self, runners::AsyncRunner, ImageExt};
use testcontainers_modules::zookeeper::Zookeeper;

pub struct ZkFixture {
    pub client: ZkClientImpl,
    pub container: testcontainers::ContainerAsync<Zookeeper>,
}

impl ZkFixture {
    pub async fn setup() -> Self {
        let container = Zookeeper::default()
            .with_env_var("ZOO_4LW_COMMANDS_WHITELIST", "ruok")
            .start()
            .await
            .expect("Failed to start ZK container");
        let host = container.get_host().await.expect("Failed to get host");
        let port = container
            .get_host_port_ipv4(2181)
            .await
            .expect("Failed to get port");
        let addr = format!("{}:{}", host, port);
        let client = ZkClientImpl::connect(&addr)
            .await
            .expect("Failed to connect to ZK");
        Self { client, container }
    }
}
