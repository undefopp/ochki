use crate::cli::{Cli, Commands, HealthCommand};
use crate::client::ZkClientImpl;
use crate::output::OutputFormat;
use anyhow::Result;
use clap::Parser;
use rustyline::completion::{Completer, Candidate};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::Validator;
use rustyline::Helper;

pub async fn run_once(client: ZkClientImpl, cmd: Commands, json: bool) -> Result<()> {
    let fmt = if json { OutputFormat::Json } else { OutputFormat::Human };
    dispatch(&client, &cmd, &fmt, "/").await?;
    Ok(())
}

pub async fn dispatch(client: &ZkClientImpl, cmd: &Commands, fmt: &OutputFormat, cwd: &str) -> Result<()> {
    match cmd {
        Commands::Ls { path, recursive, detailed } => {
            let path = resolve_path(cwd, path);
            let out = crate::commands::ls::run(client, &path, *recursive, *detailed).await?;
            match fmt {
                OutputFormat::Json => println!("{}", crate::commands::ls::format_json(&out)),
                OutputFormat::Human => println!("{}", crate::commands::ls::format_human(&out)),
            }
        }
        Commands::Get { path } => {
            let path = resolve_path(cwd, path);
            let out = crate::commands::get::run(client, &path).await?;
            match fmt {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
                OutputFormat::Human => println!("{}", crate::commands::get::format_human(&out)),
            }
        }
        Commands::Set { path, data, version } => {
            let path = resolve_path(cwd, path);
            let out = crate::commands::set::run(client, &path, data, *version).await?;
            match fmt {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
                OutputFormat::Human => println!("{}", crate::commands::set::format_human(&out)),
            }
        }
        Commands::Create { path, data, ephemeral, sequential, recursive } => {
            let path = resolve_path(cwd, path);
            let out = crate::commands::create::run(client, &path, data.as_deref(), *ephemeral, *sequential, *recursive).await?;
            match fmt {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
                OutputFormat::Human => println!("{}", crate::commands::create::format_human(&out)),
            }
        }
        Commands::Delete { path, recursive } => {
            let path = resolve_path(cwd, path);
            let out = crate::commands::delete::run(client, &path, *recursive).await?;
            match fmt {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
                OutputFormat::Human => println!("{}", crate::commands::delete::format_human(&out)),
            }
        }
        Commands::Stat { path } => {
            let path = resolve_path(cwd, path);
            let out = crate::commands::stat::run(client, &path).await?;
            match fmt {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
                OutputFormat::Human => println!("{}", crate::commands::stat::format_human(&out)),
            }
        }
        Commands::Exists { path, verbose } => {
            let path = resolve_path(cwd, path);
            let out = crate::commands::exists::run(client, &path, *verbose).await?;
            match fmt {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
                OutputFormat::Human => {
                    if let Some(detail) = crate::commands::exists::format_human(&out) {
                        println!("{}", detail);
                    }
                    if !out.exists {
                        eprintln!("Node does not exist");
                    }
                }
            }
        }
        Commands::Tree { path, depth } => {
            let path = resolve_path(cwd, path);
            let out = crate::commands::tree::run(client, &path, *depth).await?;
            match fmt {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
                OutputFormat::Human => println!("{}", crate::commands::tree::format_human(&out)),
            }
        }
        Commands::Find { path, pattern, r#type } => {
            let path = resolve_path(cwd, path);
            let out = crate::commands::find::run(client, &path, pattern, r#type.as_deref()).await?;
            match fmt {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
                OutputFormat::Human => println!("{}", crate::commands::find::format_human(&out)),
            }
        }
        Commands::Watch { path } => {
            let path = resolve_path(cwd, path);
            crate::commands::watch::run_persistent(client, &path).await?;
        }
        Commands::Cp { src, dst, recursive } => {
            let src = resolve_path(cwd, src);
            let dst = resolve_path(cwd, dst);
            let out = crate::commands::cp::run(client, &src, &dst, *recursive).await?;
            match fmt {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
                OutputFormat::Human => println!("{}", crate::commands::cp::format_human(&out)),
            }
        }
        Commands::Mv { src, dst, dry_run } => {
            let src = resolve_path(cwd, src);
            let dst = resolve_path(cwd, dst);
            let out = crate::commands::mv::run(client, &src, &dst, *dry_run).await?;
            match fmt {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
                OutputFormat::Human => println!("{}", crate::commands::mv::format_human(&out)),
            }
        }
        Commands::GetAcl { path } => {
            let path = resolve_path(cwd, path);
            let out = crate::commands::acl::get_acl(client, &path).await?;
            match fmt {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
                OutputFormat::Human => println!("{}", crate::commands::acl::format_human(&out)),
            }
        }
        Commands::SetAcl { path, acl } => {
            let path = resolve_path(cwd, path);
            let out = crate::commands::acl::set_acl(client, &path, acl, None).await?;
            match fmt {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
                OutputFormat::Human => println!("{}", crate::commands::acl::format_human(&out)),
            }
        }
        Commands::Dump { path } => {
            let path = resolve_path(cwd, path);
            let out = crate::commands::dump::run(client, &path).await?;
            println!("{}", out);
        }
        Commands::Load { path, file, overwrite } => {
            let path = resolve_path(cwd, path);
            let out = crate::commands::load::run(client, &path, file, *overwrite).await?;
            match fmt {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
                OutputFormat::Human => println!("{}", out),
            }
        }
        Commands::Health { cmd } => {
            let (four_lw, host) = match cmd {
                HealthCommand::Ruok { host } => ("ruok", host.as_str()),
                HealthCommand::Stat { host } => ("stat", host.as_str()),
                HealthCommand::Srvr { host } => ("srvr", host.as_str()),
                HealthCommand::Mntr { host } => ("mntr", host.as_str()),
            };
            let out = crate::commands::health::run(four_lw, host).await?;
            match fmt {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
                OutputFormat::Human => println!("{}", crate::commands::health::format_human(&out)),
            }
        }
        Commands::Diff { path, host1: _, host2 } => {
            let path = resolve_path(cwd, path);
            let c2 = ZkClientImpl::connect(host2).await?;
            let out = crate::commands::diff::run(client, &c2, &path).await?;
            match fmt {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
                OutputFormat::Human => println!("{}", crate::commands::diff::format_human(&out)),
            }
        }
    }
    Ok(())
}

fn resolve_path(cwd: &str, path: &str) -> String {
    if path.starts_with('/') {
        path.to_string()
    } else if path == "." {
        cwd.to_string()
    } else if path.starts_with("./") {
        let rest = &path[2..];
        if cwd == "/" {
            format!("/{}", rest)
        } else {
            format!("{}/{}", cwd, rest)
        }
    } else if path == ".." {
        let mut parts: Vec<&str> = cwd.split('/').filter(|s| !s.is_empty()).collect();
        parts.pop();
        if parts.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", parts.join("/"))
        }
    } else if path.starts_with("../") {
        let mut parts: Vec<&str> = cwd.split('/').filter(|s| !s.is_empty()).collect();
        parts.pop();
        let parent = if parts.is_empty() { "/".to_string() } else { format!("/{}", parts.join("/")) };
        let rest = &path[3..];
        if parent == "/" {
            format!("/{}", rest)
        } else {
            format!("{}/{}", parent, rest)
        }
    } else {
        let base = if cwd == "/" { "" } else { cwd };
        format!("{}{}", base, crate::client::normalize_path(path))
    }
}

