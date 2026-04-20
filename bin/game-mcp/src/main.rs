use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, default_value_t = true)]
    once: bool,
}

fn main() -> Result<()> {
    let _args = Args::parse();
    let scaffold = game_mcp::McpScaffold::new();
    scaffold.start()?;
    Ok(())
}
