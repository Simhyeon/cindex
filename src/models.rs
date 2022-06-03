use crate::error::CIndexError;
use crate::CIndexResult;
use indexmap::IndexSet;
#[cfg(feature = "rayon")]
use rayon::prelude::*;
use regex::Regex;
use std::fmt::Display;
use crate::query::{Operator, Predicate};

#[derive(Debug)]
pub(crate) struct Row {
    pub(crate) data: Vec<Data>,
}

impl Display for Row {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.data
                .iter()
                .map(|datum| datum.value.as_str())
                .collect::<Vec<&str>>()
                .join(",")
        )
    }
}

impl Row {
    pub fn new(headers: &Vec<(String, CsvType)>, row: &Vec<&str>) -> CIndexResult<Self> {
        let mut data: Vec<Data> = Vec::new();
        // Check index of headers and type check
        for (index, &item) in row.iter().enumerate() {
            data.push(Data::new(headers[index].1, item)?);
        }

        Ok(Self { data })
    }

    pub fn filter(
        &self,
        headers: &IndexSet<String>,
        predicates: &Vec<Predicate>,
    ) -> CIndexResult<bool> {
        #[cfg(feature = "rayon")]
        let iter = predicates.par_iter();
        #[cfg(not(feature = "rayon"))]
        let iter = predicates.iter();

        let failed: Result<Vec<_>, CIndexError> = iter
            .map(|pre| {
                // This is safe to unwrap because table's query method alwasy check if column
                // exists before filtering
                let target = &self.data[headers.get_index_of(&pre.column).unwrap()];
                target.operate(&pre.operation, &pre.arguments)
            })
            .collect();

        let failed = failed?;
        let failed: Vec<_> = failed.iter().filter(|s| *s == &false).collect();

        // Failed is zero which means it has succeeded
        Ok(failed.len() == 0)
    }

    pub fn splited(&self) -> Vec<&str> {
        self.data.iter().map(|d| d.value.as_str()).collect()
    }

    pub fn column_splited(&self, columns: &Vec<ColumnIndex>) -> Vec<&str> {
        let mut col_iter = columns.iter();
        let mut formatted = vec![];

        // First item
        if let Some(col) = col_iter.next() {
            if let ColumnIndex::Real(index) = col {
                formatted.push(self.data[*index].value.as_str());
            } else {
                formatted.push("");
            }
        }

        while let Some(col) = col_iter.next() {
            if let ColumnIndex::Real(index) = col {
                formatted.push(self.data[*index].value.as_str());
            } else {
                formatted.push("");
            }
        }
        formatted
    }
}

#[derive(Debug)]
pub(crate) struct Data {
    data_type: CsvType,
    value: String,
}

impl Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Data {
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
            CsvType::Null => {
                if !self.value.is_empty() {
                    return Err(CIndexError::InvalidDataType(format!(
                        "Value \"{}\" is not NULL",
                        self.value
                    )));
                }
            }
            CsvType::Float => {
                self.value.parse::<f32>().map_err(|_| {
                    CIndexError::InvalidDataType(format!(
                        "Value \"{}\" is not a floating point number",
                        self.value
                    ))
                })?;
            }
            CsvType::Integer => {
                self.value.parse::<i32>().map_err(|_| {
                    CIndexError::InvalidDataType(format!(
                        "Value \"{}\" is not an integer",
                        self.value
                    ))
                })?;
            }
            _ => (),
        }
        Ok(())
    }

    pub fn operate(&self, operation: &Operator, args: &Vec<String>) -> CIndexResult<bool> {
        if args.len() < 1 {
            eprintln!("ERROR!");
            panic!();
        }

        let var = Variant::from_data(&self)?;
        let arg_data = Data::new(self.data_type, &args[0])?;
        let arg = Variant::from_data(&arg_data)?;

        let result = match operation {
            Operator::Like => {
                let arg = arg.as_string();
                let matcher = Regex::new(&arg).map_err(|_| {
                    CIndexError::InvalidQueryStatement(format!(
                        "Could not make a regex statemtn from given value: \"{}\"",
                        arg
                    ))
                })?;
                matcher.is_match(&var.as_string())
            }
            Operator::Bigger => var > arg,
            Operator::BiggerOrEqual => var >= arg,
            Operator::Smaller => var < arg,
            Operator::SmallerOrEqual => var <= arg,
            Operator::Equal => var == arg,
            Operator::NotEqual => var != arg,
            Operator::In => args.contains(&self.value),
            Operator::Between => self.value >= args[0] && self.value <= args[1],
        };

        Ok(result)
    }
}

/// CSV data Type
#[derive(Clone, Copy, Debug)]
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
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub(crate) enum Variant<'var> {
    Null,
    Text(&'var str),
    Integer(i32),
    Float(f32),
    BLOB(&'var [u8]),
}

impl<'var> Variant<'var> {
    fn as_string(&self) -> String {
        match self {
            Self::Null => String::new(),
            Self::Text(text) => text.to_string(),
            Self::BLOB(bytes) => String::from_utf8_lossy(&bytes.to_vec()).to_string(),
            Self::Float(num) => num.to_string(),
            Self::Integer(num) => num.to_string(),
        }
    }

    pub fn from_data(data: &'var Data) -> CIndexResult<Self> {
        let variant = match data.data_type {
            CsvType::Null => Variant::Null,
            CsvType::Text => Variant::Text(&data.value),
            CsvType::Integer => Variant::Integer(
                data.value
                    .parse()
                    .map_err(|_| CIndexError::TypeDiscord(format!("{} as integer", data.value)))?,
            ),
            CsvType::Float => Variant::Float(
                data.value
                    .parse()
                    .map_err(|_| CIndexError::TypeDiscord(format!("{} as float", data.value)))?,
            ),
            CsvType::BLOB => Variant::BLOB(data.value.as_bytes()),
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
            _ => {
                return Err(CIndexError::InvalidQueryStatement(format!(
                    "Ordertype can only be ASEC OR DESC but given \"{}\"",
                    text
                )))
            }
        }
    }
}

pub enum ColumnIndex {
    Real(usize),
    Supplement,
}

