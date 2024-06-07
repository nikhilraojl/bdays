use std::{env::consts::OS, path::PathBuf};

#[derive(Debug)]
pub enum Error {
    Io(std::env::VarError),
    UnSupportedOs,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(_) => write!(f, "No Home Dir found"),
            Self::UnSupportedOs => write!(f, "Unsupported OS"),
        }
    }
}
impl From<std::env::VarError> for Error {
    fn from(err: std::env::VarError) -> Self {
        Self::Io(err)
    }
}

impl std::error::Error for Error {}

type Result<T> = std::result::Result<T, Error>;

pub fn get() -> Result<PathBuf> {
    match OS {
        "windows" => Ok(PathBuf::from(std::env::var("userprofile")?)),
        "linux" => Ok(PathBuf::from(std::env::var("HOME")?)),
        _ => Err(Error::UnSupportedOs),
    }
}
