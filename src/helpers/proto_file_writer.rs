use std::{collections::HashMap, env};

use tokio::{fs::File, io::AsyncWriteExt};

use crate::helpers::{
    common::{convert_text_first_char_to_uppercase, get_static_str},
    config::STRUCT_PROTO_FILE_NAME,
    strucks::Table,
};

use super::common::{get_table_names, write_files};

pub async fn proto_file_writer(
    path: &Option<String>,
    use_split_file: bool,
    table_list: &Vec<Table>,
) -> Result<(), Box<dyn std::error::Error>> {
    match use_split_file {
        true => {
            proto_split_file_writer(table_list).await?;
        }
        false => {
            proto_one_file_writer(path, table_list).await?;
        }
    }
    Ok(())
}

pub async fn proto_one_file_writer(
    path: &Option<String>,
    table_list: &Vec<Table>,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = File::create(match path.is_some() {
        true => path.as_ref().unwrap().as_str(),
        false => STRUCT_PROTO_FILE_NAME,
    })
    .await?;
    let mut writer = tokio::io::BufWriter::new(result);
    let mut file: String = "syntax = \"proto3\";\npackage database;\n\n".into();

    for table in table_list {
        let (table_name, _, _) = get_table_names(table);

        file.push_str(make_message(table_name.as_str(), table).as_str());
    }
    file.pop();
    writer.write_all(file.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

pub async fn proto_split_file_writer(
    table_list: &Vec<Table>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file_list: HashMap<String, String> = HashMap::new();

    for table in table_list {
        let (table_name, _, file_name) = get_table_names(table);
        let mut file: String = format!("syntax = \"proto3\";\npackage {};\n\n", file_name);

        file.push_str(make_message(table_name.as_str(), table).as_str());
        file.pop();

        let current_path: String = env::current_dir()?.to_str().unwrap().into();
        let current_path = current_path.replacen("\"", "", 2);
        let current_path = format!("{}\\sample\\{}.proto", current_path, file_name);
        let current_path = current_path.replacen("\\", "/", current_path.len());
        print!("current_path22: {:?}\n", &current_path);
        file_list.insert(current_path, file);
    }
    write_files(file_list).await?;
    Ok(())
}

fn make_message(table_name: &str, table: &Table) -> String {
    let mut file = String::new();
    file.push_str(format!("message {} {}", table_name, "{\n").as_str());
    let mut column_index = 1;
    for column in &table.columns {
        let column_name = match column
            .column_name
            .find(|c: char| !c.is_ascii_alphabetic() && !c.is_ascii_digit() && c != '_')
            .is_some()
        {
            true => {
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
                let mut column_name = column.column_name.clone();
                if column_name
                    .char_indices()
                    .next()
                    .unwrap()
                    .1
                    .is_ascii_digit()
                {
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

        let data_type = match column.data_type.as_str() {
            "int" => "int32",
            "money" => "double",
            "datetime" => "string",
            "bit" => "bool",
            "smallint" => "int32",
            "ntext" => "string",
            "nvarchar" => "string",
            "text" => "string",
            "real" => "double",
            "tinyint" => "int32",
            "binary" => "bytes",
            "image" => "bytes",
            _ => "string",
        };

        file.push_str(
            format!(
                "    {} {} = {};\n",
                match column.is_nullable.as_str() {
                    "YES" => format!("optional {}", data_type),
                    _ => data_type.into(),
                },
                column_name,
                column_index
            )
            .as_str(),
        );
        column_index += 1;
    }
    file.push_str("}\n\n");
    file
}
