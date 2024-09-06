use serde::{Deserialize, Serialize};

use crate::helpers::common::{
    convert_text_first_char_to_uppercase, convert_text_first_char_to_uppercase_else_lowercase,
};

use super::args_parser::AuthType;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct TableName {
    pub table_catalog: String,
    pub table_schema: String,
    pub table_name: String,
    pub table_type: String,
}

impl TableName {
    pub fn get_table_name(&self) -> String {
        format!(
            "{}{}{}",
            convert_text_first_char_to_uppercase(self.table_catalog.as_str()),
            convert_text_first_char_to_uppercase(self.table_schema.as_str()),
            convert_text_first_char_to_uppercase(self.table_name.as_str())
        )
    }
    pub fn get_table_name_dart(&self) -> String {
        format!(
            "{}{}{}",
            convert_text_first_char_to_uppercase_else_lowercase(self.table_catalog.as_str()),
            convert_text_first_char_to_uppercase_else_lowercase(self.table_schema.as_str()),
            convert_text_first_char_to_uppercase_else_lowercase(self.table_name.as_str())
        )
    }
    pub fn get_file_name(&self) -> String {
        format!(
            "{}_{}_{}",
            self.table_catalog.to_lowercase(),
            self.table_schema.to_lowercase(),
            self.table_name.to_lowercase()
        )
    }
    pub fn get_sql_table_name(&self) -> String {
        format!(
            "{}.{}.{}",
            self.table_catalog, self.table_schema, self.table_name
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct ColumnName {
    pub table_catalog: String,
    pub table_schema: String,
    pub table_name: String,
    pub column_name: String,
    pub ordinal_position: i32,
    pub column_default: Option<String>,
    pub is_nullable: String,
    pub data_type: String,
    pub character_maximum_length: Option<i32>,
    pub character_octet_length: Option<i32>,
    pub numeric_precision: Option<u8>,
    pub numeric_precision_radix: Option<i16>,
    pub numeric_scale: Option<i32>,
    pub datetime_precision: Option<i16>,
    pub character_set_catalog: Option<String>,
    pub character_set_schema: Option<String>,
    pub character_set_name: Option<String>,
    pub collation_catalog: Option<String>,
    pub collation_schema: Option<String>,
    pub collation_name: Option<String>,
    pub domain_catalog: Option<String>,
    pub domain_schema: Option<String>,
    pub domain_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Table {
    pub name: TableName,
    pub columns: Vec<ColumnName>,
    #[serde(default = "use_proto_parser_default")]
    pub use_proto_parser: bool,
    #[serde(default = "use_proto_file_default")]
    pub use_proto_file: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InnerArgs {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database_name: Option<String>,
    pub application_name: Option<String>,
    pub instance_name: Option<String>,
    pub user: String,
    pub password: String,
    #[serde(rename = "type")]
    pub _type: AuthType,
    #[serde(default = "use_proto_parser_default")]
    pub use_proto_parser: bool,
    #[serde(default = "use_split_file_default")]
    pub use_split_file: bool,
    pub path: Option<String>,
    pub proto_path: Option<String>,
    pub database: Option<DatabaseConfig>,
}

fn use_proto_parser_default() -> bool {
    false
}

fn use_split_file_default() -> bool {
    false
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DatabaseConfig {
    #[serde(default = "use_import_special_default")]
    pub use_import_special: bool,
    pub tables: Option<Vec<TableConfig>>,
}

fn use_import_special_default() -> bool {
    false
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TableConfig {
    pub table_name: String,
    #[serde(default = "use_proto_parser_default")]
    pub use_proto_parser: bool,
    #[serde(default = "use_proto_file_default")]
    pub use_proto_file: bool,
}

fn use_proto_file_default() -> bool {
    true
}
