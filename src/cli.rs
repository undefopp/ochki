use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "ochki",
    version,
    about = "Unified ZooKeeper / ClickHouse Keeper CLI",
    arg_required_else_help = true
)]
pub struct Cli {
    #[arg(short, long, default_value = "127.0.0.1:2181")]
    pub connect: String,

    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    Ls {
        path: String,
        #[arg(short = 'R', long)]
        recursive: bool,
        #[arg(short = 'l', long)]
        detailed: bool,
    },
    Get {
        path: String,
    },
    Set {
        path: String,
        data: String,
        #[arg(long)]
        version: Option<i32>,
    },
    Create {
        path: String,
        data: Option<String>,
        #[arg(long)]
        ephemeral: bool,
        #[arg(long)]
        sequential: bool,
        #[arg(long)]
        recursive: bool,
    },
    Delete {
        path: String,
        #[arg(short = 'R', long)]
        recursive: bool,
    },
    Stat {
        path: String,
    },
    Exists {
        path: String,
        #[arg(long)]
        verbose: bool,
    },
    Tree {
        path: String,
        #[arg(long)]
        depth: Option<usize>,
    },
    Find {
        path: String,
        pattern: String,
        #[arg(long)]
        r#type: Option<String>,
    },
    Watch {
        path: String,
    },
    Cp {
        src: String,
        dst: String,
        #[arg(short = 'R', long)]
        recursive: bool,
    },
    Mv {
        src: String,
        dst: String,
        #[arg(long)]
        dry_run: bool,
    },
    GetAcl {
        path: String,
    },
    SetAcl {
        path: String,
        acl: String,
    },
    Dump {
        path: String,
    },
    Load {
        path: String,
        file: String,
        #[arg(long)]
        overwrite: bool,
    },
    Health {
        #[command(subcommand)]
        cmd: HealthCommand,
    },
    Diff {
        path: String,
        host1: String,
        host2: String,
    },
    AddAuth {
        scheme: String,
        credential: String,
    },
    Completions {
        shell: String,
    },
}

#[derive(Subcommand, Clone)]
pub enum HealthCommand {
    Ruok { host: String },
    Stat { host: String },
    Srvr { host: String },
    Mntr { host: String },
}
