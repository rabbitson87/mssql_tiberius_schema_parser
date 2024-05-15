use chrono::{DateTime, Months};
use tiberius::ColumnData;

use crate::helpers::traits::select_parser::SelectParser;
use serde::de::DeserializeOwned;

pub trait GetTableSchema {
    fn get_table_schema<T>(self: &Self) -> Vec<T>
    where
        T: DeserializeOwned;
}

impl GetTableSchema for SelectParser<'_> {
    fn get_table_schema<T>(self: &Self) -> Vec<T>
    where
        T: DeserializeOwned,
    {
        let mut select_list: Vec<T> = Vec::new();
        self.rows.iter().for_each(|row| {
            let mut json: String = "{".into();
            let mut index = 0;
            let total_count = &self.columns.len();
            for column in &self.columns {
                if let Some(row_data) = row.get(index) {
                    match row_data {
                        ColumnData::String(Some(data)) => {
                            json.push_str(
                                format!(
                                    "\"{}\": \"{}\"",
                                    column.name(),
                                    data.replace("\"", "\\\"")
                                        .replace("\n", "\\\\n")
                                        .replace("\r", "\\\\r")
                                        .replace("\t", "\\\\t"),
                                )
                                .as_str(),
                            );
                        }
                        ColumnData::U8(Some(data)) => {
                            json.push_str(
                                format!("\"{}\": {}", column.name(), data.to_string()).as_str(),
                            );
                        }
                        ColumnData::I16(Some(data)) => {
                            json.push_str(
                                format!("\"{}\": {}", column.name(), data.to_string()).as_str(),
                            );
                        }
                        ColumnData::I32(Some(data)) => {
                            json.push_str(
                                format!("\"{}\": {}", column.name(), data.to_string()).as_str(),
                            );
                        }
                        ColumnData::I64(Some(data)) => {
                            json.push_str(
                                format!("\"{}\": {}", column.name(), data.to_string()).as_str(),
                            );
                        }
                        ColumnData::F32(Some(data)) => {
                            json.push_str(
                                format!("\"{}\": {}", column.name(), data.to_string()).as_str(),
                            );
                        }
                        ColumnData::F64(Some(data)) => {
                            json.push_str(
                                format!("\"{}\": {}", column.name(), data.to_string()).as_str(),
                            );
                        }
                        ColumnData::Numeric(Some(data)) => {
                            json.push_str(
                                format!("\"{}\": {}", column.name(), data.to_string()).as_str(),
                            );
                        }
                        ColumnData::DateTime(Some(data)) => {
                            let date = DateTime::from_timestamp(
                                (data.days() as i64) * 24 * 60 * 60
                                    + (data.seconds_fragments() as i64) / 300,
                                0,
                            )
                            .unwrap();
                            let date = date.checked_sub_months(Months::new(840)).unwrap();
                            json.push_str(
                                format!("\"{}\": {}", column.name(), date.timestamp().to_string())
                                    .as_str(),
                            );
                        }
                        ColumnData::DateTimeOffset(Some(data)) => {
                            let seconds = (data.datetime2().time().increments() as i64)
                                / i64::pow(10, data.datetime2().time().scale() as u32);
                            let milliseconds = ((data.datetime2().time().increments() as i64)
                                % i64::pow(10, data.datetime2().time().scale() as u32))
                                * 100;
                            let milliseconds = milliseconds - milliseconds % 1000;
                            let date = DateTime::from_timestamp(
                                (data.datetime2().date().days() as i64) * 24 * 60 * 60
                                    - (get_days_from_years(1969) * 24 * 60 * 60)
                                    + seconds,
                                milliseconds as u32,
                            )
                            .unwrap();
                            json.push_str(
                                format!("\"{}\": {}", column.name(), date.timestamp().to_string())
                                    .as_str(),
                            );
                        }
                        ColumnData::Bit(data) => {
                            json.push_str(
                                format!(
                                    "\"{}\": {}",
                                    column.name(),
                                    match data {
                                        Some(child_data) => match child_data {
                                            true => "true",
                                            false => "false",
                                        },
                                        None => "false",
                                    }
                                )
                                .as_str(),
                            );
                        }
                        _ => {
                            json.push_str(format!("\"{}\": null", column.name()).as_str());
                        }
                    }
                }

                if index == total_count - 1 {
                    json.push_str("}");
                    break;
                } else {
                    json.push_str(",");
                }
                index += 1;
            }
            match serde_json::from_str(&json) {
                Ok(json) => select_list.push(json),
                Err(e) => {
                    println!("Error: {}", e);
                }
            };
        });
        select_list
    }
}

pub fn get_days_from_years(years: i64) -> i64 {
    let mut days = years * 365;
    days += years.div_euclid(4);
    days -= years.div_euclid(100);
    days += years.div_euclid(400);
    days
}
