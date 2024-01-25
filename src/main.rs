use crate::helpers::args_parser::Cli;
use clap::Parser;
use helpers::auth_handler::auth_handler;

mod helpers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    auth_handler(args).await?;

    Ok(())
}
