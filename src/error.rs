#[derive(Debug)]
pub enum Error {
    EnvVarError(std::env::VarError),
    IoError(std::io::Error),
    FmtError(std::fmt::Error),
    ParseError(ical::parser::ParserError),
    IterError,
    IncorrectRrule,
    UnSupportedOs,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EnvVarError(_) => write!(f, "No Home Dir found"),
            Self::IoError(e) => write!(f, "{e}"),
            Self::FmtError(e) => write!(f, "{e}"),
            Self::ParseError(e) => write!(f, "{e}"),
            Self::IterError => write!(f, "No next value in iterator"),
            Self::IncorrectRrule => write!(f, "Unable to extract recurring rule from events"),
            Self::UnSupportedOs => write!(f, "Unsupported OS"),
        }
    }
}
impl From<std::env::VarError> for Error {
    fn from(err: std::env::VarError) -> Self {
        Self::EnvVarError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<std::fmt::Error> for Error {
    fn from(err: std::fmt::Error) -> Self {
        Self::FmtError(err)
    }
}

impl From<ical::parser::ParserError> for Error {
    fn from(err: ical::parser::ParserError) -> Self {
        Self::ParseError(err)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
