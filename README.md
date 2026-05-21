# ochk — Unified ZK/ClickHouse Keeper CLI

Rust CLI tool for unified access to ZooKeeper and ClickHouse Keeper via port 2181.

## Stack

| Crate | Version | Purpose |
|-------|---------|---------|
| zookeeper-client | 0.11 (tokio feature) | ZK wire protocol |
| clap | 4 (derive) | CLI arg parsing |
| rustyline | 18 | REPL with history/completion |
| tokio | 1 (rt-multi-thread,macros,net,time,io-util,sync) | Async runtime |
| anyhow | 1 | Error handling (CLI) |
| serde + serde_json | 1 (derive) | JSON output / dump/load |
| colored | 2 | Human-readable output |
| regex | 1 | `find` command patterns |

Dev deps: `testcontainers-modules 0.15 (zookeeper)`, `tempfile 3`, `assert_cmd 2`

## Commands

### P0 — Core
- `ls <path>` — list children (`-R` recursive, `-l` detailed)
- `get <path>` — node data (UTF-8 string, hex for binary)
- `set <path> <data>` — set data (`--version N` for CAS)
- `create <path> [data]` — `--ephemeral`, `--sequential`, `--recursive` (mkdir -p)
- `delete <path>` — `--recursive` (deleteall)
- `stat <path>` — metadata (versions, timestamps, children count)
- `exists <path>` — exit code 0/1

### P1 — Navigation
- `tree <path>` — pretty tree view (`--depth N`)
- `find <path> <pattern>` — regex pattern match
- `watch <path>` — persistent watcher (REPL: async events, `unwatch` to cancel; one-shot: first event then exit)
- `cp <src> <dst>` — `--recursive` (ephemeral nodes skipped with warning)
- `mv <src> <dst>` — non-atomic: read → create → delete (warning printed)

### P2 — Operations
- `getacl <path>` / `setacl <path> <scheme:id:perms>` — ACL management
- `dump <path>` — export subtree as JSON (base64 for binary data)
- `load <path> <file>` — import JSON subtree (`--overwrite` for conflicts)
- `health <host:port>` — 4LW commands via raw TCP (`ruok`, `stat`, `srvr`, `mntr`)
- `diff <path> <host1> <host2>` — compare same path across two clusters

## Usage

```bash
# One-shot
ochk -s 127.0.0.1:2181 ls /
ochk -s 127.0.0.1:2181 --json get /some/node

# REPL
ochk -s 127.0.0.1:2181
ochk(127.0.0.1:2181)> ls /
ochk(127.0.0.1:2181)> create /test "hello"
ochk(127.0.0.1:2181)> exit
```

## Architecture

```
src/
├── main.rs                # entry point, tokio::main
├── cli.rs                 # clap Cli struct + Commands enum
├── client.rs              # ZkClient trait (frozen before parallel work)
├── repl.rs                # rustyline loop + tokio bridge (mpsc channels)
├── output.rs              # human-readable + JSON formatting
├── four_letter_word.rs    # raw TCP 4LW for health command
├── commands/
│   ├── mod.rs
│   ├── ls.rs
│   ├── get.rs
│   ├── set.rs
│   ├── create.rs
│   ├── delete.rs
│   ├── stat.rs
│   ├── exists.rs
│   ├── tree.rs
│   ├── find.rs
│   ├── watch.rs
│   ├── cp.rs
│   ├── mv.rs
│   ├── acl.rs
│   ├── dump.rs
│   ├── load.rs
│   ├── health.rs
│   └── diff.rs
tests/
├── common/
│   └── mod.rs             # testcontainers fixtures (ZK + CK Keeper)
├── ls_test.rs ... diff_test.rs
```

### REPL ↔ Tokio Bridge

Readline runs in a dedicated `std::thread::spawn`. Commands are sent to the tokio runtime via `tokio::sync::mpsc` channel. Watch events flow back through a separate channel; the readline thread drains it before each prompt.

### ZkClient Trait

Frozen in Phase 2 with stubs. All commands code against this interface:

