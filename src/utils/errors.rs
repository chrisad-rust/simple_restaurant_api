#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, apistos::ApiErrorComponent)]
#[openapi_error(status(code = 400), status(code = 409), status(code = 503))]
pub enum Error {
    InvalidArgument(String),
    Conflict(String),
    Database(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidArgument(message) => write!(f, "{}", message),
            Error::Conflict(message) => write!(f, "{}", message),
            Error::Database(message) => write!(f, "{}", message),
        }
    }
}

impl actix_web::ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Error::InvalidArgument(_) => actix_web::http::StatusCode::BAD_REQUEST,
            Error::Conflict(_) => actix_web::http::StatusCode::CONFLICT,
            Error::Database(_) => actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::Configuration(error) => Self::Database(format!("{}", error)),
            sqlx::Error::InvalidArgument(value) => Self::InvalidArgument(value),
            sqlx::Error::Database(error) => Self::Conflict(format!("{}", error)),
            sqlx::Error::Io(error) => Self::Database(format!("{}", error)),
            sqlx::Error::Tls(error) => Self::Database(format!("{}", error)),
            sqlx::Error::Protocol(value) => Self::Database(value),
            sqlx::Error::RowNotFound => Self::Database(format!("Row not found!")),
            sqlx::Error::TypeNotFound { type_name } => {
                Self::Database(format!("Type {} not found!", type_name))
            }
            sqlx::Error::ColumnIndexOutOfBounds { index, len } => {
                Self::Database(format!("Column index out of bounds {}..{}!", index, len))
            }
            sqlx::Error::ColumnNotFound(value) => {
                Self::Database(format!("Column {} not found!", value))
            }
            sqlx::Error::ColumnDecode { index, source } => {
                Self::Database(format!("Column {} could not decoded by {}!", index, source))
            }
            sqlx::Error::Encode(error) => Self::Database(format!("{}", error)),
            sqlx::Error::Decode(error) => Self::Database(format!("{}", error)),
            sqlx::Error::AnyDriverError(error) => Self::Database(format!("{}", error)),
            sqlx::Error::PoolTimedOut => Self::Database(format!("Pool timeout!")),
            sqlx::Error::PoolClosed => Self::Database(format!("Pool closed!")),
            sqlx::Error::WorkerCrashed => Self::Database(format!("Worker crashed!")),
            sqlx::Error::Migrate(migrate_error) => Self::Database(format!("{}", migrate_error)),
            sqlx::Error::InvalidSavePointStatement => {
                Self::Database(format!("Invalid save point statement."))
            }
            sqlx::Error::BeginFailed => Self::Database(format!("Begin Failed")),
            _ => Self::Database(format!("Unknown error.")),
        }
    }
}
