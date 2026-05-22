mod common;

use common::ZkFixture;

#[tokio::test]
async fn test_create_and_get() {
    let f = ZkFixture::setup().await;
    let r = ochki::commands::create::run(&f.client, "/test", Some("hello"), false, false, false)
        .await
        .expect("create failed");
    assert_eq!(r.path, "/test");

    let r = ochki::commands::get::run(&f.client, "/test")
        .await
        .expect("get failed");
    assert_eq!(r.data, "hello");
    assert_eq!(r.stat.data_length, 5);
    assert_eq!(r.stat.version, 0);
}

#[tokio::test]
async fn test_set_updates_version() {
    let f = ZkFixture::setup().await;
    ochki::commands::create::run(&f.client, "/x", Some("v1"), false, false, false)
        .await
        .expect("create failed");

    let r = ochki::commands::set::run(&f.client, "/x", "v2", None)
        .await
        .expect("set failed");
    assert_eq!(r.version, 1);

    let r = ochki::commands::get::run(&f.client, "/x")
        .await
        .expect("get failed");
    assert_eq!(r.data, "v2");
    assert_eq!(r.stat.version, 1);
}

#[tokio::test]
async fn test_delete() {
    let f = ZkFixture::setup().await;
    ochki::commands::create::run(&f.client, "/del", None, false, false, false)
        .await
        .expect("create failed");

    let r = ochki::commands::delete::run(&f.client, "/del", false)
        .await
        .expect("delete failed");
    assert_eq!(r.path, "/del");

    let exists = f.client.exists("/del").await.expect("exists failed");
    assert!(!exists);
}

#[tokio::test]
async fn test_stat() {
    let f = ZkFixture::setup().await;
    ochki::commands::create::run(&f.client, "/s", Some("data"), false, false, false)
        .await
        .expect("create failed");

    let r = ochki::commands::stat::run(&f.client, "/s")
        .await
        .expect("stat failed");
    assert_eq!(r.data_length, 4);
    assert_eq!(r.num_children, 0);
    assert_eq!(r.version, 0);
    assert_eq!(r.ephemeral_owner, 0);
}

#[tokio::test]
async fn test_exists() {
    let f = ZkFixture::setup().await;
    ochki::commands::create::run(&f.client, "/e", None, false, false, false)
        .await
        .expect("create failed");

    let r = ochki::commands::exists::run(&f.client, "/e", false)
        .await
        .expect("exists failed");
    assert!(r.exists);

    let r = ochki::commands::exists::run(&f.client, "/nope", false)
        .await
        .expect("exists failed");
    assert!(!r.exists);
}

#[tokio::test]
async fn test_create_ephemeral() {
    let f = ZkFixture::setup().await;
    let r = ochki::commands::create::run(&f.client, "/eph", Some("tmp"), true, false, false)
        .await
        .expect("create ephemeral failed");
    assert_eq!(r.path, "/eph");
    assert_eq!(r.mode, "ephemeral");

    let r = ochki::commands::stat::run(&f.client, "/eph")
        .await
        .expect("stat failed");
    assert_ne!(r.ephemeral_owner, 0);
}

#[tokio::test]
async fn test_create_sequential() {
    let f = ZkFixture::setup().await;
    let r1 = ochki::commands::create::run(&f.client, "/seq", Some("a"), false, true, false)
        .await
        .expect("create sequential 1 failed");
    let r2 = ochki::commands::create::run(&f.client, "/seq", Some("b"), false, true, false)
        .await
        .expect("create sequential 2 failed");

    assert!(r1.path.starts_with("/seq"));
    assert!(r2.path.starts_with("/seq"));
    assert_ne!(r1.path, r2.path);
}
