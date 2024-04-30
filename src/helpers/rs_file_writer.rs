use std::{collections::HashMap, env};

use tokio::{fs::File, io::AsyncWriteExt};

use crate::helpers::{
    common::convert_text_first_char_to_uppercase, config::STRUCT_FILE_NAME, strucks::Table,
};

use super::{
    common::{convert_text_to_all_lowercase_snake_case, get_table_names, write_files},
    strucks::ColumnName,
};

pub async fn rs_file_writer(
    path: &Option<String>,
    use_proto_parser: bool,
    use_split_file: bool,
    table_list: &Vec<Table>,
) -> Result<(), Box<dyn std::error::Error>> {
    match use_split_file {
        true => rs_split_file_writer(table_list, use_proto_parser).await?,
        false => rs_one_file_writer(path, table_list).await?,
    }
    Ok(())
}

pub async fn rs_one_file_writer(
    path: &Option<String>,
    table_list: &Vec<Table>,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = File::create(match path.is_some() {
        true => path.as_ref().unwrap().as_str(),
        false => STRUCT_FILE_NAME,
    })
    .await?;
    let mut writer: tokio::io::BufWriter<File> = tokio::io::BufWriter::new(result);
    let mut file: String = "use serde::{Deserialize, Serialize};\n\n".into();

    for table in table_list {
        let (table_name, _, _) = get_table_names(table);
        file.push_str(make_struct(table_name.as_str(), table).as_str());
    }
    file.pop();

    file = format!("{}{}", import_file(&file), file);

    writer.write_all(file.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

pub async fn rs_split_file_writer(
    table_list: &Vec<Table>,
    use_proto_parser: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file_list: HashMap<String, String> = HashMap::new();

    for table in table_list {
        let mut file: String = "use serde::{Deserialize, Serialize};\n\n".into();
        let (table_name, table_name_dart, file_name) = get_table_names(table);

        file.push_str(make_struct(table_name.as_str(), table).as_str());

        if use_proto_parser {
            file.push_str(format!("impl {} ", &table_name).as_str());
            file.push_str("{\n");

            file.push_str(
                format!(
                    "    pub fn to_dart(&self) -> crate::messages::{}::{} {}",
                    &file_name, &table_name_dart, "{\n"
                )
                .as_str(),
            );
            file.push_str(
                format!(
                    "        crate::messages::{}::{} {}",
                    &file_name, &table_name_dart, "{\n"
                )
                .as_str(),
            );
            for column in &table.columns {
                let column_name = get_column_name(column);
                let data_type = match column.data_type.as_str() {
                    "datetime" => ".to_rfc3339()",
                    "real" => " as f64",
                    "tinyint" => " as i32",
                    "smallint" => " as i32",
                    _ => "",
                };

                file.push_str(
                    format!(
                        "            {}: {},\n",
                        convert_text_to_all_lowercase_snake_case(&column_name),
                        match column.is_nullable.as_str() == "YES" {
                            true => match column.data_type.as_str() {
                                "ntext" => make_string_matcher(&column_name),
                                "nvarchar" => make_string_matcher(&column_name),
                                "text" => make_string_matcher(&column_name),
                                "datetime" => make_matcher(
                                    format!("Some(Into::into(&*value{}))", data_type).as_str(),
                                    &column_name
                                ),
                                "real" => make_number_matcher(&column_name, data_type),
                                "tinyint" => make_number_matcher(&column_name, data_type),
                                "smallint" => make_number_matcher(&column_name, data_type),
                                _ => format!("self.{}{}", column_name, data_type),
                            },
                            false => format!("self.{}{}", column_name, data_type),
                        }
                    )
                    .as_str(),
                );
            }
            file.push_str("        }\n");
            file.push_str("    }\n");
            file.push_str("}\n\n");
        }
        file.pop();

        file = format!("{}{}", import_file(&file), file);

        let current_path: String = env::current_dir()?.to_str().unwrap().into();
        let current_path = current_path.replacen("\"", "", 2);
        let current_path = format!("{}\\sample\\{}.rs", current_path, file_name);
        let current_path = current_path.replacen("\\", "/", current_path.len());
        file_list.insert(current_path, file);
    }
    write_files(file_list).await?;

    Ok(())
}

fn make_string_matcher(column_name: &str) -> String {
    make_matcher(
        r#"Some(
                    value.replace("\"", "\\\"")
                        .replace("\n", "\\\\n")
                        .replace("\r", "\\\\r")
                        .replace("\t", "\\\\t")
                        .into()
                )"#,
        &column_name,
    )
}

fn make_number_matcher(column_name: &str, data_type: &str) -> String {
    make_matcher(format!("Some(*value{})", data_type).as_str(), &column_name)
}

fn make_matcher(some_text: &str, column_name: &str) -> String {
    format!(
        "match &self.{} {}{}{}{}",
        &column_name,
        "{\n",
        "                Some(value) => ",
        some_text,
        ",\n                None => None,\n            }"
    )
}

fn make_struct(table_name: &str, table: &Table) -> String {
    let mut file = String::new();
    file.push_str("#[allow(non_snake_case, non_camel_case_types)]\n");
    file.push_str("#[derive(Serialize, Deserialize, Debug)]\n");
    file.push_str(format!("pub struct {} ", table_name).as_str());
    file.push_str("{\n");
    for column in &table.columns {
        let column_name = get_column_name(column);
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
                match column.is_nullable.as_str() == "YES" {
                    true => format!("Option<{}>", data_type),
                    false => data_type.into(),
                }
            )
            .as_str(),
        );
    }
    file.push_str("}\n\n");
    file
}

fn import_file(file: &str) -> String {
    let mut import_file = String::new();

    if file.contains("#[serde(with = \"ts_seconds_option\")]")
        || file.contains("#[serde(with = \"ts_seconds\")]")
    {
        import_file.push_str("use tiberius::time::chrono::{DateTime, Utc};\n");
        if file.contains("#[serde(with = \"ts_seconds_option\")]") {
            import_file.push_str("use chrono::serde::ts_seconds_option;\n");
        }
        if file.contains("#[serde(with = \"ts_seconds\")]") {
            import_file.push_str("use chrono::serde::ts_seconds;\n");
        }
    }
    import_file
}

fn get_column_name(column: &ColumnName) -> String {
    match column
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
    }
}