struct ReplState {
    client: ZkClientImpl,
    cwd: String,
}

const COMMANDS: &[&str] = &[
    "ls", "get", "set", "create", "delete", "stat", "exists",
    "tree", "find", "watch", "cp", "mv", "get-acl", "set-acl",
    "dump", "load", "health", "diff", "cd", "connect", "exit", "quit",
    "help", "pwd",
];

const PATH_COMMANDS: &[&str] = &[
    "ls", "get", "set", "create", "delete", "stat", "exists",
    "tree", "find", "watch", "cp", "mv", "get-acl", "set-acl",
    "dump", "load", "diff", "cd",
];

const TWO_PATH_COMMANDS: &[&str] = &["cp", "mv"];

#[derive(Clone, Debug)]
pub struct PathCandidate(pub String);

impl Candidate for PathCandidate {
    fn display(&self) -> &str {
        &self.0
    }
    fn replacement(&self) -> &str {
        &self.0
    }
}

pub struct ReplHelper {
    client: ZkClientImpl,
    rt_handle: tokio::runtime::Handle,
    cwd: std::sync::Arc<std::sync::Mutex<String>>,
}

impl ReplHelper {
    pub fn new(client: ZkClientImpl, rt_handle: tokio::runtime::Handle, cwd: std::sync::Arc<std::sync::Mutex<String>>) -> Self {
        Self { client, rt_handle, cwd }
    }

