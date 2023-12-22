use crate::error::{CIndexError, CIndexResult};
use crate::models::OrderType;
use crate::parser::Parser;
use regex::Regex;
use std::collections::HashSet;

/// Query to index a table
#[derive(Debug)]
pub struct Query {
    pub table_name: String,
    pub column_names: Vec<String>,
    pub column_map: Option<Vec<String>>,
    pub(crate) predicates: Option<Vec<Predicate>>,
    pub(crate) order_type: OrderType,
    pub flags: QueryFlags,
    pub range: (usize, usize),

    // TODO
    // Currently join is not supported
    #[allow(dead_code)]
    joined_tables: Option<Vec<String>>,
}

impl std::str::FromStr for Query {
    type Err = CIndexError;

    /// This creates parser inside.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Parser::new().parse(s)
    }
}

impl Query {
    /// Build an empty query
    pub fn build() -> Self {
        Self {
            table_name: String::new(),
            column_names: vec![],
            predicates: None,
            joined_tables: None,
            order_type: OrderType::None,
            column_map: None,
            flags: QueryFlags::new(),
            range: (0, 0),
        }
    }

    /// Set table name
    pub fn table(mut self, name: &str) -> Self {
        self.table_name = name.to_string();
        self
    }

    /// Append target columns as builder pattern
    pub fn columns(mut self, colum_names: Vec<impl AsRef<str>>) -> Self {
        self.column_names = colum_names.iter().map(|s| s.as_ref().to_owned()).collect();
        self
    }

    /// Append predicate as builder pattern
    pub fn predicate(mut self, predicate: Predicate) -> Self {
        if self.predicates.is_none() {
            self.predicates = Some(vec![]);
        }
        // This is safe to unwrap
        self.predicates.as_mut().unwrap().push(predicate);
        self
    }

    // This is ok to have too many arguments because it is inner usage only
    /// Create a query with every information
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        table_name: String,
        column_names: Vec<String>,
        predicates: Option<Vec<Predicate>>,
        joined_tables: Option<Vec<String>>,
        order_type: OrderType,
        column_map: Option<Vec<String>>,
        flags: QueryFlags,
        range: (usize, usize),
    ) -> Self {
        Self {
            table_name,
            column_names,
            predicates,
            joined_tables,
            order_type,
            column_map,
            flags,
            range,
        }
    }
}

/// Predicate to decide whether a specific row qualifies a query or not
#[derive(Debug)]
pub struct Predicate {
    pub(crate) separator: Separator,
    pub(crate) column: String,
    pub(crate) operation: Operator,
    pub(crate) arguments: Vec<String>,
    pub(crate) matcher: Option<Regex>,
}

impl Predicate {
    // <BUILDER>
    /// Create empty predicate
    pub fn build() -> Self {
        Self {
            separator: Separator::And,
            column: String::new(),
            operation: Operator::Equal,
            arguments: vec![],
            matcher: None,
        }
    }

    pub fn separator(mut self, separator: Separator) -> Self {
        self.separator = separator;
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

    /// Create regex matcher from regex pattern
    pub fn matcher(mut self, pattern: &str) -> CIndexResult<Self> {
        self.matcher.replace(Regex::new(pattern).map_err(|_| {
            CIndexError::InvalidQueryStatement(format!("Invalid regex pattern : \"{}\"", pattern))
        })?);
        Ok(self)
    }

    /// Append raw args ( String )
    pub fn raw_args(mut self, args: &str) -> Self {
        self.arguments = args
            .split(' ')
            .map(|v| v.to_owned())
            .collect::<Vec<String>>();
        self
    }

    pub fn args(mut self, args: Vec<impl AsRef<str>>) -> Self {
        self.arguments = args.iter().map(|s| s.as_ref().to_owned()).collect();
        self
    }

    // </BUILDER>

    pub fn new(column: &str, operation: Operator) -> Self {
        Self {
            separator: Separator::And,
            column: column.to_owned(),
            operation,
            arguments: vec![],
            matcher: None,
        }
    }

    pub fn set_matcher(&mut self, pattern: &str) -> CIndexResult<()> {
        self.matcher.replace(Regex::new(pattern).map_err(|_| {
            CIndexError::InvalidQueryStatement(format!("Invalid regex pattern : \"{}\"", pattern))
        })?);
        Ok(())
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

    pub fn set_args(&mut self, args: Vec<String>) {
        self.arguments = args;
    }

    pub fn add_arg(&mut self, arg: &str) {
        self.arguments.push(arg.to_owned());
    }
}

/// Operator to calculate operands
#[derive(Debug, PartialEq, Eq)]
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
    pub fn from_token(token: &str) -> CIndexResult<Self> {
        let op = match token.to_lowercase().as_str() {
            ">" => Self::Bigger,
            ">=" => Self::BiggerOrEqual,
            "<" => Self::Smaller,
            "<=" => Self::SmallerOrEqual,
            "=" => Self::Equal,
            "!=" => Self::NotEqual,
            "between" => Self::Between,
            "in" => Self::In,
            "like" => Self::Like,
            _ => {
                return Err(CIndexError::InvalidQueryStatement(format!(
                    "Unsupported operator \"{}\"",
                    &token
                )))
            }
        };
        Ok(op)
    }
}

#[derive(Debug)]
pub enum Separator {
    And,
    Or,
}

#[derive(Debug)]
pub struct QueryFlags {
    flags: HashSet<QueryFlagType>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum QueryFlagType {
    Phd, // Print header
    Sup, // Supplement
    TP,  // Tranpose
}

impl Default for QueryFlags {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryFlags {
    pub fn new() -> Self {
        Self {
            flags: HashSet::new(),
        }
    }
    /// Clear all bit flags
    pub fn clear(&mut self) {
        self.flags.clear();
    }

    pub fn contains(&self, flag: QueryFlagType) -> bool {
        self.flags.contains(&flag)
    }

    pub fn set(&mut self, flag: &str) -> CIndexResult<()> {
        match flag.to_lowercase().as_str() {
            "phd" | "print-header" => self.flags.insert(QueryFlagType::Phd),
            "sup" | "supplment" => self.flags.insert(QueryFlagType::Sup),
            "tp" | "tranpose" => self.flags.insert(QueryFlagType::TP),
            _ => {
                return Err(CIndexError::InvalidQueryStatement(
                    "Invalid query flag".to_string(),
                ));
            }
        };
        Ok(())
    }
}
