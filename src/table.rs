use crate::error::{CIndexError, CIndexResult};
use crate::models::OrderType;
use crate::query::Query;
use crate::ReaderOption;
use crate::{Operator, Predicate};
use dcsv::{Reader, Row, VirtualData};
#[cfg(feature = "rayon")]
use rayon::prelude::*;
use std::collections::HashSet;
use std::fmt::Display;
use std::io::BufRead;
use std::iter::FromIterator;

pub(crate) struct Table {
    pub(crate) header: HashSet<String>,
    pub(crate) data: VirtualData,
}

impl Table {
    pub fn build(table_content: impl BufRead, reader_option: ReaderOption) -> CIndexResult<Self> {
        let data = Reader::new()
            .with_option(reader_option)
            .data_from_stream(table_content)
            .map_err(|err| {
                CIndexError::InvalidTableInput(format!(
                    "Failed to read table contents with error : {}",
                    err
                ))
            })?;

        Ok(Self {
            header: HashSet::from_iter(data.columns.iter().map(|c| c.name.clone())),
            data,
        })
    }

    pub fn new(table_content: impl BufRead) -> CIndexResult<Self> {
        let data = Reader::new()
            .consume_dquote(true)
            .ignore_empty_row(true)
            .has_header(true)
            .data_from_stream(table_content)
            .map_err(|err| {
                CIndexError::InvalidTableInput(format!(
                    "Failed to read table contents with error : {}",
                    err
                ))
            })?;

        Ok(Self {
            header: HashSet::from_iter(data.columns.iter().map(|c| c.name.clone())),
            data,
        })
    }

    pub fn new_with_headers(table_content: impl BufRead, headers: &[String]) -> CIndexResult<Self> {
        let data = Reader::new()
            .ignore_empty_row(true)
            .consume_dquote(true)
            .custom_header(headers)
            .data_from_stream(table_content)
            .map_err(|err| {
                CIndexError::InvalidTableInput(format!(
                    "Failed to read table contents with error : {}",
                    err
                ))
            })?;

        Ok(Self {
            header: HashSet::from_iter(data.columns.iter().map(|c| c.name.clone())),
            data,
        })
    }

    pub(crate) fn query(&self, query: &Query) -> CIndexResult<Vec<&Row>> {
        let boilerplate = vec![];
        let predicates = if let Some(pre) = query.predicates.as_ref() {
            for item in pre {
                if !self.header.contains(&item.column) {
                    return Err(CIndexError::InvalidColumn(format!(
                        "Failed to get column \"{}\" from header",
                        item.column
                    )));
                }
            }
            pre
        } else {
            &boilerplate // Empty predicates
        };

        // TODO
        // Can it be improved?
        #[cfg(feature = "rayon")]
        let iter = self.data.rows.par_iter();
        #[cfg(not(feature = "rayon"))]
        let iter = self.data.rows.iter();

        let mut queried: Vec<&Row> = iter.filter(|row| self.filter(row, predicates)).collect();

        // This is already a queried records, which means columns already persists for every
        // records, therefore it is safe to get value and unwrap
        match &query.order_type {
            OrderType::Desc(col) | OrderType::Asec(col) => {
                if self.header.contains(col) {
                    queried.sort_by(|&a, &b| {
                        let a = a.get_cell_value(col).unwrap();
                        let b = b.get_cell_value(col).unwrap();
                        if let OrderType::Desc(_) = &query.order_type {
                            // Descending
                            b.partial_cmp(a).unwrap()
                        } else {
                            // Aescending
                            a.partial_cmp(b).unwrap()
                        }
                    });
                } else {
                    return Err(CIndexError::InvalidQueryStatement(format!(
                        "Column \"{}\" doesn't exist",
                        col
                    )));
                }
            }
            _ => (),
        }

        // If offset or limit has been provided
        // Slice it
        if query.range.0 != 0 || query.range.1 != 0 {
            let offset = (query.range.0).min(queried.len());
            let limit = query.range.1;

            let query_limit = if limit == 0 {
                queried.len()
            } else {
                (offset + limit).min(queried.len())
            };
            queried = queried[offset..query_limit].to_vec();
        }

        Ok(queried)
    }

    /// Iterator method
    // This should not return result because it is usesd in filter_adopter
    fn filter(&self, row: &Row, predicates: &Vec<Predicate>) -> bool {
        for pre in predicates {
            let column = pre.column.as_str();
            if !operate_value(
                &row.get_cell_value(column).unwrap().to_string(),
                &pre.arguments,
                pre,
            ) {
                return false;
            }
        }

        true
    }
}

fn operate_value(comparator: &str, values: &[String], pre: &Predicate) -> bool {
    let var = comparator;
    let arg = values[0].as_str();
    let operation = &pre.operation;
    match operation {
        // It is safe to unwrap, because Like panics when matcher doesn't compiled
        Operator::Like => pre.matcher.as_ref().unwrap().is_match(var),
        Operator::Bigger => var > arg,
        Operator::BiggerOrEqual => var >= arg,
        Operator::Smaller => var < arg,
        Operator::SmallerOrEqual => var <= arg,
        Operator::Equal => var == arg,
        Operator::NotEqual => var != arg,
        Operator::In => values.contains(&var.to_owned()),
        Operator::Between => var >= &values[0] && var <= &values[1],
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}
