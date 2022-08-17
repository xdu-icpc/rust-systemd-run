use zbus::zvariant::OwnedValue;

/// The error type of `systemd_run`.
///
/// The various errors that can be returned by this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error connecting to D-Bus.
    #[error("cannot connect to dbus: {0}")]
    DBusConnectionFail(zbus::Error),
    /// Invalid D-Bus path.
    #[error("cannot get a valid dbus path: {0}")]
    DBusInvalidPath(zbus::zvariant::Error),
    /// An error subscribing to D-Bus `PropertiesChanged` signal.
    #[error("cannot start listening property change events: {0}")]
    ListenPropertyChangeFail(zbus::Error),
    /// An error parsing the changed properties from the D-Bus
    /// `PropertiesChanged` signal.
    #[error("cannot parse the property change event: {0}")]
    ParsePropertyChangeFail(zbus::Error),
    /// An error quering one property of a unit.
    #[error("cannot query the property: {0}")]
    QueryPropertyFail(zbus::fdo::Error),
    /// An error calling systemd to start the transient unit.
    #[error("cannot start the transient service: {0}")]
    StartFail(zbus::Error),
    /// An error attempting to calculate the time usage of a service.
    #[error("cannot calculate {0} time usage: t0 = {1:?}, t1 = {2:?}")]
    TimeUsageFail(&'static str, Box<OwnedValue>, Box<OwnedValue>),
}

/// Alias for a [Result][std::result::Result] with the error type [Error].
pub type Result<T> = std::result::Result<T, Error>;
