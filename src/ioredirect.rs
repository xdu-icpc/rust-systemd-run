#[allow(dead_code)]
pub enum Priv {
    Inherit,
    Null,
    Journal,
    File(String),
    Truncate(String),
    Append(String),
}

impl Priv {
    fn marshal(self) -> (&'static str, String) {
        use Priv::*;
        match self {
            Inherit => ("", "inherit".to_string()),
            Null => ("", "null".to_string()),
            Journal => ("", "journal".to_string()),
            File(x) => ("File", x),
            Truncate(x) => ("FileToTruncate", x),
            Append(x) => ("FileToAppend", x),
        }
    }
}

/// The description of a input.
///
/// Read `StandardInput` in [systemd.exec(5)](man:systemd.exec(5)) for
/// details.
pub struct InputSpec(Priv);

impl InputSpec {
    /// Connect `/dev/null` to the input.
    pub fn null() -> Self {
        Self(Priv::Null)
    }

    /// Specify the a path where the file is connected to the input.
    ///
    /// This setting will be unavailable if the feature `systemd_236` is
    /// disabled.
    #[cfg(feature = "systemd_236")]
    pub fn file<T: AsRef<str>>(path: T) -> Self {
        Self(Priv::File(path.as_ref().to_owned()))
    }
}

pub fn marshal_input(spec: InputSpec) -> (&'static str, String) {
    spec.0.marshal()
}

/// The description of a output.
///
/// Read `StandardOutput` and `StandardError` in
/// [systemd.exec(5)](man:systemd.exec(5)) for details.
pub struct OutputSpec(Priv);

impl OutputSpec {
    /// For `stdout`, use the same file for `stdin`.
    /// For `stderr`, use the same file for `stdout`.
    pub fn inherit() -> Self {
        Self(Priv::Inherit)
    }

    /// Connect `/dev/null` to the output.
    pub fn null() -> Self {
        Self(Priv::Null)
    }

    /// Log the output into system journal.
    pub fn journal() -> Self {
        Self(Priv::Journal)
    }

    /// Specify a path where the file will be overwritten from offset 0.
    /// If the file does not exist, it will be created.
    ///
    /// This setting will be unavailable if the feature `systemd_236` is
    /// disabled.
    #[cfg(feature = "systemd_236")]
    pub fn file<T: AsRef<str>>(path: T) -> Self {
        Self(Priv::File(path.as_ref().to_owned()))
    }

    /// Like [OutputSpec::file], but the file will be truncated before
    /// written.
    ///
    /// This setting will be unavailable if the feature `systemd_248` is
    /// disabled.
    #[cfg(feature = "systemd_248")]
    pub fn truncate<T: AsRef<str>>(path: T) -> Self {
        Self(Priv::Truncate(path.as_ref().to_owned()))
    }

    /// Like [OutputSpec::file], but the file will be written from its end,
    /// instead of offset 0.
    ///
    /// This setting will be unavailable if the feature `systemd_240` is
    /// disabled.
    #[cfg(feature = "systemd_240")]
    pub fn append<T: AsRef<str>>(path: T) -> Self {
        Self(Priv::Append(path.as_ref().to_owned()))
    }
}

pub fn marshal_output(spec: OutputSpec) -> (&'static str, String) {
    spec.0.marshal()
}