    pub fn complete_path(&self, line: &str, pos: usize) -> (usize, Vec<PathCandidate>) {
        let before = &line[..pos];
        let word_start = before.rfind(' ').map(|i| i + 1).unwrap_or(0);
        let partial = &before[word_start..];

        if word_start == 0 {
            let matches: Vec<_> = COMMANDS
                .iter()
                .filter(|c| c.starts_with(partial))
                .map(|c| PathCandidate(c.to_string()))
                .collect();
            return (0, matches);
        }

        let cmd_word = before[..word_start - 1].trim();
        let first_space = cmd_word.find(' ').unwrap_or(cmd_word.len());
        let cmd = &cmd_word[..first_space];

        if !PATH_COMMANDS.contains(&cmd) {
            return (pos, Vec::new());
        }

        let cwd = self.cwd.lock().unwrap().clone();
        let (parent_path, prefix, _) = split_path(&cwd, partial);

        let children = self.fetch_children(&parent_path);
        let matches: Vec<_> = children
            .iter()
            .filter(|c| c.starts_with(&prefix))
            .map(|c| {
                let c = c.clone();
                if partial.starts_with('/') || partial.starts_with('.') {
                    if parent_path == "/" {
                        format!("/{}", c)
                    } else {
                        format!("{}/{}", parent_path, c)
                    }
                } else if prefix.is_empty() {
                    c
                } else {
                    let idx = partial.len() - prefix.len();
                    format!("{}{}", &partial[..idx], c)
                }
            })
            .map(PathCandidate)
            .collect();

        (word_start, matches)
    }

    fn fetch_children(&self, path: &str) -> Vec<String> {
        let client = self.client.clone();
        let path = path.to_string();
        let handle = self.rt_handle.clone();
        let result = tokio::task::block_in_place(|| {
            handle.block_on(client.ls(&path))
        });
        result.unwrap_or_default()
    }
}

impl Helper for ReplHelper {}
impl Highlighter for ReplHelper {}
impl Hinter for ReplHelper {
    type Hint = String;
}
impl Validator for ReplHelper {}

impl Completer for ReplHelper {
    type Candidate = PathCandidate;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<PathCandidate>)> {
        let (start, candidates) = self.complete_path(line, pos);
        Ok((start, candidates))
    }
}

fn split_path(cwd: &str, partial: &str) -> (String, String, bool) {
    let resolved = if partial.starts_with('/') {
        partial.to_string()
    } else if partial == "." || partial.starts_with("./") {
        resolve_path(cwd, partial)
    } else if partial == ".." || partial.starts_with("../") {
        resolve_path(cwd, partial)
    } else {
        resolve_path(cwd, partial)
    };

    let (parent, prefix) = if let Some(slash_pos) = resolved.rfind('/') {
        let parent = &resolved[..slash_pos];
        let prefix = &resolved[slash_pos + 1..];
        (if parent.is_empty() { "/".to_string() } else { parent.to_string() }, prefix.to_string())
    } else {
        (cwd.to_string(), resolved)
    };

    let is_second = partial.contains('/');
    (parent, prefix, is_second)
}

pub async fn run_repl(client: ZkClientImpl) -> Result<()> {
    let cwd = std::sync::Arc::new(std::sync::Mutex::new("/".to_string()));
    let h = ReplHelper {
        client: client.clone(),
        rt_handle: tokio::runtime::Handle::current(),
        cwd: cwd.clone(),
    };
    let mut rl = rustyline::Editor::<ReplHelper, DefaultHistory>::with_history(
        rustyline::Config::builder().build(),
        rustyline::history::DefaultHistory::new(),
    )?;
    rl.set_helper(Some(h));
    let history_path = dirs_home_history();
    let _ = rl.load_history(&history_path);

    let mut state = ReplState {
        client,
        cwd: "/".to_string(),
    };

    let ctrlc = setup_ctrlc();

    loop {
        {
            let mut c = cwd.lock().unwrap();
            *c = state.cwd.clone();
        }
        let prompt = format!("ochk:{}> ", abbrev_path(&state.cwd));
        let readline = rl.readline(&prompt);
        if ctrlc.load(std::sync::atomic::Ordering::Relaxed) {
            println!();
            break;
        }
        match readline {
            Ok(line) => {
                let trimmed: &str = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed == "exit" || trimmed == "quit" {
                    break;
                }
                if handle_builtin(trimmed, &mut state).await {
                    let _ = rl.add_history_entry(trimmed);
                    continue;
                }
                let _ = rl.add_history_entry(trimmed);

                let args = shell_words(trimmed);
                let cli_result = Cli::try_parse_from(
                    std::iter::once("ochk").chain(args.iter().map(|s| s.as_str()))
                );
                match cli_result {
                    Ok(cli) => {
                        if let Some(cmd) = cli.command {
                            match dispatch(&state.client, &cmd, &OutputFormat::Human, &state.cwd).await {
                                Ok(()) => {}
                                Err(e) => eprintln!("Error: {}", format_error(&e)),
                            }
                        }
                    }
                    Err(e) => {
                        let msg = format!("{}", e);
                        let cleaned = msg.replace("error: ", "");
                        eprintln!("{}", cleaned);
                    }
                }
            }
            Err(_) => break,
        }
    }

    let _ = rl.save_history(&history_path);
    Ok(())
}

