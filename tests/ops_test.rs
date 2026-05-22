mod common;

use common::ZkFixture;

#[tokio::test]
async fn test_dump_load_roundtrip() {
    let f = ZkFixture::setup().await;
    ochki::commands::create::run(&f.client, "/dump/a", Some("leaf"), false, false, true)
        .await
        .expect("setup failed");

    let json = ochki::commands::dump::run(&f.client, "/dump")
        .await
        .expect("dump failed");

    let parsed: serde_json::Value = serde_json::from_str(&json).expect("dump not valid json");
    assert_eq!(parsed["path"], "/dump");
    assert!(parsed["children"].is_array());

    let tmp = tempfile::NamedTempFile::new().expect("tempfile failed");
    std::fs::write(tmp.path(), &json).expect("write failed");

    let r = ochki::commands::load::run(&f.client, "/restored", tmp.path().to_str().unwrap(), false)
        .await
        .expect("load failed");
    assert!(r.contains("2 nodes"));

    let r = ochki::commands::get::run(&f.client, "/restored/dump/a")
        .await
        .expect("get restored failed");
    assert_eq!(r.data, "leaf");
}

#[tokio::test]
async fn test_acl_roundtrip() {
    let f = ZkFixture::setup().await;
    ochki::commands::create::run(&f.client, "/acl-test", None, false, false, false)
        .await
        .expect("setup failed");

    let r = ochki::commands::acl::get_acl(&f.client, "/acl-test")
        .await
        .expect("get_acl failed");
    assert!(!r.acls.is_empty());
    assert_eq!(r.acls[0].scheme, "world");
    assert!(r.acls[0].permissions.contains('r'));

    ochki::commands::acl::set_acl(&f.client, "/acl-test", "world:anyone:r", None)
        .await
        .expect("set_acl failed");

    let r = ochki::commands::acl::get_acl(&f.client, "/acl-test")
        .await
        .expect("get_acl after set failed");
    assert_eq!(r.acls[0].permissions, "r");
}

#[tokio::test]
async fn test_health_ruok() {
    let f = ZkFixture::setup().await;
    let port = f
        .container
        .get_host_port_ipv4(2181)
        .await
        .expect("port failed");
    let host = f.container.get_host().await.expect("host failed");

    let r = ochki::commands::health::run("ruok", &format!("{}:{}", host, port))
        .await
        .expect("health failed");
    assert_eq!(r.output, "imok");
}

#[tokio::test]
async fn test_diff_same_cluster() {
    let f = ZkFixture::setup().await;
    ochki::commands::create::run(&f.client, "/diff-test", Some("same"), false, false, false)
        .await
        .expect("setup failed");

    let port = f
        .container
        .get_host_port_ipv4(2181)
        .await
        .expect("port failed");
    let host = f.container.get_host().await.expect("host failed");
    let c2 = ochki::client::ZkClientImpl::connect(&format!("{}:{}", host, port))
        .await
        .expect("connect c2 failed");

    let r = ochki::commands::diff::run(&f.client, &c2, "/diff-test")
        .await
        .expect("diff failed");
    assert!(r.added.is_empty());
    assert!(r.removed.is_empty());
    assert!(r.modified.is_empty());
}

#[tokio::test]
async fn test_json_output() {
    let f = ZkFixture::setup().await;
    ochki::commands::create::run(&f.client, "/json-test", Some("data"), false, false, false)
        .await
        .expect("setup failed");

    let r = ochki::commands::get::run(&f.client, "/json-test")
        .await
        .expect("get failed");
    let json_str = serde_json::to_string_pretty(&r).expect("serialize failed");
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("parse failed");
    assert_eq!(parsed["data"], "data");
    assert_eq!(parsed["stat"]["version"], 0);

    let r = ochki::commands::exists::run(&f.client, "/json-test", false)
        .await
        .expect("exists failed");
    let json_str = serde_json::to_string_pretty(&r).expect("serialize failed");
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("parse failed");
    assert_eq!(parsed["exists"], true);
}

#[tokio::test]
async fn test_binary_data() {
    let f = ZkFixture::setup().await;
    let data: Vec<u8> = vec![0x00, 0x01, 0xFF, 0xFE];
    f.client
        .create("/bin", &data, zookeeper_client::CreateMode::Persistent)
        .await
        .expect("create binary failed");

    let r = ochki::commands::get::run(&f.client, "/bin")
        .await
        .expect("get binary failed");
    assert_eq!(r.data_encoding, "hex");
    assert!(r.data.contains("<binary:4:"));
}

#[tokio::test]
async fn test_path_normalization() {
    let f = ZkFixture::setup().await;
    let r = ochki::commands::create::run(&f.client, "norm", None, false, false, false)
        .await
        .expect("create without slash failed");
    assert_eq!(r.path, "/norm");

    let r = ochki::commands::get::run(&f.client, "norm")
        .await
        .expect("get without slash failed");
    assert_eq!(r.path, "/norm");
}
