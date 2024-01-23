use crate::helpers::args_parser::{AuthType, Cli};
use clap::Parser;
use helpers::win_auth::win_auth;

mod helpers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    match args._type {
        AuthType::WinAuth => win_auth(args).await?,
        AuthType::ServerAuth => println!("SQL Authentication"),
    }

    Ok(())
}
