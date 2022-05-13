#[cfg(feature = "hustoj")]
type SqlxError = sqlx::Error;
#[cfg(not(feature = "hustoj"))]
type SqlxError = std::convert::Infallible;

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    TOMLParseError(toml::de::Error),
    ByteParseError(byte_unit::ByteError),
    BadLogLevel(String),
    BadPathEncoding(std::path::PathBuf),
    SystemdError(systemd_run::Error),
    UnconfiguredLanguage(String),
    BadSolutionID(String),
    SQLError(SqlxError),
    BadProblem(i32),
    NonUtf8Msg(std::string::FromUtf8Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IOError(e) => {
                write!(f, "input/output error: {}", e)
            }
            Self::TOMLParseError(e) => {
                write!(f, "error parsing TOML: {}", e)
            }
            Self::ByteParseError(e) => {
                write!(f, "error parsing byte value: {}", e)
            }
            Self::BadLogLevel(e) => {
                write!(f, "invalid log level {}", e)
            }
            Self::BadPathEncoding(p) => {
                write!(f, "non-UTF8 path {}", p.as_path().display())
            }
            Self::SystemdError(e) => {
                write!(f, "systemd error: {}", e)
            }
            Self::UnconfiguredLanguage(l) => {
                write!(f, "unconfigured language {}", &l)
            }
            Self::BadSolutionID(s) => {
                write!(f, "bad solution ID {}", &s)
            }
            Self::SQLError(e) => {
                write!(f, "sql error: {}", e)
            }
            Self::BadProblem(p) => {
                write!(f, "bad configuration for problem {}", p)
            }
            Self::NonUtf8Msg(_) => {
                write!(f, "message is not UTF-8")
            }
        }
    }
}

impl std::error::Error for Error {}
