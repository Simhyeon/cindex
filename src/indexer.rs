use std::io::Write;
use std::{io::Read, fs::File};
use std::collections::HashMap;
#[cfg(feature = "rayon")]
use rayon::prelude::*;

use crate::{CIndexResult, CIndexError, consts};
use crate::models::{CsvType, Table, Query};

/// Entry struct for indexing csv tables
pub struct Indexer {
    print_header: bool,
    always_unix_newline: bool,
    tables: HashMap<String, Table>,
}

impl Indexer {
    /// Create new indexer
    pub fn new() -> Self {
        Self {
            print_header: true,
            always_unix_newline: false,
            tables: HashMap::new(),
        }
    }

    /// Decide whether header to be printed or not
    pub fn set_print_header(&mut self, tv: bool) {
        self.print_header = tv;
    }

    /// Always use unix newline for formatting
    pub fn always_use_unix_newline(&mut self, tv:bool) {
        self.always_unix_newline = tv;
    }

    fn get_newline(&self) -> &str {
        if self.always_unix_newline {
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
    pub fn add_table_fast(&mut self, table_name: &str, input: impl Read) -> CIndexResult<()>{
        self.add_table(table_name, vec![], input)
    }

    /// Add table
    pub fn add_table(&mut self, table_name: &str, header_types: Vec<CsvType>, mut input: impl Read) -> CIndexResult<()> {
        let mut table_content = String::new(); 
        input.read_to_string(&mut table_content)?;

        let mut lines = table_content.lines();
        let headers : Vec<(String, CsvType)>;
        let mut rows  = vec![];

        if let Some(headers_line) = lines.next() {
            // Pad headers if heade's length is longer than header_types

            let header_types_iter = header_types[0..].iter().chain(std::iter::repeat(&CsvType::Text));
            let header_lines_iter = headers_line.split(',');

            // NOTE
            // Technically speaking, types can be bigger than header values length
            // But it yields expectable behaviour, thus it stays as it is.
            let len = header_lines_iter.clone().collect::<Vec<&str>>().len();

            headers = header_types_iter.zip(header_lines_iter).take(len).map(|(value,t)| (t.to_owned(), *value)).collect();
        } else {
            return Err(CIndexError::InvalidTableInput(format!("No header option is not supported")));
        }

        for line in lines {
            let row: Vec<&str> = line.split(',').collect();

            if row.len() != headers.len() {
                return Err(CIndexError::InvalidTableInput(format!("Row's length \"{}\" is different from header's length \"{}\"", row.len(), headers.len())));
            }

            rows.push(row);
        }

        self.tables.insert(table_name.to_owned(), Table::new(headers, rows)?);
        Ok(())
    }

    //<INDEXING>
    /// Index with raq query
    pub fn index_raw(&self, raw_query: &str, out_option: OutOption) -> CIndexResult<()> {
        self.index(Query::from_str(raw_query)?, out_option)
    }

    /// Index with pre-built query
    pub fn index(&self, query: Query, mut out_option: OutOption) -> CIndexResult<()> {
        let table = self.tables.get(query.table_name.as_str()).ok_or(CIndexError::InvalidTableName(format!("Table \"{}\" doesn't exist", query.table_name)))?;

        let filtered_rows = table.query(&query)?;
        let mut rows_iter = filtered_rows.iter();
        let columns: Option<Vec<usize>> = if let Some(ref cols) = query.column_names {
            if cols.len() > 0 && cols[0] == "*" { None }
            else {
                #[cfg(feature = "rayon")]
                let iter = cols.par_iter();
                #[cfg(not(feature = "rayon"))]
                let iter = cols.iter();

                let collection : Result<Vec<usize>,CIndexError> = iter.map(|i| -> Result<usize, CIndexError> {
                    Ok(table.headers.get_index_of(i).ok_or(CIndexError::InvalidColumn(format!("No such column \"{}\"", i)))?)
                }).collect();
                Some(collection?)
            }
        } else { None };

        // Print headers
        if self.print_header {
            let columns = query.column_names.unwrap_or(vec!["*".to_owned()]);
            let map = query.column_map.unwrap_or(vec![]);
            let mapped_columns : String;
            if columns[0] == "*" {
                let headers = table.headers
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                mapped_columns = map.iter()
                    .chain(headers[map.len()..].iter())
                    .map(|s| s.as_str())
                    .collect::<Vec<&str>>().join(",");
            } else {
                mapped_columns = map.iter()
                    .chain(columns[map.len()..].iter())
                    .map(|s| s.as_str())
                    .collect::<Vec<&str>>()
                    .join(",");
            }

            self.write(&(mapped_columns + self.get_newline()), &mut out_option)?;
        }

        while let Some(&row) = rows_iter.next() {
            let row_string = if let Some(cols) = &columns {
                row.column_splited(cols) + self.get_newline()
            } else {
                row.to_string() + self.get_newline()
            };
            self.write(&row_string, &mut out_option)?;
        }
        Ok(())
    }

    fn write(&self, content: &str, out_option: &mut OutOption) -> CIndexResult<()> {
        match out_option {
            OutOption::Term => print!("{}", content),
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