async fn handle_builtin(input: &str, state: &mut ReplState) -> bool {
    let args = shell_words(input);
    if args.is_empty() {
        return false;
    }
    match args[0].as_str() {
        "cd" => {
            let target = args.get(1).map(|s| s.as_str()).unwrap_or("/");
            let resolved = resolve_path(&state.cwd, target);
            match state.client.exists(&resolved).await {
                Ok(true) => state.cwd = resolved,
                Ok(false) => eprintln!("Node not found: {}", resolved),
                Err(e) => eprintln!("Error: {}", format_error(&e)),
            }
            true
        }
        "connect" => {
            if args.len() < 2 {
                eprintln!("Usage: connect <host:port>");
                return true;
            }
            match ZkClientImpl::connect(&args[1]).await {
                Ok(c) => {
                    state.client = c;
                    state.cwd = "/".to_string();
                    eprintln!("Connected to {}", args[1]);
                }
                Err(e) => eprintln!("Connection failed: {}", format_error(&e)),
            }
            true
        }
        "pwd" => {
            println!("{}", state.cwd);
            true
        }
        "help" => {
            print_help();
            true
        }
        _ => false,
    }
}

fn print_help() {
    let help = r#"Commands:
  ls <path> [-R] [-l]      List children
  get <path>               Get node data
  set <path> <data>        Set node data
  create <path> [data]     Create node (--ephemeral, --sequential, --recursive)
  delete <path> [-R]       Delete node
  stat <path>              Node metadata
  exists <path>            Check if node exists
  tree <path> [--depth N]  Tree view
  find <path> <pattern>    Find nodes by name
  watch <path>             Watch node changes
  cp <src> <dst> [-R]      Copy subtree
  mv <src> <dst>           Move node
  get-acl <path>           Get ACL
  set-acl <path> <acl>     Set ACL
  dump <path>              Export subtree as JSON
  load <path> <file>       Import subtree from JSON
  health ruok <host>       Health check
  diff <path> <host1> <host2>  Compare subtrees

REPL only:
  cd <path>                Change working directory
  connect <host:port>      Connect to different server
  pwd                      Print working directory
  exit / quit              Exit
  help                     This help"#;
    println!("{}", help);
}

fn abbrev_path(path: &str) -> String {
    if path == "/" {
        "/".to_string()
    } else {
        path.to_string()
    }
}

fn setup_ctrlc() -> std::sync::Arc<std::sync::atomic::AtomicBool> {
    let flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let f = flag.clone();
    let _ = ctrlc::set_handler(move || {
        f.store(true, std::sync::atomic::Ordering::Relaxed);
    });
    flag
}

fn format_error(e: &anyhow::Error) -> String {
    let msg = format!("{}", e);
    match msg.as_str() {
        s if s.contains("ConnectionLoss") || s.contains("timeout") => "Connection lost. Is the server running?".to_string(),
        s if s.contains("NoNode") => "Node not found".to_string(),
        s if s.contains("NodeExists") => "Node already exists".to_string(),
        s if s.contains("BadVersion") => "Version mismatch (use stat to check current version)".to_string(),
        s if s.contains("NoChildrenForEphemerals") => "Cannot create children of ephemeral nodes".to_string(),
        s if s.contains("InvalidACL") || s.contains("InvalidAcl") => "Invalid ACL. Format: scheme:id:perms (e.g. world:anyone:cdrwa)".to_string(),
        s if s.contains("NotEmpty") => "Node has children. Use --recursive to delete.".to_string(),
        s if s.contains("ConnectionRefused") => "Connection refused. Is the server running?".to_string(),
        _ => msg,
    }
}

fn shell_words(s: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut escape = false;

    for c in s.chars() {
        if escape {
            current.push(c);
            escape = false;
        } else if c == '\\' {
            escape = true;
        } else if c == '"' {
            in_quotes = !in_quotes;
        } else if c == ' ' && !in_quotes {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

fn dirs_home_history() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    format!("{}/.ochk_history", home)
}
