/// An enum representing the identity for running a transient service.
pub enum Identity {
    /// Run the transient service in the per-user instance of the service
    /// manager.  This is the default behavior.
    Session,
    /// Run the transient service as the the UNIX user `x` and group `y`
    /// for `UserGroup(x, y)`.
    ///
    /// You need to be the `root` user to start a transient service with
    /// this.  Read `User=` and `Group=` in
    /// [`systemd.exec(5)`](man:systemd.exec(5)) for details.
    UserGroup(String, String),
    /// Run the transient service asa UNIX user and group pair dynamically
    /// allocated.
    ///
    /// You need to be the `root` user to start a transient service with
    /// this.  Read `DynamicUser=` in
    /// [`systemd.exec(5)`](man:systemd.exec(5)) for details.
    Dynamic,
}

impl Identity {
    /// Create an `Identity::UserGroup` with an UNIX user, and the UNIX
    /// group with the same name as the user.
    pub fn user<T: AsRef<str>>(u: T) -> Self {
        let u = u.as_ref().to_owned();
        let g = u.clone();
        Self::UserGroup(u, g)
    }

    /// Create an `Identity::UserGroup` with an UNIX user and an UNIX group.
    pub fn user_group<U: AsRef<str>, G: AsRef<str>>(u: U, g: G) -> Self {
        Self::UserGroup(u.as_ref().to_owned(), g.as_ref().to_owned())
    }

    /// Shorthand for `Self::user("root")`.
    pub fn root() -> Self {
        Self::user("root")
    }
}
