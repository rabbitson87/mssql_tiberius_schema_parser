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

pub fn convert_text_to_all_lowercase_snake_case(text: &str) -> String {
    let mut result = String::new();
    let mut is_uppercase = true;
    let mut is_first = true;
    let original_text = text.chars();
    let mut index = 0;

    for cha in text.chars() {
        if is_first {
            result.push_str(&cha.to_lowercase().to_string());
            is_first = false;
        } else {
            if cha.is_uppercase() {
                if !is_uppercase
                    || index + 1 <= original_text.as_str().len()
                        && original_text.clone().nth(index + 1).is_some()
                        && original_text
                            .clone()
                            .nth(index + 1)
                            .as_ref()
                            .unwrap()
                            .is_lowercase()
                {
                    result.push_str("_");
                }
                result.push_str(&cha.to_lowercase().to_string());
                is_uppercase = true;
            } else if cha.is_lowercase() {
                result.push(cha);
                is_uppercase = false;
            } else {
                result.push(cha);
                is_uppercase = false;
            }
        }
        index += 1;
    }
    result
}

pub fn convert_text_first_char_to_uppercase_else_lowercase(text: &str) -> String {
    let mut result = String::new();
    let mut first_char = true;
    for c in text.chars() {
        if first_char {
            result.push_str(&c.to_uppercase().to_string());
            first_char = false;
        } else {
            result.push_str(&c.to_lowercase().to_string());
        }
    }
    result
}

pub fn get_table_names(table: &crate::helpers::structs::Table) -> (String, String, String, String) {
    let table_name = table.name.get_table_name();
    let table_name_dart = table.name.get_table_name_dart();
    let file_name = table.name.get_file_name();
    let sql_table_name = table.name.get_sql_table_name();
    (table_name, table_name_dart, file_name, sql_table_name)
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
