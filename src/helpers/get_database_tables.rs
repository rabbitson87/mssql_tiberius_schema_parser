use std::sync::Arc;

use tiberius::Client;
use tokio::{
    net::TcpStream,
    sync::{mpsc, Mutex},
};
use tokio_util::compat::Compat;

use crate::helpers::traits::select_parser::{SelectParser, SelectParserTrait};

pub async fn get_database_tables<'a>(
    tx: mpsc::Sender<(SelectParser<'a>, SelectParser<'a>)>,
    database_name: String,
    client: Arc<Mutex<Client<Compat<TcpStream>>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = client.lock().await;
    let tables = client
        .simple_query(format!(
            "SELECT
                *
            FROM
            {}.INFORMATION_SCHEMA.TABLES
            ",
            database_name
        ))
        .await?
        .into_results()
        .await?
        .select_parser();

    let columns = client
        .simple_query(format!(
            "SELECT
                *
            FROM
            {}.INFORMATION_SCHEMA.COLUMNS
            ",
            database_name
        ))
        .await?
        .into_results()
        .await?
        .select_parser();
    let _ = tx.send((tables, columns)).await;
    Ok(())
}
