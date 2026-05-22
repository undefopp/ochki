# ochki — Unified ZooKeeper / ClickHouse Keeper CLI

A single Rust binary for managing ZooKeeper and ClickHouse Keeper nodes. Works as a one-shot CLI or interactive REPL with tab completion, working directory, and JSON output.

## Install

```bash
cargo install --path .
```

Requires Rust 1.76+ (stable).

## Quick Start

```bash
# One-shot commands
ochki ls /
ochki get /some/node
ochki create /test "hello" --recursive
ochki --json stat /test

# Interactive REPL
ochki
ochki:/> create /apps/api "data" --recursive
ochki:/> cd /apps
ochki:/apps> ls .
ochki:/apps> get api
ochki:/apps> connect other-host:2181
ochki:/> exit
```

## Commands

### Core (P0)

| Command | Description |
|---------|-------------|
| `ls <path> [-R] [-l]` | List children. `-R` recursive, `-l` with stat |
| `get <path>` | Get node data (UTF-8 or hex for binary) |
| `set <path> <data> [--version N]` | Set data. `--version` for CAS |
| `create <path> [data]` | Create node. `--ephemeral`, `--sequential`, `--recursive` |
| `delete <path> [-R]` | Delete node. `-R` recursive |
| `stat <path>` | Node metadata (versions, timestamps, children) |
| `exists <path>` | Check if node exists |

### Navigation (P1)

| Command | Description |
|---------|-------------|
| `tree <path> [--depth N]` | Pretty tree view |
| `find <path> <pattern> [--type ephemeral\|persistent]` | Find nodes by regex |
| `watch <path>` | Persistent watcher (prints events until Ctrl+C) |
| `cp <src> <dst> [-R]` | Copy subtree (ephemeral nodes skipped with warning) |
| `mv <src> <dst> [--dry-run]` | Move node (non-atomic: read → create → delete) |

### Operations (P2)

| Command | Description |
|---------|-------------|
| `get-acl <path>` | Get ACL list |
| `set-acl <path> <scheme:id:perms>` | Set ACL (e.g. `world:anyone:cdrwa`) |
| `add-auth <scheme> <credential>` | Authenticate (e.g. `add-auth digest user1:password`) |
| `dump <path>` | Export subtree as JSON (base64 for binary data) |
| `load <path> <file> [--overwrite]` | Import JSON subtree |
| `health ruok\|stat\|srvr\|mntr <host:port>` | 4LW health commands via raw TCP |
| `diff <path> <host1> <host2>` | Compare same path across two clusters |

### REPL-only

| Command | Description |
|---------|-------------|
| `cd <path>` | Change working directory (`.` and `..` supported) |
| `pwd` | Print working directory |
| `connect <host:port>` | Switch to different server |
| `help` | Print command reference |
| `exit` / `quit` | Exit REPL |

## JSON Output

All commands support `--json` for structured output:

```bash
ochki --json get /test
# {
#   "path": "/test",
#   "data": "hello",
#   "data_encoding": "utf-8",
#   "stat": { "version": 0, "data_length": 5, ... }
# }

ochki --json ls /
# { "path": "/", "children": ["test", "zookeeper"] }

ochki --json exists /nope
# { "path": "/nope", "exists": false }
```

## ClickHouse Keeper Compatibility

Works with ClickHouse Keeper 24.8+ via the same port 2181. A local patch of `zookeeper-client` crate fixes two incompatibilities:

1. 16-byte password in handshake (CK Keeper expects exactly 16 bytes)
2. `OpCode::Create` instead of `OpCode::Create2` (CK Keeper doesn't support Create2)

> **Note:** CK Keeper does not support 4LW commands (`health` won't work).

## Testing

42 integration tests with testcontainers (ZooKeeper 3.9):

```bash
cargo test                    # all tests
cargo test --test edge_cases  # edge cases
cargo test --test completion  # tab completion
```

Test coverage:

| Suite | Tests | Coverage |
|-------|-------|----------|
| `crud_test` | 7 | create, get, set, delete, stat, exists, ephemeral, sequential |
| `recursive_test` | 8 | create_recursive, delete_recursive, ls -R/-l, tree, find, cp, mv |
| `ops_test` | 7 | dump/load, ACL, health, diff, JSON, binary, path normalization |
| `edge_cases_test` | 13 | large data (100K), empty, unicode, special chars, binary roundtrip, version conflict, deep nesting (10 levels), sequential numbering, many children (100), error cases |
| `completion_test` | 7 | command completion, root children, subpaths, cwd-relative, `./` prefix, no match, non-path commands |

## Architecture

```
src/
├── main.rs              # entry point
├── lib.rs               # public modules
├── cli.rs               # clap CLI definition
├── client.rs            # ZkClientImpl wrapper (path normalization, recursive ops)
├── style.rs             # terminal color helpers
├── repl.rs              # REPL with rustyline, tab completion, cd, connect
├── output.rs            # JSON output structs (Serialize)
├── commands/
│   ├── ls.rs            # ls [-R] [-l]
│   ├── get.rs           # get (UTF-8 or hex)
│   ├── set.rs           # set [--version]
│   ├── create.rs        # create [--ephemeral] [--sequential] [--recursive]
│   ├── delete.rs        # delete [-R]
│   ├── stat.rs          # stat
│   ├── exists.rs        # exists
│   ├── tree.rs          # tree [--depth]
│   ├── find.rs          # find <pattern>
│   ├── watch.rs         # persistent watcher
│   ├── cp.rs            # cp [-R]
│   ├── mv.rs            # mv [--dry-run]
│   ├── acl.rs           # get-acl / set-acl
│   ├── dump.rs          # export to JSON
│   ├── load.rs          # import from JSON
│   ├── health.rs        # 4LW via raw TCP
│   └── diff.rs          # compare two clusters
tests/
├── common/mod.rs            # testcontainers ZK fixture
├── crud_test.rs             # core CRUD tests
├── recursive_test.rs        # recursive operations tests
├── ops_test.rs              # dump/load, ACL, health, diff tests
├── edge_cases_test.rs       # edge cases and error handling
└── completion_test.rs       # tab completion tests
```

## Stack

| Dependency | Purpose |
|------------|---------|
| `zookeeper-client` 0.11 (patched) | ZK wire protocol |
| `clap` 4 (derive) | CLI parsing |
| `rustyline` 18 | REPL with history |
| `tokio` 1 | Async runtime |
| `anyhow` 1 | Error handling |
| `serde` + `serde_json` 1 | JSON serialization |
| `regex` 1 | `find` patterns |
| `colored` 2 | Terminal colors |
| `ctrlc` 3 | Graceful Ctrl+C in REPL |

Dev: `testcontainers-modules` 0.15, `tempfile` 3

## Constraints

| Decision | Choice |
|----------|--------|
| TLS/SASL | Not in scope (plaintext only) |
| Binary data | UTF-8 string or hex dump |
| `mv` atomicity | Non-atomic (warning printed) |
| `cp` ephemeral | Skip with warning |
| Default ACL | `world:anyone:cdrwa` |
| Rust version | Stable 1.76+ |
