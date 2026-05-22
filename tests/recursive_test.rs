mod common;

use common::ZkFixture;

#[tokio::test]
async fn test_create_recursive() {
    let f = ZkFixture::setup().await;
    let r = ochki::commands::create::run(&f.client, "/a/b/c/d", Some("deep"), false, false, true)
        .await
        .expect("create recursive failed");
    assert_eq!(r.path, "/a/b/c/d");

    let r = ochki::commands::ls::run(&f.client, "/a/b/c", false, false)
        .await
        .expect("ls failed");
    match r {
        ochki::commands::ls::LsOutput::Simple(s) => assert!(s.children.contains(&"d".to_string())),
        _ => panic!("expected simple ls"),
    }
}

#[tokio::test]
async fn test_delete_recursive() {
    let f = ZkFixture::setup().await;
    ochki::commands::create::run(&f.client, "/r/a/b", Some("x"), false, false, true)
        .await
        .expect("setup failed");

    let r = ochki::commands::delete::run(&f.client, "/r", true)
        .await
        .expect("delete recursive failed");
    assert!(r.recursive);

    let exists = f.client.exists("/r").await.expect("exists failed");
    assert!(!exists);
}

#[tokio::test]
async fn test_ls_recursive() {
    let f = ZkFixture::setup().await;
    ochki::commands::create::run(&f.client, "/t/a", None, false, false, true)
        .await
        .expect("setup failed");
    ochki::commands::create::run(&f.client, "/t/b", None, false, false, false)
        .await
        .expect("setup failed");

    let r = ochki::commands::ls::run(&f.client, "/t", true, false)
        .await
        .expect("ls recursive failed");
    match r {
        ochki::commands::ls::LsOutput::Simple(s) => {
            assert!(s.children.iter().any(|c| c == "/t/a"));
            assert!(s.children.iter().any(|c| c == "/t/b"));
        }
        _ => panic!("expected simple ls"),
    }
}

#[tokio::test]
async fn test_ls_detailed() {
    let f = ZkFixture::setup().await;
    ochki::commands::create::run(&f.client, "/d1", Some("data"), false, false, false)
        .await
        .expect("setup failed");
    ochki::commands::create::run(&f.client, "/d2", None, true, false, false)
        .await
        .expect("setup failed");

    let r = ochki::commands::ls::run(&f.client, "/", false, true)
        .await
        .expect("ls detailed failed");
    match r {
        ochki::commands::ls::LsOutput::Detailed(d) => {
            let d1 = d
                .children
                .iter()
                .find(|c| c.name == "d1")
                .expect("d1 not found");
            assert!(!d1.ephemeral);
            assert_eq!(d1.data_length, 4);

            let d2 = d
                .children
                .iter()
                .find(|c| c.name == "d2")
                .expect("d2 not found");
            assert!(d2.ephemeral);
        }
        _ => panic!("expected detailed ls"),
    }
}

#[tokio::test]
async fn test_tree() {
    let f = ZkFixture::setup().await;
    ochki::commands::create::run(&f.client, "/tr/a/b", None, false, false, true)
        .await
        .expect("setup failed");

    let r = ochki::commands::tree::run(&f.client, "/tr", None)
        .await
        .expect("tree failed");
    assert_eq!(r.tree.len(), 3);
    assert_eq!(r.tree[0].path, "/tr");
    assert_eq!(r.tree[0].depth, 0);
}

#[tokio::test]
async fn test_find() {
    let f = ZkFixture::setup().await;
    ochki::commands::create::run(&f.client, "/f/alpha", None, false, false, true)
        .await
        .expect("setup failed");
    ochki::commands::create::run(&f.client, "/f/beta", None, false, false, false)
        .await
        .expect("setup failed");

    let r = ochki::commands::find::run(&f.client, "/f", "al.*", None)
        .await
        .expect("find failed");
    assert_eq!(r.matches.len(), 1);
    assert!(r.matches[0].ends_with("/alpha"));
}

#[tokio::test]
async fn test_cp_recursive() {
    let f = ZkFixture::setup().await;
    ochki::commands::create::run(&f.client, "/src/a", Some("data-a"), false, false, true)
        .await
        .expect("setup failed");

    let r = ochki::commands::cp::run(&f.client, "/src", "/dst", true)
        .await
        .expect("cp failed");
    assert!(r.recursive);

    let r = ochki::commands::get::run(&f.client, "/dst/a")
        .await
        .expect("get failed");
    assert_eq!(r.data, "data-a");
}

#[tokio::test]
async fn test_mv() {
    let f = ZkFixture::setup().await;
    ochki::commands::create::run(&f.client, "/old", Some("moved-data"), false, false, false)
        .await
        .expect("setup failed");

    let r = ochki::commands::mv::run(&f.client, "/old", "/new", false)
        .await
        .expect("mv failed");
    assert_eq!(r.src, "/old");
    assert_eq!(r.dst, "/new");

    let r = ochki::commands::get::run(&f.client, "/new")
        .await
        .expect("get failed");
    assert_eq!(r.data, "moved-data");

    let exists = f.client.exists("/old").await.expect("exists failed");
    assert!(!exists);
}
