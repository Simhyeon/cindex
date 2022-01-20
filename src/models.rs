use std::fmt::Display;
use crate::{parser::Parser, CIndexResult};
#[cfg(feature = "rayon")]
use rayon::prelude::*;
use regex::Regex;
use crate::error::CIndexError;
use indexmap::IndexSet;

#[derive(Debug)]
pub struct CsvTable {
    pub(crate) headers: IndexSet<String>,
    pub(crate) rows: Vec<CsvRow>,
}

impl CsvTable {
    pub fn new(headers: Vec<(String, CsvType)>, rows: Vec<Vec<&str>>) -> CIndexResult<Self> {
        // Make this rayon compatible iterator
        let rows : CIndexResult<Vec<CsvRow>> = rows.iter().map(|row| {
            CsvRow::new(&headers, row)
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

    pub fn query(&self, query: &Query) -> CIndexResult<Vec<&CsvRow>> {
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
        #[cfg(feature = "rayon")]
        let iter = self.rows.par_iter();
        #[cfg(not(feature = "rayon"))]
        let iter = self.rows.iter();

        let mut queried : Vec<_> = iter.filter_map(|row| {
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

        match &query.order_type {
            OrderType::None => (),
            OrderType::Asec(col) => {
                if let Some(index) = self.headers.get_index_of(col.as_str()) {
                    queried.sort_by(|a,b| {
                        let a = CsvVariant::from_data(&a.data[index]).unwrap();
                        let b = CsvVariant::from_data(&b.data[index]).unwrap();
                        a.partial_cmp(&b).unwrap()
                    });
                } else {
                    return Err(CIndexError::InvalidQueryStatement(format!("Column \"{}\" doesn't exist", col)))
                }
            },
            OrderType::Desc(col) => {
                if let Some(index) = self.headers.get_index_of(col.as_str()) {
                    queried.sort_by(|a,b| {
                        let a = CsvVariant::from_data(&a.data[index]).unwrap();
                        let b = CsvVariant::from_data(&b.data[index]).unwrap();
                        b.partial_cmp(&a).unwrap()
                    });
                } else {
                    return Err(CIndexError::InvalidQueryStatement(format!("Column \"{}\" doesn't exist", col)))
                }
            },
        } 

        Ok(queried)
    }
}

impl Display for CsvTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n",self.headers.iter().map(|s| s.as_str()).collect::<Vec<&str>>().join(","))?;

        for row in &self.rows {
            write!(f, "{}\n",row)?;
        }
        write!(f,"")
    }
}

#[derive(Debug)]
pub struct CsvRow {
    data: Vec<CsvData>,
}

impl Display for CsvRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data.iter().map(|datum| datum.value.as_str()).collect::<Vec<&str>>().join(","))
    }
} 

impl CsvRow {
    pub fn new(headers: &Vec<(String, CsvType)>, row: &Vec<&str>) -> CIndexResult<Self> {
        let mut data : Vec<CsvData> = Vec::new();
        // Check index of headers and type check
        for (index, &item) in row.iter().enumerate() {
            data.push(CsvData::new(headers[index].1,item)?);
        }

        Ok(Self { data })
    }

    pub fn filter(&self, headers: &IndexSet<String>, predicates: &Vec<Predicate>) -> CIndexResult<bool> {
        #[cfg(feature = "rayon")]
        let iter = predicates.par_iter();
        #[cfg(not(feature = "rayon"))]
        let iter = predicates.iter();

        let failed : Result<Vec<_>, CIndexError> = 
            iter.map(|pre|
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
pub struct CsvData {
    data_type : CsvType,
    value: String,
}

impl Display for CsvData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}", self.value)
    }
}

impl CsvData {
    pub fn new(data_type: CsvType, value: &str) -> CIndexResult<Self> {
        let data = Self {
            data_type,
            value: value.to_owned(),
        };
        data.check_type()?;

        Ok(data)
    }

