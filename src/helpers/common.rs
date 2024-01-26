pub fn get_static_str(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

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
    let mut is_uppercase = false;
    let mut is_first = true;
    for cha in text.chars() {
        if is_first {
            result.push_str(&cha.to_lowercase().to_string());
            is_first = false;
        } else {
            if cha.is_uppercase() {
                if !is_uppercase {
                    result.push_str("_");
                }
                result.push_str(&cha.to_lowercase().to_string());
                is_uppercase = true;
            }
            if cha.is_lowercase() {
                result.push(cha);
                is_uppercase = false;
            }
        }
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

pub fn get_table_names(table: &crate::helpers::strucks::Table) -> (String, String, String) {
    let table_name = format!(
        "{}{}{}",
        convert_text_first_char_to_uppercase(table.name.table_catalog.as_str()),
        convert_text_first_char_to_uppercase(table.name.table_schema.as_str()),
        convert_text_first_char_to_uppercase(table.name.table_name.as_str())
    );
    let table_name_dart = format!(
        "{}{}{}",
        convert_text_first_char_to_uppercase_else_lowercase(table.name.table_catalog.as_str()),
        convert_text_first_char_to_uppercase_else_lowercase(table.name.table_schema.as_str()),
        convert_text_first_char_to_uppercase_else_lowercase(table.name.table_name.as_str())
    );
    let file_name = format!(
        "{}_{}_{}",
        table.name.table_catalog.to_lowercase(),
        table.name.table_schema.to_lowercase(),
        table.name.table_name.to_lowercase()
    );
    (table_name, table_name_dart, file_name)
}

pub async fn write_files(
    file_list: std::collections::HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    use tokio::io::AsyncWriteExt;

    let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    let first_path = file_list
        .keys()
        .find(|file_name| file_name.split("/").count() > 1);

    print!("first_path: {:?}\n", first_path);
    if first_path.is_some() {
        let folder_path = first_path
            .unwrap()
            .split("/")
            .fold(String::new(), |acc, x| {
                match x.contains(".") || x.is_empty() {
                    true => acc,
                    false => match acc.is_empty() {
                        true => x.into(),
                        false => format!("{}/{}", acc, x),
                    },
                }
            });
        print!("folder_path: {:?}\n", folder_path);
        tokio::fs::create_dir_all(folder_path).await?;
    }

    for (file_name, file) in file_list {
        let tx_copy = tx.clone();
        tokio::spawn(async move {
            let result = tokio::fs::File::create(file_name).await?;
            print!("file result: {:?}\n", result);
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
