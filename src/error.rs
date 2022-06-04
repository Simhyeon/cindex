pub type CIndexResult<T> = Result<T, CIndexError>;

#[derive(Debug)]
pub enum CIndexError {
    InvalidTableInput(String),
    IoError(std::io::Error),
    InvalidTableName(String),
    TypeDiscord(String),
    InvalidColumn(String),
    InvalidDataType(String),
    InvalidQueryStatement(String),
}

impl std::fmt::Display for CIndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidTableInput(txt) => write!(f, "Invalid csv table\n= {}", txt),
            Self::IoError(err) => write!(f, "IO Error\n= {}", err),
            Self::InvalidTableName(err) => write!(f, "Invalid table name\n= {}", err),
            Self::TypeDiscord(err) => {
                write!(f, "Failed to convert data into designted type\n= {}", err)
            }
            Self::InvalidColumn(err) => write!(f, "Invalid column\n= {}", err),
            Self::InvalidDataType(err) => write!(f, "Invalid data type \n= {}", err),
            Self::InvalidQueryStatement(err) => write!(f, "Invalid query statement \n= {}", err),
        }
    }
}

impl From<std::io::Error> for CIndexError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}
