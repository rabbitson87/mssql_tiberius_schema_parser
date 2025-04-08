use crate::helpers::config::{STRUCT_FILE_NAME, STRUCT_SIGNAL_FILE_NAME};
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use strum::AsRefStr;

use super::structs::InnerArgs;

#[derive(Parser, Debug)] // requires `derive` feature
#[command(author, version = env!("CARGO_PKG_VERSION"), about, long_about = None)]
pub struct Cli {
    #[arg(
        long = "host",
        value_name = "HOST",
        help = "A host or ip address to connect to.\n- Defaults to `localhost`"
    )]
    pub host: Option<String>,

    #[arg(
        long = "port",
        value_name = "PORT",
        help = "The server port.\n- Defaults to `1434`"
    )]
    pub port: Option<u16>,

    #[arg(
        short = 'd',
        value_name = "DATABASE",
        help = "The database to connect to.\n- Defaults to `master`"
    )]
    pub database: Option<String>,

    #[arg(
        short = 'a',
        value_name = "APPLICATION NAME",
        help = "Sets the application name to the connection,\nqueryable with the `APP_NAME()` command.\n- Defaults to no name specified."
    )]
    pub application_name: Option<String>,

    #[arg(
        short = 'i',
        value_name = "INSTANCE NAME",
        help = "The instance name as defined in the SQL Browser.\nOnly available on Windows platforms.\nIf specified, the port is replaced with the value returned from the browser.\nIf you write win_auth, please write down except the computer name\n- Required for win_auth\n- Defaults to no name specified."
    )]
    pub instance_name: Option<String>,

    #[arg(
        short = 'u',
        value_name = "USER",
        help = "The user to connect with.\nIf you write win_auth, please write down except the computer name\n- Defaults to no name specified.",
        default_value = ""
    )]
    pub user: String,

    #[arg(
        short = 'p',
        value_name = "PASSWORD",
        help = "The password to connect with.\n- Defaults to no name specified.",
        default_value = ""
    )]
    pub password: String,

    #[arg(
        short = 't',
        value_name = "TYPE",
        help = format!("The authentication type to use.\n\n- Defaults to {}", AuthType::ServerAuth.as_ref()),
        default_value_t = AuthType::ServerAuth
    )]
    #[clap(value_enum, default_value_t=AuthType::ServerAuth)]
    pub _type: AuthType,

    #[arg(
        long = "use_signal_parser",
        value_name = "USE SIGNAL PARSER",
        help = "Use to_dart function. add cli option with --use_signal_parser.\n- Defaults to false",
        default_value = "false"
    )]
    pub use_signal_parser: bool,

    #[arg(
        long = "use_split_file",
        value_name = "USE SPLIT FILE",
        help = "Use split file. add cli option with --use_split_file.\n- Defaults to false",
        default_value = "false"
    )]
    pub use_split_file: bool,

    #[arg(
        long = "path",
        value_name = "PATH",
        help = format!("The path to the rs file to execute.\n- Defaults to {}", STRUCT_FILE_NAME)
    )]
    pub path: Option<String>,

    #[arg(
        long = "signal_path",
        value_name = "SIGNAL PATH",
        help = format!("The path to the signal file to execute.\n- Defaults to {}", STRUCT_SIGNAL_FILE_NAME)
    )]
    pub signal_path: Option<String>,

    #[arg(
        long = "config_path",
        value_name = "CONFIG PATH",
        help = "The path to the config file to execute.\n- Defaults no name specified."
    )]
    pub config_path: Option<String>,

    #[arg(
        long = "use_insert_query",
        value_name = "USE INSERT QUERY",
        help = "Use insert query. add cli option with --use_insert_query.\n- Defaults to true",
        default_value = "true"
    )]
    pub use_insert_query: bool,
}

impl Cli {
    pub fn to_inner_args(self) -> InnerArgs {
        InnerArgs {
            host: self.host,
            port: self.port,
            database_name: self.database,
            application_name: self.application_name,
            instance_name: self.instance_name,
            user: self.user,
            password: self.password,
            _type: self._type,
            use_signal_parser: self.use_signal_parser,
            use_split_file: self.use_split_file,
            use_insert_query: self.use_insert_query,
            path: self.path,
            signal_path: self.signal_path,
            database: None,
        }
    }
}

/// Doc comment
#[derive(ValueEnum, Clone, Debug, AsRefStr, Deserialize, Serialize)]
#[value(rename_all = "snake_case")]
pub enum AuthType {
    /// Doc comment
    #[value(help = "Use Windows Authentication")]
    WinAuth,
    #[value(help = "Use SQL Server Authentication")]
    ServerAuth,
}