    pub fn check_type(&self) -> CIndexResult<()> {
        match self.data_type {
            CsvType::Null    => {
                if !self.value.is_empty() {
                    return Err(CIndexError::InvalidDataType(format!("Value \"{}\" is not NULL", self.value)));
                }
            },
            CsvType::Float   => {
                self.value.parse::<f32>().map_err(|_| CIndexError::InvalidDataType(format!("Value \"{}\" is not a floating point number", self.value)))?;
            },
            CsvType::Integer => {
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

        let var = CsvVariant::from_data(&self)?;
        let arg_data = CsvData::new(self.data_type, &args[0])?;
        let arg = CsvVariant::from_data(&arg_data)?;

        let result = match operation {
            Operator::Like => {
                let arg = arg.as_string();
                let matcher = Regex::new(&arg).map_err(|_|CIndexError::InvalidQueryStatement(format!("Could not make a regex statemtn from given value: \"{}\"", arg)))?;
                matcher.is_match(&var.as_string())
            }
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
                args.contains(&self.value)
            },
            Operator::Between => {
                self.value >= args[0] && self.value <= args[1]
            },
        };

        Ok(result)
    }
}

/// CSV data Type
#[derive(Clone,Copy,Debug)]
pub enum CsvType {
    Null, 
    Text, 
    Integer,
    Float, 
    BLOB,
}

impl CsvType {
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

/// Wrapper around csvtyped data 
///
/// This enables various comparsion operation as single enum value.
#[derive(Clone,Debug, PartialEq, PartialOrd)]
pub enum CsvVariant<'var> {
    Null, 
    Text(&'var str),
    Integer(i32),
    Float(f32),
    BLOB(&'var [u8]),
}

impl<'var> CsvVariant<'var> {
    fn as_string(&self) -> String {
        match self {
            Self::Null => String::new(),
            Self::Text(text) => text.to_string(),
            Self::BLOB(bytes) => String::from_utf8_lossy(&bytes.to_vec()).to_string(),
            Self::Float(num) => num.to_string(),
            Self::Integer(num) => num.to_string(),
        }
    }

    fn from_data(data: &'var CsvData) -> CIndexResult<Self> {
        let variant = match data.data_type {
            CsvType::Null => CsvVariant::Null,
            CsvType::Text => CsvVariant::Text(&data.value),
            CsvType::Integer => {
                CsvVariant::Integer(data.value.parse().map_err(|_| CIndexError::TypeDiscord(format!("{} as integer", data.value)))?)
            },
            CsvType::Float => {
                CsvVariant::Float(data.value.parse().map_err(|_| CIndexError::TypeDiscord(format!("{} as float", data.value)))?)
            },
            CsvType::BLOB => CsvVariant::BLOB(data.value.as_bytes()),
        };

        Ok(variant)
    }
}

#[derive(Debug)]
pub enum OrderType {
    None,
    Asec(String),
    Desc(String),
}

impl OrderType {
    pub fn from_str(text: &str, column: &str) -> CIndexResult<Self> {
        match text.to_lowercase().as_str() {
            "asec" => Ok(Self::Asec(column.to_string())),
            "desc" => Ok(Self::Desc(column.to_string())),
            _ => return Err(CIndexError::InvalidQueryStatement(format!("Ordertype can only be ASEC OR DESC but given \"{}\"", text)))
        }
    }
} 

/// Query to index a table
#[derive(Debug)]
pub struct Query {
    pub table_name: String,
    pub column_names: Option<Vec<String>>,
    predicates: Option<Vec<Predicate>>,
    order_type: OrderType,
    
    // TODO
    // Currently join is not supported
    #[allow(dead_code)]
    joined_tables: Option<Vec<String>>,
}

impl Query {
    pub fn from_str(query: &str) -> CIndexResult<Self> {
        Parser::new().parse(query)
    }

    pub fn empty(table_name: &str) -> Self {
        Self {
            table_name: table_name.to_owned(), 
            column_names: None,
            predicates: None,
            order_type: OrderType::None,
            joined_tables: None,
        }
    }

    pub fn build() -> Self {
        Self {
            table_name: String::new(), 
            column_names: None,
            predicates: None,
            joined_tables: None,
            order_type: OrderType::None,
        }
    }

    pub fn columns(mut self, colum_names: Vec<impl AsRef<str>>) -> Self {
        self.column_names.replace(colum_names.iter().map(|s| s.as_ref().to_owned()).collect());
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

    pub fn new(table_name: String, column_names: Option<Vec<String>>,predicates: Option<Vec<Predicate>>, joined_tables: Option<Vec<String>>, order_type: OrderType) -> Self {
        Self {
            table_name,
            column_names,
            predicates,
            joined_tables,
            order_type,
        }
    }
}


/// Predicate to decide whether a specific row should be included
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

    pub fn args(mut self, args:Vec<impl AsRef<str>>) -> Self {
        self.arguments = args.iter().map(|s| s.as_ref().to_owned()).collect();
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

/// Operator to calculate operands
#[derive(Debug)]
pub enum Operator {
    Bigger,
    BiggerOrEqual,
    Smaller,
    SmallerOrEqual,
    Equal,
    NotEqual,
    Like,
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
            "like"      => Self::Like,
            _ => panic!("NOt implemented"),
        }
    }
}

#[derive(Debug)]
pub enum Separator {
    And,
    Or,
}
