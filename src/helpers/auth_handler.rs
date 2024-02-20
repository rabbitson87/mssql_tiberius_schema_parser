use std::sync::Arc;

use crate::helpers::{
    args_parser::{AuthType, Cli},
    get_database_tables::get_database_tables,
    get_table_schema::get_table_schema,
    proto_file_writer::proto_file_writer,
    rs_file_writer::rs_file_writer,
    strucks::{ColumnName, Table, TableName},
    traits::select_parser::SelectParserTrait,
};
use gethostname::gethostname;
use tiberius::{AuthMethod, Client, ColumnData, Config};
use tokio::{
    net::TcpStream,
    sync::{mpsc, Mutex},
};
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

pub async fn auth_handler(args: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::new();
    let hostname = gethostname();

    config.authentication(match args._type {
        AuthType::ServerAuth => AuthMethod::sql_server(
            format!("{:?}\\{}", hostname, args.user.as_str()),
            args.password.as_str(),
        ),
        AuthType::WinAuth => AuthMethod::windows(
            format!("{:?}\\{}", hostname, args.user.as_str()),
            args.password.as_str(),
        ),
    });
    if args.instance_name.is_none() {
        return Err("instance_name is required")?;
    }
    config.instance_name(format!("{:?}\\{}", hostname, args.instance_name.unwrap()));
    config.port(61363);
    config.trust_cert();

    let tcp = TcpStream::connect(config.get_addr()).await?;
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

    let mut table_list: Vec<Table> = vec![];
    while let Some((tables, columns)) = rx.recv().await {
        let table_names = get_table_schema::<TableName>(&tables);
        let column_names = get_table_schema::<ColumnName>(&columns);

        for table_name in table_names {
            let mut table = Table {
                name: table_name.clone(),
                columns: vec![],
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
        args.use_proto_parser,
        args.use_split_file,
        &table_list,
    )
    .await?;

    proto_file_writer(&args.proto_path, args.use_split_file, &table_list).await?;
    Ok(())
}
