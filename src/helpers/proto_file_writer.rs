use std::str::FromStr;

use tokio::{fs::File, io::AsyncWriteExt};

use crate::helpers::{
    common::{convert_text_first_char_to_uppercase, get_static_str},
    config::STRUCT_PROTO_FILE_NAME,
    strucks::Table,
};

pub async fn proto_file_writer(
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
        file.push_str(
            format!(
                "message {}{}{} ",
                convert_text_first_char_to_uppercase(table.name.table_catalog.as_str()),
                convert_text_first_char_to_uppercase(table.name.table_schema.as_str()),
                convert_text_first_char_to_uppercase(table.name.table_name.as_str())
            )
            .as_str(),
        );
        file.push_str("{\n");
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
                    let mut column_name = String::from_str(column.column_name.as_str())?;
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
    }
    file.pop();
    writer.write_all(file.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}
