use std::fmt::Display;
use crate::{parser::Parser, CIndexResult};
use rayon::prelude::*;
use crate::error::CIndexError;
use indexmap::IndexSet;

#[derive(Debug)]
pub struct CSVTable {
    pub(crate) headers: IndexSet<String>,
    pub(crate) rows: Vec<CSVRow>,
}

impl CSVTable {
    pub fn new(headers: Vec<(String, CSVType)>, rows: Vec<Vec<&str>>) -> CIndexResult<Self> {
        // Make this rayon compatible iterator
        let rows : CIndexResult<Vec<CSVRow>> = rows.iter().map(|row| {
            CSVRow::new(&headers, row)
        }).collect();

        let mut header_set : IndexSet<String> = IndexSet::new();

        for (header,_) in headers.iter() {
            header_set.insert(header.to_owned());
        }

        Ok(Self {
            headers: header_set,
            rows: rows?,
        })
    }

    pub fn query(&self, query: &Query) -> CIndexResult<Vec<&CSVRow>> {
        let boilerplate = vec![];
        let predicates = if let Some(pre) = query.predicates.as_ref() {
            for item in pre {
                if !self.headers.contains(&item.column) {
                    return Err(CIndexError::InvalidColumn(format!("Failed to get column \"{}\" from header", item.column)));
                }
            }
            pre
        } else { &boilerplate };

        // TODO
        // Can it be improved?
        let queried : Vec<_> = self.rows.par_iter().filter_map(|row| {
            match row.filter(&self.headers,&predicates) {
                Ok(boolean) => {
                    if boolean { Some(row) } else { None }
                },
                Err(err) => {
                    eprintln!("{}", err);
                    None
                },
            } 
        }).collect();

        Ok(queried)
    }
}

impl Display for CSVTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n",self.headers.iter().map(|s| s.as_str()).collect::<Vec<&str>>().join(","))?;

        for row in &self.rows {
            write!(f, "{}\n",row)?;
        }
        write!(f,"")
    }
}

#[derive(Debug)]
pub struct CSVRow {
    data: Vec<CSVData>,
}

impl Display for CSVRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data.par_iter().map(|datum| datum.value.as_str()).collect::<Vec<&str>>().join(","))
    }
} 

impl CSVRow {
    pub fn new(headers: &Vec<(String, CSVType)>, row: &Vec<&str>) -> CIndexResult<Self> {
        let mut data : Vec<CSVData> = Vec::new();
        // Check index of headers and type check
        for (index, &item) in row.iter().enumerate() {
            data.push(CSVData::new(headers[index].1,item)?);
        }

        Ok(Self { data })
    }

    pub fn filter(&self, headers: &IndexSet<String>, predicates: &Vec<Predicate>) -> CIndexResult<bool> {
        let failed : Result<Vec<_>, CIndexError> = 
            predicates.par_iter().map(|pre|
                {
                    // This is safe to unwrap because table's query method alwasy check if column
                    // exists before filtering
                    let target = &self.data[headers.get_index_of(&pre.column).unwrap()];
                    target.operate(&pre.operation, &pre.arguments)
                }
            ).collect();

        let failed = failed?;
        let failed: Vec<_> = failed.iter().filter(|s| *s == &false).collect();

        // Failed is zero which means it has succeeded
        Ok(failed.len() == 0)
    }

    pub fn column_splited(&self, columns: &Vec<usize>) -> String {
        let mut col_iter = columns.iter();
        let mut formatted = String::new();

        // First item
        if let Some(col) = col_iter.next() {
            formatted.push_str(&self.data[*col].to_string());
        }

        while let Some(col) = col_iter.next() {
            formatted.push(',');
            formatted.push_str(&self.data[*col].to_string());
        }
        formatted
    }
}

#[derive(Debug)]
pub struct CSVData {
    data_type : CSVType,
    value: String,
}

impl Display for CSVData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}", self.value)
    }
}

impl CSVData {
    pub fn new(data_type: CSVType, value: &str) -> CIndexResult<Self> {
        let data = Self {
            data_type,
            value: value.to_owned(),
        };
        data.check_type()?;

        Ok(data)
    }

    pub fn check_type(&self) -> CIndexResult<()> {
        match self.data_type {
            CSVType::Null    => {
                if !self.value.is_empty() {
                    return Err(CIndexError::InvalidDataType(format!("Value \"{}\" is not NULL", self.value)));
                }
            },
            CSVType::Float   => {
                self.value.parse::<f32>().map_err(|_| CIndexError::InvalidDataType(format!("Value \"{}\" is not a floating point number", self.value)))?;
            },
            CSVType::Integer => {
                self.value.parse::<i32>().map_err(|_| CIndexError::InvalidDataType(format!("Value \"{}\" is not an integer", self.value)))?;
            },
            _ => (),
        }
        Ok(())
    }

