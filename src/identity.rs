use zbus::zvariant::Value;

enum IdentityInner {
    Session,
    UserGroup(String, String),
    #[allow(dead_code)]
    Dynamic,
}

pub struct Identity(IdentityInner);

impl Identity {
    pub fn user_group<U: AsRef<str>, G: AsRef<str>>(u: U, g: G) -> Self {
        Self(IdentityInner::UserGroup(
            u.as_ref().to_owned(),
            g.as_ref().to_owned(),
        ))
    }

    pub fn dynamic() -> Self {
        Self(IdentityInner::Dynamic)
    }

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
