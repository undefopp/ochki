mod common;

use common::ZkFixture;

#[tokio::test]
async fn test_large_data() {
    let f = ZkFixture::setup().await;
    let data = "x".repeat(100_000);
    ochki::commands::create::run(&f.client, "/big", Some(&data), false, false, false)
        .await
        .expect("create big node failed");

    let r = ochki::commands::get::run(&f.client, "/big")
        .await
        .expect("get big failed");
    assert_eq!(r.data.len(), 100_000);

    let json = serde_json::to_string_pretty(&r).expect("json failed");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("parse failed");
    assert_eq!(parsed["data"].as_str().unwrap().len(), 100_000);
}

#[tokio::test]
async fn test_empty_data() {
    let f = ZkFixture::setup().await;
    ochki::commands::create::run(&f.client, "/empty", None, false, false, false)
        .await
        .expect("create empty failed");

    let r = ochki::commands::get::run(&f.client, "/empty")
        .await
        .expect("get empty failed");
    assert_eq!(r.data, "");
    assert_eq!(r.stat.data_length, 0);
}

#[tokio::test]
async fn test_unicode_data() {
    let f = ZkFixture::setup().await;
    let unicode = "Привет мир 🌍 日本語テスト";
    ochki::commands::create::run(&f.client, "/unicode", Some(unicode), false, false, false)
        .await
        .expect("create unicode failed");

    let r = ochki::commands::get::run(&f.client, "/unicode")
        .await
        .expect("get unicode failed");
    assert_eq!(r.data, unicode);
}

#[tokio::test]
async fn test_special_chars_in_data() {
    let f = ZkFixture::setup().await;
    let special = "line1\nline2\ttab\rcarriage\\backslash\"quote";
    ochki::commands::create::run(&f.client, "/special", Some(special), false, false, false)
        .await
        .expect("create special failed");

    let r = ochki::commands::get::run(&f.client, "/special")
        .await
        .expect("get special failed");
    assert_eq!(r.data, special);
}

#[tokio::test]
async fn test_binary_roundtrip() {
    let f = ZkFixture::setup().await;
    let binary: Vec<u8> = (0u8..=255).collect();
    f.client
        .create("/binary", &binary, zookeeper_client::CreateMode::Persistent)
        .await
        .expect("create binary failed");

    let (data, _stat) = f.client.get("/binary").await.expect("get binary failed");
    assert_eq!(data, binary);
    assert_eq!(data.len(), 256);
}

#[tokio::test]
async fn test_version_conflict() {
    let f = ZkFixture::setup().await;
    ochki::commands::create::run(&f.client, "/ver", Some("v1"), false, false, false)
        .await
        .expect("create failed");

    ochki::commands::set::run(&f.client, "/ver", "v2", None)
        .await
        .expect("set v2 failed");

    let result = ochki::commands::set::run(&f.client, "/ver", "v3", Some(0)).await;
    assert!(result.is_err(), "should fail with version mismatch");
}

#[tokio::test]
async fn test_delete_nonempty_fails() {
    let f = ZkFixture::setup().await;
    ochki::commands::create::run(&f.client, "/parent/child", None, false, false, true)
        .await
        .expect("setup failed");

    let result = ochki::commands::delete::run(&f.client, "/parent", false).await;
    assert!(result.is_err(), "should fail - node has children");
}

#[tokio::test]
async fn test_create_already_exists() {
    let f = ZkFixture::setup().await;
    ochki::commands::create::run(&f.client, "/dup", None, false, false, false)
        .await
        .expect("first create failed");

    let result = ochki::commands::create::run(&f.client, "/dup", None, false, false, false).await;
    assert!(result.is_err(), "should fail - node already exists");
}

#[tokio::test]
async fn test_get_nonexistent() {
    let f = ZkFixture::setup().await;
    let result = ochki::commands::get::run(&f.client, "/nope").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_deep_nesting() {
    let f = ZkFixture::setup().await;
    let mut path = String::new();
    for i in 0..10 {
        path.push_str(&format!("/level{}", i));
    }
    ochki::commands::create::run(&f.client, &path, Some("deep"), false, false, true)
        .await
        .expect("create deep failed");

    let r = ochki::commands::get::run(&f.client, &path)
        .await
        .expect("get deep failed");
    assert_eq!(r.data, "deep");

    let r = ochki::commands::tree::run(&f.client, "/level0", None)
        .await
        .expect("tree failed");
    assert_eq!(r.tree.len(), 10);
}

#[tokio::test]
async fn test_sequential_numbering() {
    let f = ZkFixture::setup().await;
    let mut paths = Vec::new();
    for i in 0..5 {
        let r = ochki::commands::create::run(
            &f.client,
            "/seq-node",
            Some(&format!("{}", i)),
            false,
            true,
            false,
        )
        .await
        .unwrap_or_else(|_| panic!("create seq {} failed", i));
        paths.push(r.path.clone());
    }

    assert_eq!(paths.len(), 5);
    for i in 1..5 {
        assert_ne!(paths[i - 1], paths[i], "sequential paths should differ");
    }

    let sorted = {
        let mut p = paths.clone();
        p.sort();
        p
    };
    assert_eq!(paths, sorted, "sequential paths should be ordered");
}

#[tokio::test]
async fn test_many_children() {
    let f = ZkFixture::setup().await;
    for i in 0..100 {
        let name = format!("/many/c{:03}", i);
        ochki::commands::create::run(&f.client, &name, None, false, false, i == 0)
            .await
            .unwrap_or_else(|_| panic!("create child {} failed", i));
    }

    let r = ochki::commands::ls::run(&f.client, "/many", false, false)
        .await
        .expect("ls failed");
    match r {
        ochki::commands::ls::LsOutput::Simple(s) => assert_eq!(s.children.len(), 100),
        _ => panic!("expected simple ls"),
    }
}

#[tokio::test]
async fn test_json_binary_data() {
    let f = ZkFixture::setup().await;
    let binary: Vec<u8> = vec![0x00, 0x01, 0x02, 0xFF, 0xFE];
    f.client
        .create("/jbin", &binary, zookeeper_client::CreateMode::Persistent)
        .await
        .expect("create failed");

    let r = ochki::commands::get::run(&f.client, "/jbin")
        .await
        .expect("get failed");
    let json = serde_json::to_string_pretty(&r).expect("json failed");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("parse failed");
    assert_eq!(parsed["data_encoding"], "hex");
    assert!(parsed["data"].as_str().unwrap().contains("<binary:"));
}
