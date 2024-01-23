use tiberius::{Column, ColumnData, Row};

#[derive(Debug)]
pub struct SelectParser<'a> {
    pub columns: Vec<Column>,
    pub rows: Vec<Vec<ColumnData<'a>>>,
}

pub trait SelectParserTrait<'a> {
    fn select_parser(&mut self) -> SelectParser<'a>;
}

impl<'a> SelectParserTrait<'a> for Vec<Vec<Row>> {
    fn select_parser(&mut self) -> SelectParser<'a> {
        match self.pop() {
            Some(rows) => {
                let mut columns_set = false;
                let mut columns = Vec::new();
                let mut result_rows = Vec::with_capacity(rows.len());

                for row in rows.into_iter() {
                    if !columns_set {
                        columns = row.columns().to_vec();
                        columns_set = true;
                    }

                    let mut values: Vec<ColumnData<'_>> = Vec::with_capacity(row.len());

                    for val in row.into_iter() {
                        values.push(val);
                    }

                    result_rows.push(values);
                }

                SelectParser {
                    columns: columns,
                    rows: result_rows,
                }
            }
            None => SelectParser {
                columns: Vec::new(),
                rows: Vec::new(),
            },
        }
    }
}
