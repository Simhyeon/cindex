use std::fmt::Display;
use rayon::prelude::*;

#[derive(Debug)]
pub struct CSVTable {
    pub(crate) name: String,
    pub(crate) headers: Vec<String>,
    pub(crate) header_types: Vec<CSVType>,
    pub(crate) rows: Vec<CSVRow>,
}

impl CSVTable {
    pub fn new(name: &str, headers: Vec<(String, CSVType)>, rows: Vec<Vec<&str>>) -> Self {
        // Make this rayon compatible iterator
        let rows = rows.iter().map(|row| {
            CSVRow::new(&headers, row)
        }).collect::<Vec<CSVRow>>();
        let (headers, header_types) = headers.iter().map(|(s,t)| (s.to_string(),t)).unzip();

        Self {
            name: name.to_owned(),
            headers,
            header_types,
            rows,
        }
    }

    pub fn query(&self, query: &Query) -> Vec<&CSVRow> {
        let queried : Vec<&CSVRow> = self.rows.par_iter().filter(|row| {
            row.filter(&self.headers,&query.predicates.as_ref().unwrap_or(&vec![]))
        }).collect();

        queried
    }
}

impl Display for CSVTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n",self.headers.join(","))?;
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
    pub fn new(headers: &Vec<(String, CSVType)>, row: &Vec<&str>) -> Self {
        let mut data : Vec<CSVData> = Vec::new();
        // Check index of headers and type check
        for (index, &item) in row.iter().enumerate() {
            data.push(CSVData::new(headers[index].1,item));
        }

        Self { data }
    }

    pub fn filter(&self, headers: &Vec<String>, predicates: &Vec<Predicate>) -> bool {
        let failed : Vec<_> = 
            predicates.par_iter().filter(|pre|
                {
                    let target = &self.data[headers.iter().position(|h| h == pre.column).unwrap()];
                    !target.operate(&pre.operation, &pre.arguments)
                }
            ).collect();

        // Failed is zero which means it has succeeded
        failed.len() == 0
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
    pub fn new(data_type: CSVType, value: &str) -> Self {
        Self {
            data_type,
            value: value.to_owned(),
        }
    }

    pub fn operate(&self, operation : &Operation, args : &Vec<String>) -> bool {

        if args.len() < 1 {
            eprintln!("ERROR!");
            panic!();
        }

        let (var, arg) = match self.data_type {
            CSVType::Null => (CSVVariant::Null,CSVVariant::Null),
            CSVType::Text => (CSVVariant::Text(&self.value), CSVVariant::Text(&args[0])),
            CSVType::Integer => (CSVVariant::Integer(self.value.parse().expect("F")), CSVVariant::Integer(args[0].parse().expect("F"))),
            CSVType::Float => (CSVVariant::Float(self.value.parse().expect("F")), CSVVariant::Float(args[0].parse().expect("F"))),
            CSVType::BLOB => (CSVVariant::BLOB(self.value.as_bytes()), CSVVariant::BLOB(&args[0].as_bytes())),
        };

        match operation {
            Operation::Bigger => {
                var > arg
            },
            Operation::BiggerOrEqual => {
                var >= arg
            },
            Operation::Smaller => {
                var < arg
            },
            Operation::SmallerOrEqual => { 
                var <= arg
            },
            Operation::Equal => { 
                var == arg
            },
            Operation::NotEqual => {
                var != arg
            },
            Operation::In => {
                // TODO
                //args.contains(&self.value)
                true
            },
            Operation::Between => {
                //self.value >= args[0] && self.value <= args[1]
                true
            },
        }
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

pub struct Query<'query> {
    predicates: Option<Vec<Predicate<'query>>>,
    joined_tables: Option<Vec<&'query str>>,
}

impl<'query> Query<'query> {
    pub fn new(predicates: Option<Vec<Predicate<'query>>>, joined_tables: Option<Vec<&'query str>>) -> Self {
        Self {
            predicates,
            joined_tables,
        }
    }
}

pub struct Predicate<'pre> {
    column: &'pre str,
    operation: Operation,
    arguments: Vec<String>,
}

impl<'pre> Predicate<'pre> {
    pub fn new(column: &'pre str, operation: Operation) -> Self {
        Self {
            column,
            operation,
            arguments: vec![],
        }
    }

    pub fn raw_args(mut self, args: &str) -> Self {
        self.arguments = args.split(' ').map(|v| v.to_owned()).collect::<Vec<String>>();
        self
    }

    pub fn args(mut self, args:Vec<String>) -> Self {
        self.arguments = args;
        self
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


