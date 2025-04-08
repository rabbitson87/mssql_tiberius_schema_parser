# mssql_tiberius_schema_parser
Generates the schema of all tables in mssql as a structure.

# How to use
cargo install mssql_tiberius_schema_parser
mssql_tiberius_schema_parser.exe [OPTIONS] -u <USER> -p <PASSWORD> -t <TYPE>

Options:
      --host <HOST>
          A host or ip address to connect to.
          - Defaults to `localhost`

      --port <PORT>
          The server port.
          - Defaults to `61363`

  -d <DATABASE>
          The database to connect to.
          - Defaults to `master`

  -a <APPLICATION NAME>
          Sets the application name to the connection,
          queryable with the `APP_NAME()` command.
          - Defaults to no name specified.

  -i <INSTANCE NAME>
          The instance name as defined in the SQL Browser.
          Only available on Windows platforms.
          If specified, the port is replaced with the value returned from the browser.
          If you write win_auth, please write down except the computer name
          - Required for win_auth
          - Defaults to no name specified.

  -u <USER>
          The user to connect with.
          If you write win_auth, please write down except the computer name
          - Required

  -p <PASSWORD>
          The password to connect with.
          - Required

  -t <TYPE>
          The authentication type to use.
          - Required

          Possible values:
          - win_auth:    Use Windows Authentication
          - server_auth: Use SQL Server Authentication

      --use_signal_parser
          Use date time to string. add cli option with --use_signal_parser.
          - Defaults to false

      --use_split_file
          Use split file. add cli option with --use_split_file.
          - Defaults to false

      --path <PATH>
          The path to the rs file to execute.
          - Defaults to structs.rs

      --signal_path <SIGNAL PATH>
          The path to the signal file to execute.
          - Defaults to signals.rs

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
