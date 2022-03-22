use zbus::zvariant::Value;

enum IdentityInner {
    Session,
    UserGroup(String, String),
    #[allow(dead_code)]
    Dynamic,
}

/// The identity for running a transient service.
pub struct Identity(IdentityInner);

impl Identity {
    /// Run the transient service as the the UNIX user `x` and group `y`
    /// for `UserGroup(x, y)`.
    ///
    /// You need to be the `root` user to start a transient service with
    /// this.  Read `User=` and `Group=` in
    /// [`systemd.exec(5)`](man:systemd.exec(5)) for details.
    pub fn user_group<U: AsRef<str>, G: AsRef<str>>(u: U, g: G) -> Self {
        Self(IdentityInner::UserGroup(
            u.as_ref().to_owned(),
            g.as_ref().to_owned(),
        ))
    }

    /// Shorthand for `Self::user_group(u, u)`.
    pub fn user<T: AsRef<str>>(u: T) -> Self {
        Self::user_group(u.as_ref(), u.as_ref())
    }

    /// Shorthand for `Self::user("root")`.
    pub fn root() -> Self {
        Self::user("root")
    }

    /// Run the transient service as a UNIX user and group pair dynamically
    /// allocated.
    ///
    /// This is unavailable if the feature `systemd_231` is disabled.
    ///
    /// You need to be the `root` user to start a transient service with
    /// this.  Read `DynamicUser=` in
    /// [`systemd.exec(5)`](man:systemd.exec(5)) for details.
    #[cfg(feature = "systemd_231")]
    pub fn dynamic() -> Self {
        Self(IdentityInner::Dynamic)
    }

    /// Run the transient service in the per-user instance of the service
    /// manager.  This is the default behavior.
    pub fn session() -> Self {
        Self(IdentityInner::Session)
    }
}

pub fn is_session(i: &Identity) -> bool {
    matches!(i, Identity(IdentityInner::Session))
}

pub fn unit_properties(i: &Identity) -> Vec<(&'static str, Value)> {
    match i {
        Identity(IdentityInner::Session) => vec![],
        Identity(IdentityInner::UserGroup(u, g)) => vec![
            ("User", Value::from(u.clone())),
            ("Group", Value::from(g.clone())),
        ],
        Identity(IdentityInner::Dynamic) => vec![("DynamicUser", Value::from(true))],
    }
}