    pub fn operate(&self, operation : &Operator, args : &Vec<String>) -> CIndexResult<bool> {
        if args.len() < 1 {
            eprintln!("ERROR!");
            panic!();
        }

        let (var, arg) = match self.data_type {
            CSVType::Null => (CSVVariant::Null,CSVVariant::Null),
            CSVType::Text => (CSVVariant::Text(&self.value), CSVVariant::Text(&args[0])),
            CSVType::Integer => {
                (
                    CSVVariant::Integer(self.value.parse().map_err(|_| CIndexError::TypeDiscord(format!("{} as integer", self.value)))?),
                    CSVVariant::Integer(args[0].parse().map_err(|_| CIndexError::TypeDiscord(format!("{} as integer", args[0])))?),
                )
            },
            CSVType::Float => {
                (
                    CSVVariant::Float(self.value.parse().map_err(|_| CIndexError::TypeDiscord(format!("{} as float", self.value)))?),
                    CSVVariant::Float(args[0].parse().map_err(|_| CIndexError::TypeDiscord(format!("{} as float", args[0])))?),
                )
            },
            CSVType::BLOB => (CSVVariant::BLOB(self.value.as_bytes()), CSVVariant::BLOB(&args[0].as_bytes())),
        };

        let result = match operation {
            Operator::Bigger => {
                var > arg
            },
            Operator::BiggerOrEqual => {
                var >= arg
            },
            Operator::Smaller => {
                var < arg
            },
            Operator::SmallerOrEqual => { 
                var <= arg
            },
            Operator::Equal => { 
                var == arg
            },
            Operator::NotEqual => {
                var != arg
            },
            Operator::In => {
                // TODO
                //args.contains(&self.value)
                true
            },
            Operator::Between => {
                // TODO
                //self.value >= args[0] && self.value <= args[1]
                true
            },
        };

        Ok(result)
    }
}

/// csv data Type
///
/// this is compatible with sqlite data types
#[derive(Clone,Copy,Debug)]
pub enum CSVType {
    Null, 
    Text, 
    Integer,
    Float, 
    BLOB,
}

impl CSVType {
    pub fn from_str(text: &str) -> Self {
        match text.to_lowercase().as_str() {
            "blob" => Self::BLOB,
            "float" => Self::Float,
            "integer" => Self::Integer,
            "text" => Self::Text,
            _ => Self::Null,
        }
    }
}

#[derive(Clone,Debug, PartialEq, PartialOrd)]
pub enum CSVVariant<'var> {
    Null, 
    Text(&'var str),
    Integer(i32),
    Float(f32),
    BLOB(&'var [u8]),
}

#[derive(Debug)]
pub struct Query {
    pub table_name: String,
    pub column_names: Option<Vec<String>>,
    predicates: Option<Vec<Predicate>>,
    
    // TODO
    // Currently join is not supported
    #[allow(dead_code)]
    joined_tables: Option<Vec<String>>,
}

impl Query {
    pub fn from_str(query: &str) -> Self {
        Parser::new().parse(query)
    }

    pub fn empty(table_name: &str) -> Self {
        Self {
            table_name: table_name.to_owned(), 
            column_names: None,
            predicates: None,
            joined_tables: None,
        }
    }

    pub fn build() -> Self {
        Self {
            table_name: String::new(), 
            column_names: None,
            predicates: None,
            joined_tables: None,
        }
    }

    pub fn columns(mut self, colum_names: Vec<String>) -> Self {
        self.column_names.replace(colum_names);
        self
    }

    pub fn predicate(mut self, predicate: Predicate) -> Self {
        if let None = self.predicates {
            self.predicates = Some(vec![]);
        }
        // This is safe to unwrap
        self.predicates.as_mut().unwrap().push(predicate);
        self
    }

    pub fn new(table_name: String, column_names: Option<Vec<String>>,predicates: Option<Vec<Predicate>>, joined_tables: Option<Vec<String>>) -> Self {
        Self {
            table_name,
            column_names,
            predicates,
            joined_tables,
        }
    }
}

#[derive(Debug)]
pub struct Predicate {
    separator: Separator,
    pub(crate) column: String,
    operation: Operator,
    arguments: Vec<String>,
}

impl Predicate {
    // <BUILDER>
    pub fn build() -> Self {
        Self {
            separator: Separator::And,
            column: String::new(),
            operation: Operator::Equal,
            arguments: vec![],
        }
    }

    pub fn separator(mut self, separator: Separator) -> Self {
        self.separator= separator;
        self
    }

    pub fn column(mut self, column: &str) -> Self {
        self.column = column.to_owned(); 
        self
    }

    pub fn operator(mut self, op: Operator) -> Self {
        self.operation = op;
        self
    }
    pub fn raw_args(mut self, args: &str) -> Self {
        self.arguments = args.split(' ').map(|v| v.to_owned()).collect::<Vec<String>>();
        self
    }

    pub fn args(mut self, args:Vec<String>) -> Self {
        self.arguments = args;
        self
    }

    // </BUILDER>
    
    pub fn new(column: & str, operation: Operator) -> Self {
        Self {
            separator : Separator::And,
            column: column.to_owned(),
            operation,
            arguments: vec![],
        }
    }

    pub fn set_separator(&mut self, separator: Separator) {
        self.separator = separator;
    }

    pub fn set_column(&mut self, column: &str) {
        self.column = column.to_owned(); 
    }

    pub fn set_operator(&mut self, op: Operator) {
        self.operation = op;
    }

    pub fn set_args(&mut self, args:Vec<String>) {
        self.arguments = args;
    }

    pub fn add_arg(&mut self, arg: &str) {
        self.arguments.push(arg.to_owned());
    }
}

#[derive(Debug)]
pub enum Operator {
    Bigger,
    BiggerOrEqual,
    Smaller,
    SmallerOrEqual,
    Equal,
    NotEqual,
    Between,
    In,
}

impl Operator {
    pub fn from_token(token: &str) -> Self {
        match token.to_lowercase().as_str() {
            ">"         => Self::Bigger,
            ">="        => Self::BiggerOrEqual,
            "<"         => Self::Smaller,
            "<="        => Self::SmallerOrEqual,
            "="         => Self::Equal,
            "!="        => Self::NotEqual,
            "between"   => Self::Between,
            "in"        => Self::In,
            _ => panic!("NOt implemented"),
        }
    }
}

#[derive(Debug)]
pub enum Separator {
    And,
    Or,
}
