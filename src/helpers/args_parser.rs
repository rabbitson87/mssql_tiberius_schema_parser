use crate::helpers::config::{STRUCT_FILE_NAME, STRUCT_PROTO_FILE_NAME};
use clap::{Parser, ValueEnum};

#[derive(Parser)] // requires `derive` feature
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
        help = "The server port.\n- Defaults to `61363`"
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
        help = "The user to connect with.\nIf you write win_auth, please write down except the computer name\n- Required"
    )]
    pub user: String,

    #[arg(
        short = 'p',
        value_name = "PASSWORD",
        help = "The password to connect with.\n- Required"
    )]
    pub password: String,

    #[arg(
        short = 't',
        value_name = "TYPE",
        help = "The authentication type to use.\n- Required"
    )]
    pub _type: AuthType,

    #[arg(
        long = "use_proto_parser",
        value_name = "USE PROTO PARSER",
        help = "Use date time to string. add cli option with --use_proto_parser.\n- Defaults to false",
        default_value = "false"
    )]
    pub use_proto_parser: bool,

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
        long = "proto_path",
        value_name = "PROTO PATH",
        help = format!("The path to the proto file to execute.\n- Defaults to {}", STRUCT_PROTO_FILE_NAME)
    )]
    pub proto_path: Option<String>,
}

/// Doc comment
#[derive(ValueEnum, Clone, Debug)]
#[value(rename_all = "snake_case")]
pub enum AuthType {
    /// Doc comment
    #[value(help = "Use Windows Authentication")]
    WinAuth,
    #[value(help = "Use SQL Server Authentication")]
    ServerAuth,
}
