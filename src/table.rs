use indexmap::IndexSet;
use crate::error::{CIndexResult, CIndexError};
use crate::query::Query;
use crate::models::{Row, CsvType, OrderType, Variant};
use std::fmt::Display;
#[cfg(feature = "rayon")]
use rayon::prelude::*;

#[derive(Debug)]
pub struct Table {
    pub(crate) headers: IndexSet<String>,
    pub(crate) rows: Vec<Row>,
}

impl Table {
    pub fn new(headers: Vec<(String, CsvType)>, rows: Vec<Vec<&str>>) -> CIndexResult<Self> {
        // Make this rayon compatible iterator
        let rows: CIndexResult<Vec<Row>> = rows.iter().map(|row| Row::new(&headers, row)).collect();

        let mut header_set: IndexSet<String> = IndexSet::new();

        for (header, _) in headers.iter() {
            header_set.insert(header.to_owned());
        }

        Ok(Self {
            headers: header_set,
            rows: rows?,
        })
    }

    pub(crate) fn query(&self, query: &Query) -> CIndexResult<Vec<&Row>> {
        let boilerplate = vec![];
        let predicates = if let Some(pre) = query.predicates.as_ref() {
            for item in pre {
                if !self.headers.contains(&item.column) {
                    return Err(CIndexError::InvalidColumn(format!(
                        "Failed to get column \"{}\" from header",
                        item.column
                    )));
                }
            }
            pre
        } else {
            &boilerplate
        };

        // TODO
        // Can it be improved?
        #[cfg(feature = "rayon")]
        let iter = self.rows.par_iter();
        #[cfg(not(feature = "rayon"))]
        let iter = self.rows.iter();

        let mut queried: Vec<_> = iter
            .filter_map(|row| { match row.filter(&self.headers, &predicates) {
                    Ok(boolean) => {
                        if boolean {
                            Some(row)
                        } else {
                            None
                        }
                    }
                    Err(err) => {
                        eprintln!("{}", err);
                        None
                    }
                }
            })
            .collect();

        match &query.order_type {
            OrderType::None => (),
            OrderType::Asec(col) => {
                if let Some(index) = self.headers.get_index_of(col.as_str()) {
                    queried.sort_by(|a, b| {
                        let a = Variant::from_data(&a.data[index]).unwrap();
                        let b = Variant::from_data(&b.data[index]).unwrap();
                        a.partial_cmp(&b).unwrap()
                    });
                } else {
                    return Err(CIndexError::InvalidQueryStatement(format!(
                        "Column \"{}\" doesn't exist",
                        col
                    )));
                }
            }
            OrderType::Desc(col) => {
                if let Some(index) = self.headers.get_index_of(col.as_str()) {
                    queried.sort_by(|a, b| {
                        let a = Variant::from_data(&a.data[index]).unwrap();
                        let b = Variant::from_data(&b.data[index]).unwrap();
                        b.partial_cmp(&a).unwrap()
                    });
                } else {
                    return Err(CIndexError::InvalidQueryStatement(format!(
                        "Column \"{}\" doesn't exist",
                        col
                    )));
                }
            }
        }

        // If offset or limit has been provided
        // Slice it
        if query.range.0 != 0 || query.range.1 != 0 {
            let offset = (query.range.0).min(queried.len());
            let limit = query.range.1;

            let query_limit = if limit == 0 { queried.len() } else { (offset + limit).min(queried.len()) };
            queried = queried[offset..query_limit].to_vec();
        }

        Ok(queried)
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\n",
            self.headers
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>()
                .join(",")
        )?;

        for row in &self.rows {
            write!(f, "{}\n", row)?;
        }
        write!(f, "")
    }
}

