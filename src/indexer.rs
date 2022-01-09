use std::{path::PathBuf, collections::HashMap};

pub struct Indexer<'indexer> {
    read_option: ReadOption,
    tables: HashMap<&'indexer str,CSVTable<'indexer>>,
}

pub struct IndexerQuery<'query> {
    target_table: &'query str,
    predicates: Option<Vec<Predicate<'query>>>,
    joined_tables: Option<Vec<&'query str>>,
}

pub struct Predicate<'predicate> {
    column: &'predicate str,
    operation: Operation,
    argument: Vec<String>,
}

impl<'predicate> Predicate<'predicate> {
    pub fn new(column: &'predicate str, operation: Operation, argument: Vec<String>) -> Self {
        Self {
            column,
            operation,
            argument,
        }
    }
}

pub enum Operation {
    Bigger,
    BiggerOrEqual,
    Smaller,
    SmallerOrEqual,
    Equal,
    NotEqual,
    Between,
    In,
}

pub struct CSVTable<'table> {
    name: &'table str,
    rows: Vec<CSVRow<'table>>
}

pub struct CSVRow<'row> {
    row: Vec<CSVData<'row>>,
}

pub struct CSVData<'data> {
    data_type : CSVType,
    value: &'data str,
}

/// csv data Type
///
/// this is compatible with sqlite data types
pub enum CSVType {
    Null, // Seriously?
    Text, // String or say char bytes,
    Integer,
    Float, // f8
    BLOB, // Bytes
}

impl<'indexer> Indexer<'indexer> {

    //pub fn new() -> Self {
        //Self {
            //read_option: ReadOption::default(),
        //}
    //}

    pub fn index() -> String {
        format!("")
    }
}

pub enum ReadOption {
    Stdin,
    Value(String),
    File(PathBuf)
}

impl Default for ReadOption {
    fn default() -> Self {
        Self::Stdin
    }
}
