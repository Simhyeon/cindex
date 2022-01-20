use thiserror::Error;

pub type CIndexResult<T> = Result<T, CIndexError>;

#[derive(Debug, Error)]
pub enum CIndexError {
    #[error("IO Error\n= {0}")]
    IoError(std::io::Error),
    #[error("Invalid table name\n= {0}")]
    InvalidTableName(String),
    #[error("Failed to convert data into designted type\n= {0}")]
    TypeDiscord(String),
    #[error("Invalid column\n= {0}")]
    InvalidColumn(String),
    #[error("Invalid data type \n= {0}")]
    InvalidDataType(String),
    #[error("Invalid query statement \n= {0}")]
    InvalidQueryStatement(String),
}

impl From<std::io::Error> for CIndexError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}
