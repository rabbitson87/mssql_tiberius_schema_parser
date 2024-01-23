use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct TableName {
    pub table_catalog: String,
    pub table_schema: String,
    pub table_name: String,
    pub table_type: String,
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
}
