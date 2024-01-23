use tiberius::ColumnData;

use crate::helpers::traits::select_parser::SelectParser;
use serde::de::DeserializeOwned;

pub fn get_table_schema<T>(select_parser: SelectParser<'_>) -> Vec<T>
where
    T: DeserializeOwned,
{
    let mut select_list: Vec<T> = Vec::new();
    select_parser.rows.iter().for_each(|row| {
        let mut json: String = "{".into();
        let mut index = 0;
        let total_count = &select_parser.columns.len();
        for column in &select_parser.columns {
            let row_data = row.get(index).unwrap();
            if let ColumnData::String(Some(data)) = row_data {
                json.push_str(format!("\"{}\": \"{}\"", column.name(), data).as_str());
            } else if let ColumnData::I32(Some(data)) = row_data {
                json.push_str(format!("\"{}\": {}", column.name(), data.to_string()).as_str());
            } else if let ColumnData::I16(Some(data)) = row_data {
                json.push_str(format!("\"{}\": {}", column.name(), data.to_string()).as_str());
            } else if let ColumnData::U8(Some(data)) = row_data {
                json.push_str(format!("\"{}\": {}", column.name(), data.to_string()).as_str());
            } else {
                json.push_str(format!("\"{}\": null", column.name()).as_str());
            }

            if index == total_count - 1 {
                json.push_str("}");
            } else {
                json.push_str(",");
            }
            index += 1;
        }
        let json: T = serde_json::from_str(&json).unwrap();
        select_list.push(json);
    });
    select_list
}
