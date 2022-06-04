use crate::models::ColumnVariant;
use crate::query::{Query, QueryFlags};
use crate::table::Table;
use crate::{consts, CIndexError, CIndexResult};
use dcsv::Row;
#[cfg(feature = "rayon")]
use rayon::prelude::*;
use std::collections::HashMap;
use std::io::Write;
use std::{fs::File, io::Read};

/// Entry struct for indexing csv tables
pub struct Indexer {
    tables: HashMap<String, Table>,
    use_unix_newline: bool,
}

impl Indexer {
    /// Create new indexer
    pub fn new() -> Self {
        Self {
            use_unix_newline: false,
            tables: HashMap::new(),
        }
    }

    /// Always use unix newline for formatting
    pub fn always_use_unix_newline(&mut self, tv: bool) {
        self.use_unix_newline = tv;
    }

    /// Return newline with unix newline option considered
    fn get_newline(&self) -> &str {
        if self.use_unix_newline {
            "\n"
        } else {
            consts::LINE_ENDING
        }
    }

    /// Check if indexer contains table
    pub fn contains_table(&self, table_name: &str) -> bool {
        self.tables.contains_key(table_name)
    }

    /// Drop table
    pub fn drop_table(&mut self, table_name: &str) {
        self.tables.remove(table_name);
    }

    /// Add table without header
    pub fn add_table_fast(&mut self, table_name: &str, input: impl Read) -> CIndexResult<()> {
        self.add_table_internal(table_name, input)
    }

    fn add_table_internal(&mut self, table_name: &str, mut input: impl Read) -> CIndexResult<()> {
        let mut table_content = String::new();
        input.read_to_string(&mut table_content)?;
        self.tables
            .insert(table_name.to_owned(), Table::new(table_content.as_bytes())?);
        Ok(())
    }

    //<INDEXING>
    /// Index with raq query
    pub fn index_raw(&self, raw_query: &str, out_option: OutOption) -> CIndexResult<()> {
        self.index(Query::from_str(raw_query)?, out_option)
    }

    /// Index with pre-built query
    pub fn index(&self, query: Query, mut out_option: OutOption) -> CIndexResult<()> {
        let records = self.index_table(query)?;

        for row in records {
            self.write(&(row.join(",") + self.get_newline()), &mut out_option)?;
        }
        Ok(())
    }

    /// Get rows filtered by query
    pub fn index_get_records(&self, query: Query) -> CIndexResult<Vec<Vec<String>>> {
        let records = self.index_table(query)?;
        Ok(records)
    }

    /// Internal function
    fn index_table(&self, query: Query) -> CIndexResult<Vec<Vec<String>>> {
        let mut mapped_records: Vec<Vec<String>> = vec![];
        let table =
            self.tables
                .get(query.table_name.as_str())
                .ok_or(CIndexError::InvalidTableName(format!(
                    "Table \"{}\" doesn't exist",
                    query.table_name
                )))?;
        let queried_records = table.query(&query)?;

        let target_columns: Option<Vec<ColumnVariant>> = if let Some(ref cols) = query.column_names
        {
            if cols.len() > 0 && cols[0] == "*" {
                None
            } else {
                #[cfg(feature = "rayon")]
                let iter = cols.par_iter();
                #[cfg(not(feature = "rayon"))]
                let iter = cols.iter();

                // If supplement is given
                // add extra columns
                let supplement = query.flags.contains(QueryFlags::SUP);
                let collection: Vec<_> = if supplement {
                    iter.map(|i| {
                        if let Some(col) = table.header.get(i) {
                            ColumnVariant::Real(col)
                        } else {
                            ColumnVariant::Supplement
                        }
                    })
                    .collect()
                } else {
                    iter.map(|i| -> Result<ColumnVariant, CIndexError> {
                        let index = ColumnVariant::Real(table.header.get(i).ok_or(
                            CIndexError::InvalidColumn(format!("No such column \"{}\"", i)),
                        )?);
                        Ok(index)
                    })
                    .collect::<CIndexResult<Vec<_>>>()?
                };

                Some(collection)
            }
        } else {
            None
        };

        // Print headers
        if query.flags.contains(QueryFlags::PHD) {
            mapped_records.push(table.data.columns.iter().map(|c| c.name.clone()).collect());
        }

        for record in queried_records {
            if let Some(cols) = &target_columns {
                mapped_records.push(self.row_with_columns(record, cols))
            } else {
                mapped_records.push(
                    table
                        .data
                        .columns
                        .iter()
                        .map(|col| record.get_cell_value(&col.name).unwrap().to_string())
                        .collect(),
                )
            }
        }

        // Tranpose if given TP Flag
        if query.flags.contains(QueryFlags::TP) {
            mapped_records = self.tranpose_records(mapped_records);
        }

        Ok(mapped_records
            .iter_mut()
            .map(|vec| vec.iter_mut().map(|val| val.to_string()).collect())
            .collect())
    }

    fn row_with_columns(&self, row: &Row, columns: &Vec<ColumnVariant>) -> Vec<String> {
        let mut formatted = vec![];

        for col in columns {
            if let ColumnVariant::Real(key) = col {
                formatted.push(row.get_cell_value(key).unwrap().to_string());
            }
        }
        formatted
    }

    // Tranpose
    // https://stackoverflow.com/questions/64498617/how-to-transpose-a-vector-of-vectors-in-rust
    // Thank you stackoverflow ;)
    fn tranpose_records<'a>(&'a self, v: Vec<Vec<String>>) -> Vec<Vec<String>> {
        let len = v[0].len();
        let mut iters: Vec<_> = v.into_iter().map(|n| n.into_iter()).collect();
        (0..len)
            .map(|_| {
                iters
                    .iter_mut()
                    .map(|n| n.next().unwrap())
                    .collect::<Vec<String>>()
            })
            .collect()
    }

    fn write(&self, content: &str, out_option: &mut OutOption) -> CIndexResult<()> {
        match out_option {
            OutOption::Term => std::io::stdout().write_all(content.as_bytes())?,
            OutOption::File(file) => file.write_all(content.as_bytes())?,
            OutOption::Value(target) => target.push_str(content),
        }
        Ok(())
    }
}

/// Ouput redirect option
pub enum OutOption<'a> {
    Term,
    Value(&'a mut String),
    File(File),
}