```rust
trait ZkClient {
    async fn ls(&self, path: &str) -> Result<Vec<String>>;
    async fn get(&self, path: &str) -> Result<(Vec<u8>, Stat)>;
    async fn set(&self, path: &str, data: &[u8], version: Option<i32>) -> Result<Stat>;
    async fn create(&self, path: &str, data: &[u8], mode: CreateMode, acl: &[Acl]) -> Result<String>;
    async fn delete(&self, path: &str, version: Option<i32>) -> Result<()>;
    async fn stat(&self, path: &str) -> Result<Stat>;
    async fn exists(&self, path: &str) -> Result<bool>;
    async fn get_acl(&self, path: &str) -> Result<(Vec<Acl>, Stat)>;
    async fn set_acl(&self, path: &str, acl: &[Acl], version: Option<i32>) -> Result<()>;
    async fn watch(&self, path: &str, mode: AddWatchMode) -> Result<PersistentWatcher>;
    fn clone_client(&self) -> Self;
}
```

### CLI → API Mapping

| CLI command | zookeeper-client method | Notes |
|-------------|------------------------|-------|
| `ls` | `list_children` / `get_children` | Two variants: names only vs names+stat |
| `get` | `get_data` | Returns (Vec\<u8\>, Stat) |
| `set` | `set_data` | version: None = unconditional |
| `create` | `create(path, data, &CreateOptions)` | CreateOptions wraps CreateMode + Acl |
| `create --recursive` | Walk segments, create missing | Not a single API call |
| `delete` | `delete(path, version)` | version: None = unconditional |
| `stat` | `check_stat` | Returns Stat |
| `exists` | `check_stat` → Ok/Err | Map NoNode error to false |
| `watch` | `get_and_watch_data` / `watch(path, mode)` | Oneshot vs Persistent |
| `getacl` | `get_acl` | Returns (Vec\<Acl\>, Stat) |
| `setacl` | `set_acl` | ACL format: `scheme:id:perms` |
| `health` | Raw TCP 4LW | NOT via zookeeper-client |

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Node not found |
| 2 | Connection error |
| 3 | Bad arguments |

## Implementation Phases

### Phase 1: Scaffold (1 task)
- `cargo init`, Cargo.toml with all deps, `src/main.rs`, `src/cli.rs` stub
- Verify: `cargo build`, `ochk --help`, `ochk --version`

### Phase 2: Core Infrastructure (3 tasks, sequential)

**2.1: ZkClient trait + API mapping + connection lifecycle**
- Define frozen `ZkClient` trait with all methods
- Stub implementation wrapping `zookeeper_client::Client`
- Path normalization (ensure leading `/`)
- Connection timeout: 10s. Session timeout: 30s. Reconnection: handled by crate internally.
- Default ACL: `Acls::OPEN`
- Compile: all methods return `todo!()` or simple delegation

**2.2: Testcontainers fixtures**
- `tests/common/mod.rs`: start ZK container + return connection string
- Configure `4lw.commands.whitelist=*` for health tests
- CK Keeper fixture: `clickhouse/clickhouse-keeper` image with Keeper config on port 2181 (if image available; otherwise ZK-only)
- Helper: `test_client() -> (Container, ZkClient)`

**2.3: REPL engine + tokio bridge**
- `src/repl.rs`: rustyline in `std::thread::spawn`
- Command channel: `mpsc::Sender<String>` → tokio runtime
- Result channel: `mpsc::Receiver<Result<String>>` → readline thread
- Watch event channel: `mpsc::Receiver<WatchedEvent>` → drain before prompt
- Reuse clap parsing: `Cli::try_parse_from(["ochk", ...input.split()])` with REPL-specific error munging
- History: `~/.ochk_history`
- Prompt: `ochk(<connection>)> `
- Built-in: `exit`, `quit`, `help`

### Phase 3: P0 Commands (7 tasks, parallel after Phase 2)

Each: RED (failing test) → GREEN (implement) → REFACTOR

