use std::{collections::HashMap, sync::Arc};

use crate::helpers::{
    args_parser::{AuthType, Cli},
    get_database_tables::get_database_tables,
    get_table_schema::GetTableSchema,
    rs_file_writer::rs_file_writer,
    signal_file_writer::signal_file_writer,
    structs::{ColumnName, InnerArgs, Table, TableConfig, TableName},
    traits::{select_parser::SelectParserTrait, StringUtil},
};
use gethostname::gethostname;
use tiberius::{AuthMethod, Client, ColumnData, Config, SqlBrowser};
use tokio::{
    net::TcpStream,
    sync::{mpsc, Mutex},
};
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

#[test]
fn test_parse_toml() {
    let config_path = "config.toml";
    let config = std::fs::read_to_string(config_path).unwrap();
    println!("{:?}", toml::from_str::<InnerArgs>(&config).unwrap());
}

pub async fn auth_handler(args: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::new();
    let hostname = gethostname();

    let args = match args.config_path {
        Some(config_path) => {
            let config = std::fs::read_to_string(config_path)?;
            toml::from_str::<InnerArgs>(&config)?
        }
        None => args.to_inner_args(),
    };
    match args._type {
        AuthType::WinAuth => {
            config.authentication(AuthMethod::windows(
                format!("{:?}\\{}", hostname, args.user),
                &args.password,
            ));
        }
        AuthType::ServerAuth => {
            config.authentication(AuthMethod::sql_server(&args.user, &args.password));
        }
    }

    if let Some(database_name) = args.database_name {
        config.database(database_name);
    }

    match args.instance_name {
        Some(instance_name) => {
            config.instance_name(instance_name);
        }
        None => {
            return Err("instance_name is required")?;
        }
    }
    config.trust_cert();

    let tcp = TcpStream::connect_named(&config).await?;
    tcp.set_nodelay(true)?;

    let client: Client<Compat<TcpStream>> = match Client::connect(config, tcp.compat_write()).await
    {
        // Connection successful.
        Ok(client) => client,
        // The server wants us to redirect to a different address
        Err(tiberius::error::Error::Routing { host, port }) => {
            let mut config = Config::new();

            config.host(&host);
            config.port(port);
            config.authentication(AuthMethod::windows(
                format!("{:?}\\{}", hostname, args.user.as_str()),
                args.password.as_str(),
            ));

            let tcp = TcpStream::connect(config.get_addr()).await?;
            tcp.set_nodelay(true)?;

            // we should not have more than one redirect, so we'll short-circuit here.
            Client::connect(config, tcp.compat_write()).await?
        }
        Err(e) => Err(e)?,
    };
    let client = Arc::new(Mutex::new(client));
    let client_copy = client.clone();
    let mut client_copy = client_copy.lock().await;

    let results = client_copy
        .query(
            "SELECT 
                name
            FROM sys.databases d
            WHERE 1=1
            and d.database_id > 4",
            &[],
        )
        .await?
        .into_results()
        .await?
        .select_parser();
    drop(client_copy);

    let mut database_names: Vec<String> = vec![];

    results.rows.iter().for_each(|row| {
        row.iter().for_each(|col| {
            if let ColumnData::String(Some(data)) = col {
                database_names.push(data.to_string());
            }
        });
    });

    print!("database_names: {:?}\n", database_names);

    let (tx, mut rx) = mpsc::channel(32);

    for database_name in database_names {
        let tx_copy = tx.clone();
        let client_copy = client.clone();
        tokio::spawn(async move {
            let _ = get_database_tables(tx_copy, database_name, client_copy).await;
        });
    }
    drop(tx);

    let mut tables_options: HashMap<String, TableConfig> = HashMap::new();
    let mut use_import_special = false;
    let mut split_directory = vec![];
    if let Some(database) = args.database {
        use_import_special = database.use_import_special;
        if let Some(tables) = database.tables {
            tables.into_iter().for_each(|table| {
                tables_options.insert(table.table_name.copy_string(), table);
            });
        }
        if let Some(split_directories) = database.split_directory {
            split_directory = split_directories;
        }
    }

    let mut table_list: Vec<Table> = vec![];
    while let Some((tables, columns)) = rx.recv().await {
        let table_names = tables.get_table_schema::<TableName>();
        let column_names = columns.get_table_schema::<ColumnName>();

        for table_name in table_names {
            if use_import_special && !tables_options.contains_key(&table_name.get_file_name()) {
                continue;
            }
            let mut table = Table {
                name: table_name.clone(),
                columns: vec![],
                use_signal_parser: match tables_options.get(&table_name.get_file_name()) {
                    Some(table_config) => table_config.use_signal_parser,
                    None => false,
                },
                use_signal_file: match tables_options.get(&table_name.get_file_name()) {
                    Some(table_config) => table_config.use_signal_file,
                    None => true,
                },
                use_insert_query: match tables_options.get(&table_name.get_file_name()) {
                    Some(table_config) => table_config.use_insert_query,
                    None => true,
                },
            };
            column_names.iter().for_each(|column_name| {
                if &table_name.table_name == &column_name.table_name {
                    table.columns.push(column_name.clone());
                }
            });
            table_list.push(table);
        }
    }

    rs_file_writer(
        &args.path,
        args.use_split_file,
        &table_list,
        &split_directory,
    )
    .await?;

    signal_file_writer(
        &args.signal_path,
        args.use_split_file,
        &table_list,
        &split_directory,
    )
    .await?;
    Ok(())
}
