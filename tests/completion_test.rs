mod common;

use common::ZkFixture;
use rustyline::completion::Candidate;

#[tokio::test(flavor = "multi_thread")]
async fn test_complete_command_names() {
    let f = ZkFixture::setup().await;
    let cwd = std::sync::Arc::new(std::sync::Mutex::new("/".to_string()));
    let h = ochki::repl::ReplHelper::new(f.client.clone(), tokio::runtime::Handle::current(), cwd);

    let (start, candidates) = h.complete_path("ge", 2);
    assert_eq!(start, 0);
    let names: Vec<&str> = candidates.iter().map(|c| c.display()).collect();
    assert!(
        names.contains(&"get"),
        "should contain 'get', got: {:?}",
        names
    );

    let (start, candidates) = h.complete_path("cr", 2);
    assert_eq!(start, 0);
    let names: Vec<&str> = candidates.iter().map(|c| c.display()).collect();
    assert!(
        names.contains(&"create"),
        "should contain 'create', got: {:?}",
        names
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_complete_root_children() {
    let f = ZkFixture::setup().await;
    ochki::commands::create::run(&f.client, "/alpha", Some("a"), false, false, false)
        .await
        .unwrap();
    ochki::commands::create::run(&f.client, "/beta", Some("b"), false, false, false)
        .await
        .unwrap();
    ochki::commands::create::run(&f.client, "/alpha-sub", Some("s"), false, false, false)
        .await
        .unwrap();

    let cwd = std::sync::Arc::new(std::sync::Mutex::new("/".to_string()));
    let h = ochki::repl::ReplHelper::new(f.client.clone(), tokio::runtime::Handle::current(), cwd);

    let (start, candidates) = h.complete_path("get /al", 6);
    assert_eq!(start, 4);
    let names: Vec<&str> = candidates.iter().map(|c| c.display()).collect();
    assert!(
        names.contains(&"/alpha"),
        "should contain /alpha, got: {:?}",
        names
    );
    assert!(
        names.contains(&"/alpha-sub"),
        "should contain /alpha-sub, got: {:?}",
        names
    );
    assert!(
        !names.contains(&"/beta"),
        "should not contain /beta, got: {:?}",
        names
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_complete_subpath() {
    let f = ZkFixture::setup().await;
    ochki::commands::create::run(&f.client, "/apps/api", None, false, false, true)
        .await
        .unwrap();
    ochki::commands::create::run(&f.client, "/apps/web", None, false, false, false)
        .await
        .unwrap();

    let cwd = std::sync::Arc::new(std::sync::Mutex::new("/".to_string()));
    let h = ochki::repl::ReplHelper::new(f.client.clone(), tokio::runtime::Handle::current(), cwd);

    let (start, candidates) = h.complete_path("ls /apps/a", 9);
    assert_eq!(start, 3);
    let names: Vec<&str> = candidates.iter().map(|c| c.display()).collect();
    assert!(
        names.contains(&"/apps/api"),
        "should contain /apps/api, got: {:?}",
        names
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_complete_with_cwd() {
    let f = ZkFixture::setup().await;
    ochki::commands::create::run(&f.client, "/test/child1", None, false, false, true)
        .await
        .unwrap();
    ochki::commands::create::run(&f.client, "/test/child2", None, false, false, false)
        .await
        .unwrap();

    let cwd = std::sync::Arc::new(std::sync::Mutex::new("/test".to_string()));
    let h = ochki::repl::ReplHelper::new(f.client.clone(), tokio::runtime::Handle::current(), cwd);

    let (start, candidates) = h.complete_path("get chi", 7);
    assert_eq!(start, 4);
    let names: Vec<&str> = candidates.iter().map(|c| c.display()).collect();
    assert!(
        names.contains(&"child1"),
        "should contain child1, got: {:?}",
        names
    );
    assert!(
        names.contains(&"child2"),
        "should contain child2, got: {:?}",
        names
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_complete_dot_slash() {
    let f = ZkFixture::setup().await;
    ochki::commands::create::run(&f.client, "/test/foo", None, false, false, true)
        .await
        .unwrap();
    ochki::commands::create::run(&f.client, "/test/bar", None, false, false, false)
        .await
        .unwrap();

    let cwd = std::sync::Arc::new(std::sync::Mutex::new("/test".to_string()));
    let h = ochki::repl::ReplHelper::new(f.client.clone(), tokio::runtime::Handle::current(), cwd);

    let (start, candidates) = h.complete_path("get ./f", 7);
    assert_eq!(start, 4);
    let names: Vec<&str> = candidates.iter().map(|c| c.display()).collect();
    assert!(
        names.iter().any(|n| n.contains("foo")),
        "should contain foo, got: {:?}",
        names
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_complete_no_match() {
    let f = ZkFixture::setup().await;
    let cwd = std::sync::Arc::new(std::sync::Mutex::new("/".to_string()));
    let h = ochki::repl::ReplHelper::new(f.client.clone(), tokio::runtime::Handle::current(), cwd);

    let (start, candidates) = h.complete_path("get /zzz", 8);
    assert_eq!(start, 4);
    assert!(
        candidates.is_empty(),
        "should have no matches, got: {:?}",
        candidates
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_complete_non_path_command() {
    let f = ZkFixture::setup().await;
    let cwd = std::sync::Arc::new(std::sync::Mutex::new("/".to_string()));
    let h = ochki::repl::ReplHelper::new(f.client.clone(), tokio::runtime::Handle::current(), cwd);

    let (start, candidates) = h.complete_path("connect loc", 11);
    assert_eq!(start, 11);
    assert!(candidates.is_empty());
}
