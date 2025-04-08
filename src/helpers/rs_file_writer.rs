use std::{collections::HashMap, env, path::PathBuf, str::FromStr};

use tokio::{fs::File, io::AsyncWriteExt};

use crate::helpers::{config::STRUCT_FILE_NAME, structs::Table};

use super::{
    common::{get_table_names, write_files},
    signal_file_writer::get_column_name,
    structs::SplitDirectoryConfig,
    traits::StringUtil,
};

pub async fn rs_file_writer(
    path: &Option<String>,
    use_split_file: bool,
    table_list: &Vec<Table>,
    split_directorys: &Vec<SplitDirectoryConfig>,
) -> Result<(), Box<dyn std::error::Error>> {
    match use_split_file {
        true => rs_split_file_writer(path, table_list, split_directorys).await?,
        false => rs_one_file_writer(path, table_list).await?,
    }
    Ok(())
}

pub async fn rs_one_file_writer(
    path: &Option<String>,
    table_list: &Vec<Table>,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = File::create(match path {
        Some(path) => PathBuf::from_str(path.as_str())?,
        None => env::current_dir()?.join(STRUCT_FILE_NAME),
    })
    .await?;
    let mut writer: tokio::io::BufWriter<File> = tokio::io::BufWriter::new(result);
    let mut file: String = "\n".into();

    let mut use_insert_query = false;
    for table in table_list {
        let (table_name, _file_name, sql_table_name) = get_table_names(table);
        file.push_str(make_struct(table_name.as_str(), sql_table_name.as_str(), table).as_str());
        file.push_str(&make_columns(sql_table_name.as_str(), table));

        if table.use_signal_parser {
            file.push_str(&make_signal_parser(table, &table_name, &table_name));
        }

        if table.use_insert_query {
            use_insert_query = true;
        }
    }
    file.pop();

    file = format!("{}{}", import_file(&file, use_insert_query), file);

    writer.write_all(file.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

pub async fn rs_split_file_writer(
    path: &Option<String>,
    table_list: &Vec<Table>,
    split_directorys: &Vec<SplitDirectoryConfig>,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = match path {
        Some(path) => PathBuf::from_str(path.as_str())?,
        None => env::current_dir()?.join("sample"),
    };
    let mut file_list: HashMap<PathBuf, String> = HashMap::new();
    let mut mod_list: HashMap<String, Vec<String>> = HashMap::new();

    for table in table_list {
        let mut file: String = "\n".into();
        let (table_name, file_name, sql_table_name) = get_table_names(table);

        file.push_str(&make_struct(
            table_name.as_str(),
            sql_table_name.as_str(),
            table,
        ));
        file.push_str(&make_columns(sql_table_name.as_str(), table));

        if table.use_signal_parser {
            file.push_str(&make_signal_parser(table, &table_name, &table_name));
        }
        file.pop();

        file = format!("{}{}", import_file(&file, table.use_insert_query), file);

        let current_path = match split_directorys.is_empty() {
            true => path.join(format!("{}.rs", file_name)),
            false => {
                let mut current_path = path.clone();
                for split_directory in split_directorys {
                    if file_name.starts_with(&split_directory.starts_with_name) {
                        current_path =
                            current_path.join(split_directory.directory_name.copy_string());
                        match mod_list.get_mut(current_path.to_str().unwrap()) {
                            Some(mod_list) => {
                                mod_list.push(file_name.copy_string());
                            }
                            None => {
                                mod_list.insert(
                                    current_path.to_str().unwrap().into(),
                                    vec![file_name.copy_string()],
                                );
                            }
                        }
                        break;
                    }
                }
                current_path.join(format!("{}.rs", file_name))
            }
        };
        file_list.insert(current_path, file);
    }
    write_mod_files(mod_list).await?;
    write_files(file_list).await?;

    Ok(())
}

async fn write_mod_files(
    mod_list: HashMap<String, Vec<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file_list: HashMap<PathBuf, String> = HashMap::new();
    for (path, file_names) in mod_list {
        let path = PathBuf::from_str(path.as_str())?;
        let mut file: String = "".into();
        for file_name in file_names {
            file.push_str(&format!("pub mod {};\n", file_name));
        }
        file_list.insert(path.join("mod.rs"), file);
    }
    write_files(file_list).await?;
    Ok(())
}

fn make_signal_parser(table: &Table, table_name: &str, table_name_dart: &str) -> String {
    let mut file: String = "".into();
    file.push_str(&format!("impl {} ", table_name));
    file.push_str("{\n");

    file.push_str(&format!(
        "    pub fn to_dart(self) -> crate::signals::{} {}",
        &table_name_dart, "{\n"
    ));
    file.push_str(&format!(
        "        crate::signals::{} {}",
        &table_name_dart, "{\n"
    ));
    for column in &table.columns {
        let column_name = get_column_name(column);
        let data_type = match column.data_type.as_str() {
            "datetime" => ".to_rfc3339()",
            _ => "",
        };

        file.push_str(&format!(
            "            {}: {},\n",
            &column_name,
            match column.is_nullable.as_str() == "YES" {
                true => match column.data_type.as_str() {
                    "datetime" => make_matcher(
                        &format!("Some(Into::into(&*value{}))", data_type),
                        &column_name
                    ),
                    _ => format!("self.{}{}", column_name, data_type),
                },
                false => format!("self.{}{}", column_name, data_type),
            }
        ));
    }
    file.push_str("        }\n");
    file.push_str("    }\n");
    file.push_str("}\n\n");
    file
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

fn make_struct(table_name: &str, sql_table_name: &str, table: &Table) -> String {
    let mut file = String::new();
    file.push_str("#[allow(non_snake_case, non_camel_case_types)]\n");

    if !table.use_insert_query {
        file.push_str("#[derive(Serialize, Deserialize, Debug, Clone)]\n");
    } else {
        file.push_str(
            "#[derive(Serialize, Deserialize, InsertQuery, TableSchema, Debug, Clone)]\n",
        );
        file.push_str(&format!("#[table_name = \"{}\"]\n", sql_table_name));
    }
    file.push_str(&format!("pub struct {} {{\n", table_name));
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
            "bit" => "bool",
            "tinyint" => "u8",
            "smallint" => "i16",
            "int" => "i32",
            "bigint" => "i64",
            "real" => "f32",
            "float" => "f64",
            "money" => "f64",
            "datetime" => "DateTime<Utc>",
            "binary" => "Vec<u8>",
            "image" => "Vec<u8>",
            "ntext" => "String",
            "nvarchar" => "String",
            "text" => "String",
            _ => "String",
        };

        file.push_str(&format!(
            "    pub {}: {},\n",
            column_name,
            match column.is_nullable.as_str() == "YES" {
                true => format!("Option<{}>", data_type),
                false => data_type.into(),
            }
        ));
    }
    file.push_str("}\n\n");
    file
}

fn make_columns(table_name: &str, table: &Table) -> String {
    let mut file = String::new();
    let table_name_uppercase = table.name.table_name.to_uppercase();
    file.push_str(&format!(
        "pub const {}_TABLE_NAME: &'static str = \"{}\";\n\n",
        table_name_uppercase, table_name
    ));
    file.push_str(&format!(
        "pub const {}_COLUMNS: &'static str = \"\n",
        table_name_uppercase
    ));
    let mut index = 0;
    for column in &table.columns {
        let column_name = get_column_name(column);

        if index != 0 {
            file.push_str(",");
        }
        file.push_str(&format!("[{}]\n", column_name,));
        index += 1;
    }
    file.push_str("\";\n\n");
    file
}

fn import_file(file: &str, use_insert_query: bool) -> String {
    let mut import_file = String::new();

    import_file.push_str("use serde::{Deserialize, Serialize};\n");
    if file.contains("#[serde(with = \"ts_seconds_option\")]")
        || file.contains("#[serde(with = \"ts_seconds\")]")
    {
        if file.contains("#[serde(with = \"ts_seconds_option\")]") {
            import_file.push_str("use chrono::serde::ts_seconds_option;\n");
        }
        if file.contains("#[serde(with = \"ts_seconds\")]") {
            import_file.push_str("use chrono::serde::ts_seconds;\n");
        }

        if !use_insert_query {
            import_file.push_str("use tiberius::time::chrono::{DateTime, Utc};\n");
        } else {
            import_file.push_str("use table_schema_derive::{InsertQuery, TableSchema};\n");
            import_file.push_str("use table_schema_traits::{InsertQuery, TableSchema};\n");
            import_file.push_str("use tiberius::{\n");
            import_file.push_str("    time::chrono::{DateTime, Utc},\n");
            import_file.push_str("    ToSql,\n");
            import_file.push_str("};\n");
        }
    }
    import_file
}