| Task | Command | Key Details |
|------|---------|-------------|
| 3.1 | `ls` | BFS for `-R`, stat per child for `-l`, column/table output |
| 3.2 | `get` | UTF-8 or hex dump, show stat alongside |
| 3.3 | `set` | `--version N` for CAS, print updated stat |
| 3.4 | `create` | `--recursive`: walk segments. `CreateOptions` with mode + OPEN ACL |
| 3.5 | `delete` | `--recursive`: BFS collect, delete leaves-first |
| 3.6 | `stat` | Display all Stat fields (czxid, mzxid, ctime, mtime, versions, ephemeralOwner, dataLength, numChildren, pzxid) |
| 3.7 | `exists` | Map `check_stat` Ok→true/Err(NoNode)→false. Exit 0/1. `--verbose` prints stat |

### Phase 4: P1 Commands (5 tasks, parallel after Phase 3)

| Task | Command | Key Details |
|------|---------|-------------|
| 4.1 | `tree` | Indented tree, `--depth N` limit |
| 4.2 | `find` | Regex via `regex` crate. `--type ephemeral/persistent` filter |
| 4.3 | `watch` | REPL: `PersistentWatcher`, async event print, `unwatch <path>`. One-shot: `OneshotWatcher`, print event, exit |
| 4.4 | `cp` | Recursive: BFS get+create. Ephemeral → skip with warning. ACLs preserved. |
| 4.5 | `mv` | Non-atomic (warning printed). read→create→delete. `--dry-run` flag. |

### Phase 5: P2 Commands (6 tasks, parallel after Phase 3)

| Task | Command | Key Details |
|------|---------|-------------|
| 5.1 | `getacl` / `setacl` | Display: `scheme:id:perms`. Set: parse `scheme:id:perms` string |
| 5.2 | `dump` | JSON: `{ path, data (base64 for binary), stat, acl, children: [...] }` |
| 5.3 | `load` | Parse JSON, validate structure, create nodes. `--overwrite` for conflicts |
| 5.4 | `health` | Raw TCP to port 2181. Subcommands: `ruok`, `stat`, `srvr`, `mntr` |
| 5.5 | `diff` | Two `ZkClient` instances. Compare children + data. Output: added/removed/modified |
| 5.6 | Output polish | Color output, `--no-color`, `--quiet` (pipe-friendly), JSON validation tests |

### Phase 6: Polish (2 tasks)

**6.1: REPL polish**
- Tab completion: path-based via `list_children`
- Session state: show connected/disconnected/expired in prompt
- Ctrl+C graceful handling (abort current command, not REPL)
- Per-command help

**6.2: README + integration test**
- Replace this plan with proper user-facing README
- End-to-end test: full one-shot + REPL session against testcontainers ZK

## Constraints & Decisions

| Decision | Choice |
|----------|--------|
| TLS/SASL | NOT in scope. Plaintext only. Deferred to later. |
| Binary data display | UTF-8 string or hex dump |
| `mv` atomicity | Non-atomic, warning printed |
| `cp` ephemeral nodes | Skip with warning |
| Default ACL | `Acls::OPEN` (world:anyone:cdrwa) |
| `delete` versioning | `None` (unconditional), no `--version` flag |
| `find` pattern syntax | Regex |
| History file | `~/.ochk_history` |
| CK Keeper localhost bug | Use `127.0.0.1` not `localhost` |
| Crate publication | Local binary only, not on crates.io |
| Rust version | Stable, MSRV 1.76 |

## Gotchas (from research)

1. `zookeeper-client` is runtime-agnostic — must enable `tokio` feature
2. `rustyline` is synchronous — bridge via channels, not async
3. `testcontainers-modules` re-exports testcontainers — don't depend on both
4. ClickHouse Keeper has a known bug with `localhost` — use `127.0.0.1`
5. ZK 3.5+ needs `4lw.commands.whitelist=*` for 4LW commands
6. `serde` needs `features = ["derive"]` for `#[derive(Serialize, Deserialize)]`
7. `create --recursive` is NOT a single API call — must walk path segments
8. `health` uses raw TCP, NOT the zookeeper-client library
