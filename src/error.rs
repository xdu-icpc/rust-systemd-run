use std::fmt::Display;
use zbus::zvariant::OwnedValue;

/// The error type of `systemd_run`.
///
/// The various errors that can be returned by this crate.
#[derive(Debug)]
pub enum Error {
    /// An error connecting to D-Bus.
    DBusConnectionFail(zbus::Error),
    /// Invalid D-Bus path.
    DBusInvalidPath(zbus::zvariant::Error),
    /// An error subscribing to D-Bus `PropertiesChanged` signal.
    ListenPropertyChangeFail(zbus::Error),
    /// An error parsing the changed properties from the D-Bus
    /// `PropertiesChanged` signal.
    ParsePropertyChangeFail(zbus::Error),
    /// An error quering one property of a unit.
    QueryPropertyFail(zbus::fdo::Error),
    /// An error calling systemd to start the transient unit.
    StartFail(zbus::Error),
    /// An error attempting to calculate the time usage of a service.
    TimeUsageFail(&'static str, Box<OwnedValue>, Box<OwnedValue>),
}

/// Alias for a `Result` with the error type `systemd_run::Error`.
pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Self::DBusConnectionFail(e) => {
                write!(f, "cannot connect to dbus: {}", e)
            }
            Self::DBusInvalidPath(e) => {
                write!(f, "cannot get a dbus path: {}", e)
            }
            Self::StartFail(e) => {
                write!(f, "cannot start transient service: {}", e)
            }
            Self::ListenPropertyChangeFail(e) => {
                write!(f, "cannot listen property change events: {}", e)
            }
            Self::ParsePropertyChangeFail(e) => {
                write!(f, "cannot parse property changes: {}", e)
            }
            Self::QueryPropertyFail(e) => {
                write!(f, "cannot parse property changes: {}", e)
            }
            Self::TimeUsageFail(what, t0, t1) => {
                write!(f, "cannot calculate {} time usage: ", what)?;
                write!(f, "t0 = {:?}, t1 = {:?}", t0, t1)
            }
        }
    }
}

impl std::error::Error for Error {}
