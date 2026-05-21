use anyhow::Result;
use clap::CommandFactory;
use clap::Parser;

fn main() -> Result<()> {
    let cli = ochk::cli::Cli::parse();

    if let Some(ochk::cli::Commands::Completions { shell }) = &cli.command {
        let mut cmd = ochk::cli::Cli::command();
        let shell = shell.parse::<clap_complete::Shell>().unwrap_or_else(|_| {
            eprintln!("Unknown shell. Supported: bash, zsh, fish, elvish, powershell");
            std::process::exit(1);
        });
        clap_complete::generate(shell, &mut cmd, "ochk", &mut std::io::stdout());
        return Ok(());
    }

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let client = ochk::client::ZkClientImpl::connect(&cli.connect).await?;

        match cli.command {
            Some(cmd) => {
                ochk::repl::run_once(client, cmd, cli.json).await?;
            }
            None => {
                ochk::repl::run_repl(client).await?;
            }
        }

        Ok(())
    })
}
