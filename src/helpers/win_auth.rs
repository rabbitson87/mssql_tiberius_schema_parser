use std::sync::Arc;

use crate::helpers::{
    args_parser::Cli,
    common::{convert_text_first_char_to_uppercase, get_static_str},
    get_database_tables::get_database_tables,
    get_table_schema::get_table_schema,
    strucks::{ColumnName, Table, TableName},
    traits::select_parser::SelectParserTrait,
};
use gethostname::gethostname;
use tiberius::{AuthMethod, Client, ColumnData, Config};
use tokio::{
    fs::File,
    io::AsyncWriteExt,
    net::TcpStream,
    sync::{mpsc, Mutex},
};
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

pub async fn win_auth(args: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::new();
    let hostname = gethostname();

    config.authentication(AuthMethod::windows(
        format!("{:?}\\{}", hostname, args.user.as_str()),
        args.password.as_str(),
    ));
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
    let mut use_date_time = false;
    let mut use_date_time_option = false;
    let mut use_date_time_real = false;
    while let Some((tables, columns)) = rx.recv().await {
        let table_names = get_table_schema::<TableName>(tables);
        let column_names = get_table_schema::<ColumnName>(columns);

        for table_name in table_names {
            let mut table = Table {
                name: table_name.clone(),
                columns: vec![],
            };
            column_names.iter().for_each(|column_name| {
                if &table_name.table_name == &column_name.table_name {
                    table.columns.push(column_name.clone());
                }
                if column_name.data_type == "datetime" && !use_date_time {
                    use_date_time = true;
                }
                if column_name.data_type == "datetime"
                    && column_name.is_nullable == "NO"
                    && !use_date_time_real
                {
                    use_date_time_real = true;
                }
                if column_name.data_type == "datetime"
                    && column_name.is_nullable == "YES"
                    && !use_date_time_option
                {
                    use_date_time_option = true;
                }
            });
            table_list.push(table);
        }
    }

    let result = File::create(match args.path.is_some() {
        true => args.path.unwrap(),
        false => "struct.rs".into(),
    })
    .await?;
    let mut writer = tokio::io::BufWriter::new(result);
    let mut file: String = "use serde::{Deserialize, Serialize};\n".into();
    if use_date_time {
        file.push_str("use tiberius::time::chrono::{DateTime, Utc};\n");

        if use_date_time_option {
            file.push_str("use chrono::serde::ts_seconds_option;\n");
        }

        if use_date_time_real {
            file.push_str("use chrono::serde::ts_seconds;\n");
        }
    }
    file.push_str("\n");

    for table in table_list {
        file.push_str("#[allow(non_snake_case, non_camel_case_types)]\n");
        file.push_str("#[derive(Serialize, Deserialize, Debug)]\n");
        file.push_str(
            format!(
                "struct {}{}{} ",
                convert_text_first_char_to_uppercase(table.name.table_catalog.as_str()),
                convert_text_first_char_to_uppercase(table.name.table_schema.as_str()),
                convert_text_first_char_to_uppercase(table.name.table_name.as_str())
            )
            .as_str(),
        );
        file.push_str("{\n");
        for column in table.columns {
            let column_name = match column
                .column_name
                .find(|c: char| !c.is_ascii_alphabetic() && !c.is_ascii_digit() && c != '_')
                .is_some()
            {
                true => {
                    file.push_str(
                        format!(
                            "    #[serde(rename(deserialize = \"{}\"))]\n",
                            column.column_name.as_str()
                        )
                        .as_str(),
                    );
                    let mut column_name = String::new();

                    column.column_name.chars().for_each(|c| {
                        let char_item = match !c.is_ascii_alphabetic() {
                            true => "_",
                            false => get_static_str(c.to_string()),
                        };
                        column_name.push_str(char_item);
                    });
                    convert_text_first_char_to_uppercase(column_name.as_str())
                }
                false => {
                    let mut column_name = column.column_name;
                    if column_name
                        .char_indices()
                        .next()
                        .unwrap()
                        .1
                        .is_ascii_digit()
                    {
                        file.push_str(
                            format!(
                                "    #[serde(rename(deserialize = \"{}\"))]\n",
                                column_name.as_str()
                            )
                            .as_str(),
                        );
                        column_name = match column_name.char_indices().next().unwrap().1 {
                            '0' => format!("zero{}", &column_name[1..]),
                            '1' => format!("one{}", &column_name[1..]),
                            '2' => format!("two{}", &column_name[1..]),
                            '3' => format!("three{}", &column_name[1..]),
                            '4' => format!("four{}", &column_name[1..]),
                            '5' => format!("five{}", &column_name[1..]),
                            '6' => format!("six{}", &column_name[1..]),
                            '7' => format!("seven{}", &column_name[1..]),
                            '8' => format!("eight{}", &column_name[1..]),
                            '9' => format!("nine{}", &column_name[1..]),
                            _ => column_name,
                        };
                    }
                    convert_text_first_char_to_uppercase(column_name.as_str())
                }
            };
            if column.data_type == "datetime" {
                match column.is_nullable.as_str() {
                    "YES" => file.push_str("    #[serde(with = \"ts_seconds_option\")]\n"),
                    "NO" => file.push_str("    #[serde(with = \"ts_seconds\")]\n"),
                    _ => {}
                };
            }

            let data_type = match column.data_type.as_str() {
                "int" => "i32",
                "money" => "f64",
                "datetime" => "DateTime<Utc>",
                "bit" => "bool",
                "smallint" => "i16",
                "ntext" => "String",
                "nvarchar" => "String",
                "text" => "String",
                "real" => "f32",
                "tinyint" => "u8",
                "binary" => "Vec<u8>",
                "image" => "Vec<u8>",
                _ => "String",
            };

            file.push_str(
                format!(
                    "    pub {}: {},\n",
                    column_name,
                    match column.is_nullable.as_str() {
                        "YES" => format!("Option<{}>", data_type),
                        _ => data_type.into(),
                    }
                )
                .as_str(),
            );
        }
        file.push_str("}\n\n");
    }
    file.pop();
    writer.write_all(file.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}
