use std::fmt::Display;

use crate::error::CIndexError;
use crate::CIndexResult;

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
            _ => Err(CIndexError::InvalidQueryStatement(format!(
                "Ordertype can only be ASEC OR DESC but given \"{}\"",
                text
            ))),
        }
    }
}

pub enum ColumnVariant<'a> {
    Real(&'a str),
    Supplement(String),
}

impl<'a> Display for ColumnVariant<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dis = match self {
            Self::Real(col) => col.to_string(),
            Self::Supplement(col) => col.to_string(),
        };
        write!(f, "{}", dis)
    }
}
