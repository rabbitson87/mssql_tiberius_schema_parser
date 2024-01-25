use std::str::FromStr;

use tokio::{fs::File, io::AsyncWriteExt};

use crate::helpers::{
    common::{convert_text_first_char_to_uppercase, get_static_str},
    config::STRUCT_FILE_NAME,
    strucks::Table,
};

pub async fn rs_file_writer(
    path: &Option<String>,
    table_list: &Vec<Table>,
    use_date_time: bool,
    use_date_time_option: bool,
    use_date_time_real: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = File::create(match path.is_some() {
        true => path.as_ref().unwrap().as_str(),
        false => STRUCT_FILE_NAME,
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
                "pub struct {}{}{} ",
                convert_text_first_char_to_uppercase(table.name.table_catalog.as_str()),
                convert_text_first_char_to_uppercase(table.name.table_schema.as_str()),
                convert_text_first_char_to_uppercase(table.name.table_name.as_str())
            )
            .as_str(),
        );
        file.push_str("{\n");
        for column in &table.columns {
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
                    let mut column_name = String::from_str(column.column_name.as_str())?;
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
