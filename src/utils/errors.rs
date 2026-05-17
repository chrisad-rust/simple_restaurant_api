#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum Error {
    InvalidArgument(String),
    Conflict(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidArgument(message) => write!(f, "{}", message),
            Error::Conflict(message) => write!(f, "{}", message),
        }
    }
}
