use std::path::PathBuf;

pub fn convert_text_first_char_to_uppercase(text: &str) -> String {
    let mut result = String::new();
    let mut first_char = true;
    for c in text.chars() {
        if first_char {
            result.push_str(&c.to_uppercase().to_string());
            first_char = false;
        } else {
            result.push(c);
        }
    }
    result
}

pub fn get_table_names(table: &crate::helpers::structs::Table) -> (String, String, String) {
    let table_name = table.name.get_table_name();
    let file_name = table.name.get_file_name();
    let sql_table_name = table.name.get_sql_table_name();
    (table_name, file_name, sql_table_name)
}

pub async fn write_files(
    file_list: std::collections::HashMap<PathBuf, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    use tokio::io::AsyncWriteExt;

    let (tx, mut rx) = tokio::sync::mpsc::channel(32);

    for (file_name, file) in file_list {
        let folder_path = match file_name.is_dir() {
            true => file_name.to_path_buf(),
            false => file_name.parent().unwrap().to_path_buf(),
        };
        if !folder_path.exists() {
            tokio::fs::create_dir_all(folder_path).await?;
        }
        let tx_copy = tx.clone();
        tokio::spawn(async move {
            let result = tokio::fs::File::create(file_name).await?;
            let mut writer: tokio::io::BufWriter<tokio::fs::File> =
                tokio::io::BufWriter::new(result);
            writer.write_all(file.as_bytes()).await?;
            writer.flush().await?;
            tx_copy.send(true).await?;
            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        });
    }
    drop(tx);

    while let Some(_) = rx.recv().await {}
    Ok(())
}
