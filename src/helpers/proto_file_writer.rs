use std::{collections::HashMap, env, path::PathBuf, str::FromStr};

use tokio::{fs::File, io::AsyncWriteExt};

use crate::helpers::{
    common::convert_text_first_char_to_uppercase, config::STRUCT_PROTO_FILE_NAME, structs::Table,
};

use super::{
    common::{get_table_names, write_files},
    structs::SplitDirectoryConfig,
    traits::StringUtil,
};

pub async fn proto_file_writer(
    path: &Option<String>,
    use_split_file: bool,
    table_list: &Vec<Table>,
    split_directorys: &Vec<SplitDirectoryConfig>,
) -> Result<(), Box<dyn std::error::Error>> {
    match use_split_file {
        true => {
            proto_split_file_writer(path, table_list, split_directorys).await?;
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
    let result = File::create(match path {
        Some(path) => PathBuf::from_str(path.as_str())?,
        None => env::current_dir()?.join(STRUCT_PROTO_FILE_NAME),
    })
    .await?;
    let mut writer = tokio::io::BufWriter::new(result);
    let mut file: String = "syntax = \"proto3\";\npackage database;\n\n".into();

    for table in table_list {
        if table.use_proto_file == false {
            continue;
        }
        let (table_name, _, _, _) = get_table_names(table);

        file.push_str(&format!(
            "// [DART-SIGNAL]\nmessage {}Input {}",
            table_name, "{}\n\n"
        ));

        file.push_str(&format!(
            "// [RUST-SIGNAL]\nmessage {}Output {}",
            table_name, "{\n"
        ));
        file.push_str(&format!(
            "    repeated {} {} = 1;\n",
            table_name, table_name
        ));
        file.push_str("}\n\n");

        file.push_str(make_message(table_name.as_str(), table).as_str());
    }
    file.pop();
    writer.write_all(file.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

pub async fn proto_split_file_writer(
    path: &Option<String>,
    table_list: &Vec<Table>,
    split_directorys: &Vec<SplitDirectoryConfig>,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = match path {
        Some(path) => PathBuf::from_str(path.as_str())?,
        None => env::current_dir()?.join("sample"),
    };
    let mut file_list: HashMap<PathBuf, String> = HashMap::new();

    for table in table_list {
        if table.use_proto_file == false {
            continue;
        }
        let (table_name, _, file_name, _) = get_table_names(table);
        let mut file: String = format!("syntax = \"proto3\";\npackage {};\n\n", file_name);

        file.push_str(&format!(
            "// [DART-SIGNAL]\nmessage {}Input {}",
            table_name, "{}\n\n"
        ));

        file.push_str(&format!(
            "// [RUST-SIGNAL]\nmessage {}Output {}",
            table_name, "{\n"
        ));
        file.push_str(&format!(
            "    repeated {} {} = 1;\n",
            table_name, table_name
        ));
        file.push_str("}\n\n");

        file.push_str(&make_message(table_name.as_str(), table));
        file.pop();

        let current_path = match split_directorys.is_empty() {
            true => path.join(format!("{}.proto", file_name)),
            false => {
                let mut current_path = path.clone();
                for split_directory in split_directorys {
                    if file_name.starts_with(&split_directory.starts_with_name) {
                        current_path =
                            current_path.join(split_directory.directory_name.copy_string());
                        break;
                    }
                }
                current_path.join(format!("{}.proto", file_name))
            }
        };
        file_list.insert(current_path, file);
    }
    write_files(file_list).await?;
    Ok(())
}

fn make_message(table_name: &str, table: &Table) -> String {
    let mut file = String::new();
    file.push_str(&format!("message {} {}", table_name, "{\n"));
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
                        true => "_".into(),
                        false => c.to_string(),
                    };
                    column_name.push_str(char_item.as_str());
                });
                convert_text_first_char_to_uppercase(column_name.as_str())
            }
            false => {
                let mut column_name = column.column_name.clone();
                if !column_name.is_empty()
                    && column_name
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
            "bit" => "bool",
            "tinyint" => "uint32",
            "smallint" => "int32",
            "int" => "int32",
            "bigint" => "int64",
            "real" => "double",
            "float" => "double",
            "money" => "double",
            "datetime" => "string",
            "binary" => "bytes",
            "image" => "bytes",
            "ntext" => "string",
            "nvarchar" => "string",
            "text" => "string",
            _ => "string",
        };

        file.push_str(
            format!(
                "    {} {} = {};\n",
                match column.is_nullable.as_str() == "YES" {
                    true => format!("optional {}", data_type),
                    false => data_type.into(),
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
