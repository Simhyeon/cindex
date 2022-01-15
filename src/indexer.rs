use std::{path::PathBuf, collections::HashMap, io::Read};
use crate::models::{CSVType, CSVTable, Query};

pub struct Indexer<'indexer> {
    read_option: ReadOption,
    tables: HashMap<&'indexer str, CSVTable>,
}

impl<'indexer> Indexer<'indexer> {
    pub fn new() -> Self {
        Self {
            read_option: ReadOption::Undefined,
            tables: HashMap::new(),
        }
    }

    // TODO
    // Add header_types arguments
    pub fn add_table(&mut self, table_name: &'indexer str, header_types: Vec<CSVType>, read_option: ReadOption) {
        let mut table_content = String::new();
        match read_option {
            ReadOption::Undefined => eprintln!("Read option is undefined"),
            ReadOption::File(path) => {
                table_content = std::fs::read_to_string(path).expect("Failed to read file");
            },
            ReadOption::Stdin => {
                let stdin  = std::io::stdin();
                stdin.lock()
                    .read_to_string(&mut table_content)
                    .expect("Failed to read content from stdio");
            },
            ReadOption::Value(var) => {
                table_content = var;
            },
        }

        let mut lines = table_content.lines();
        let headers : Vec<(String, CSVType)>;
        let mut rows  = vec![];

        // TODO 
        // Currently every type is string and is not configurable
        if let Some(headers_line) = lines.next() {
            headers = header_types[0..].iter().zip(headers_line.split(',')).map(|(value,t)| (t.to_owned(), *value)).collect();
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

        self.tables.insert(table_name, CSVTable::new(table_name, headers, rows));
    }

    pub fn index(&self, table_name: &str, query: Option<Query>) {
        let table = self.tables.get(table_name).expect("Failed to get table by name");

        if let Some(query) = query {
            let rows = table.query(&query);
            let mut rows_iter = rows.iter().peekable();
            if let Some(&row) = rows_iter.next() {
                println!("{}", row);
            }

            while let Some(&row) = rows_iter.next() {
                println!("{}", row);
            }

        } else {
            println!("{}",table);
        }
    }
}

pub enum ReadOption {
    Stdin,
    Value(String),
    File(PathBuf),
    /// This is technically a default option and not used as variant
    Undefined,
}
