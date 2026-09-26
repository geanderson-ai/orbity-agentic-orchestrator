use clap::Parser;
use orbity_cli::{Cli, CommandDispatcher};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match CommandDispatcher::dispatch(cli).await {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
