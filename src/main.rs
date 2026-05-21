use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = ochk::cli::Cli::parse();

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
