use std::io::Write;
use std::{io::Read, fs::File};
use std::collections::HashMap;
use rayon::prelude::*;

use crate::{CIndexResult, CIndexError};
use crate::models::{CSVType, CSVTable, Query};

pub struct Indexer {
    tables: HashMap<String, CSVTable>,
}

impl Indexer {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    /// Add table without header
    pub fn add_table_fast(&mut self, table_name: &str, input: impl Read) -> CIndexResult<()>{
        self.add_table(table_name, vec![], input)
    }

    /// Add table
    pub fn add_table(&mut self, table_name: &str, header_types: Vec<CSVType>, mut input: impl Read) -> CIndexResult<()> {
        let mut table_content = String::new(); 
        input.read_to_string(&mut table_content)?;

        let mut lines = table_content.lines();
        let headers : Vec<(String, CSVType)>;
        let mut rows  = vec![];

        if let Some(headers_line) = lines.next() {
            // Pad headers if heade's length is longer than header_types

            let header_types_iter = header_types[0..].iter().chain(std::iter::repeat(&CSVType::Text));
            let header_lines_iter = headers_line.split(',');

            // NOTE
            // Technically speaking, types can be bigger than header values length
            // But it yields expectable behaviour, thus it stays as it is.
            let len = header_lines_iter.clone().collect::<Vec<&str>>().len();

            headers = header_types_iter.zip(header_lines_iter).take(len).map(|(value,t)| (t.to_owned(), *value)).collect();
        } else {
            panic!("No header option is not supported");
        }

        for line in lines {
            let row: Vec<&str> = line.split(',').collect();

            if row.len() != headers.len() {
                panic!("Row's length is different from header's length ");
            }

            rows.push(row);
        }

        self.tables.insert(table_name.to_owned(), CSVTable::new(headers, rows)?);
        Ok(())
    }

    //<INDEXING>
    pub fn index_raw(&self, raw_query: &str, out_option: OutOption) -> CIndexResult<()> {
        self.index(Query::from_str(raw_query), out_option)
    }

    pub fn index(&self, query: Query, mut out_option: OutOption) -> CIndexResult<()> {
        let table = self.tables.get(query.table_name.as_str()).ok_or(CIndexError::InvalidTableName(format!("Table \"{}\" doesn't exist", query.table_name)))?;

        let filtered_rows = table.query(&query)?;
        let mut rows_iter = filtered_rows.iter();
        let columns: Option<Vec<usize>> = if let Some(cols) = query.column_names {
            if cols.len() > 0 && cols[0] == "*" { None }
            else {
                // TODO 
                let collection : Result<Vec<usize>,CIndexError> = cols.par_iter().map(|i| -> Result<usize, CIndexError> {
                    Ok(table.headers.get_index_of(i).ok_or(CIndexError::InvalidColumn(format!("No such column \"{}\"", i)))?)
                }).collect();
                Some(collection?)
            }
        } else { None };

        if let Some(cols) = &columns {
            // Print headers
            self.write(&(self.header_splited(&table, &cols) + "\n"), &mut out_option)?;
        }

        while let Some(&row) = rows_iter.next() {
            let row_string = if let Some(cols) = &columns {
                row.column_splited(cols) + "\n"
            } else {
                row.to_string() + "\n"
            };
            self.write(&row_string, &mut out_option)?;
        }
        Ok(())
    }

    fn header_splited(&self, table : &CSVTable, columns: &Vec<usize>) -> String {
        let mut col_iter = columns.iter();
        let mut formatted = String::new();

        // First item
        if let Some(col) = col_iter.next() {
            formatted.push_str(&table.headers[*col].to_string());
        }

        while let Some(col) = col_iter.next() {
            formatted.push(',');
            formatted.push_str(&table.headers[*col].to_string());
        }
        formatted
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

pub enum OutOption<'a> {
    Term,
    Value(&'a mut String),
    File(File),
}
