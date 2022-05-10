#[allow(dead_code)]
enum Priv {
    Bind {
        src: String,
        rw: bool,
        ignore_nonexist: bool,
        recursive: bool,
    },
    Tmpfs {
        rw: bool,
        opts: Vec<String>,
    },
    Normal {
        src: String,
        rw: bool,
        ignore_nonexist: bool,
        opts: Vec<String>,
    },
}

/// The description of a mount.
pub struct Mount(Priv);

impl Mount {
    /// Create a new bind mount.
    ///
    /// Read `BindPaths` and `BindReadOnlyPaths` in
    /// [systemd.exec(5)](man:systemd.exec(5)) for details.
    ///
    /// This setting is not available if the feature `systemd_233` is
    /// disabled.
    #[cfg(feature = "systemd_233")]
    pub fn bind<T: AsRef<str>>(src: T) -> Self {
        Self(Priv::Bind {
            src: src.as_ref().to_owned(),
            rw: false,
            recursive: false,
            ignore_nonexist: false,
        })
    }

    /// Create a tmpfs mount.
    ///
    /// Read `TemporaryFileSystem` in [systemd.exec(5)](man:systemd.exec(5))
    /// for details.
    ///
    /// This setting is not available if the feature `systemd_238` is
    /// disabled.
    #[cfg(feature = "systemd_238")]
    pub fn tmpfs() -> Self {
        Self(Priv::Tmpfs {
            rw: false,
            opts: vec![],
        })
    }

    /// Create a normal mount.
    ///
    /// Read `MountImages` in [systemd.exec(5)](man:systemd.exec(5)) for
    /// details.
    ///
    /// This setting is not available if the feature `systemd_247` is
    /// disabled.
    #[cfg(feature = "systemd_247")]
    pub fn normal<T: AsRef<str>>(src: T) -> Self {
        Self(Priv::Normal {
            src: src.as_ref().to_owned(),
            rw: false,
            ignore_nonexist: false,
            opts: vec![],
        })
    }

    /// Make the [Mount] writable.
    pub fn writable(self) -> Self {
        match self {
            Self(Priv::Bind {
                src,
                recursive,
                ignore_nonexist,
                ..
            }) => Self(Priv::Bind {
                src,
                recursive,
                ignore_nonexist,
                rw: true,
            }),
            Self(Priv::Tmpfs { opts, .. }) => Self(Priv::Tmpfs { opts, rw: true }),
            Self(Priv::Normal {
                opts,
                src,
                ignore_nonexist,
                ..
            }) => Self(Priv::Normal {
                opts,
                src,
                ignore_nonexist,
                rw: true,
            }),
        }
    }

    /// Make the [Mount] recursive.  It only makes a difference for a bind
    /// mount.
    pub fn recursive(self) -> Self {
        match self {
            Self(Priv::Bind { src, rw, .. }) => Self(Priv::Bind {
                src,
                rw,
                recursive: true,
                ignore_nonexist: true,
            }),
            _ => self,
        }
    }

    /// Append a mount option.  If the option contains a comma (`,`), or
    /// it is `ro`, `rw`, or empty, or it's applied for a bind mount,
    /// [None] will be returned.  But the option will not be validated
    /// further.
    ///
    /// For `rw`, use [Self::writable] instead.
    pub fn opt<T: AsRef<str>>(self, option: T) -> Option<Self> {
        let o = option.as_ref();
        if o.contains(',') {
            return None;
        }
        match o {
            "" | "ro" | "rw" => return None,
            _ => {}
        }
        match self {
            Self(Priv::Bind { .. }) => None,
            Self(Priv::Tmpfs { rw, mut opts }) => {
                opts.push(o.to_owned());
                Some(Self(Priv::Tmpfs { rw, opts }))
            }
            Self(Priv::Normal {
                src,
                rw,
                mut opts,
                ignore_nonexist,
            }) => {
                opts.push(o.to_owned());
                Some(Self(Priv::Normal {
                    src,
                    rw,
                    opts,
                    ignore_nonexist,
                }))
            }
        }
    }

    /// Ignore the mount if the source does not exist.  This does not make
    /// any difference for tmpfs mounts.
    pub fn ignore_nonexist(self) -> Self {
        match self {
            Self(Priv::Tmpfs { .. }) => self,
            Self(Priv::Normal { src, rw, opts, .. }) => Self(Priv::Normal {
                src,
                rw,
                opts,
                ignore_nonexist: true,
            }),
            Self(Priv::Bind {
                src, rw, recursive, ..
            }) => Self(Priv::Bind {
                src,
                rw,
                recursive,
                ignore_nonexist: true,
            }),
        }
    }
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace(':', "\\:")
        .replace(' ', "\\ ")
}

pub enum MarshaledMount {
    /// src, dest, ignore_nonexist, flags (MS_REC or 0)
    Bind(String, String, bool, u64),
    /// src, dest, ignore_nonexist, flags (MS_REC or 0)
    BindReadOnly(String, String, bool, u64),
    /// src, dest, ignore_nonexist, `[(GPT label, flags)]`
    /// Currently only `root` used as `GPT label`
    Normal(String, String, bool, Vec<(&'static str, String)>),
    /// dest, flags (joined with comma)
    Tmpfs(String, String),
}

pub fn marshal<T: AsRef<str>>(mount_point: T, mount: Mount) -> MarshaledMount {
    use MarshaledMount::*;
    let mp = escape(mount_point.as_ref());

    match mount {
        Mount(Priv::Bind {
            src,
            rw,
            ignore_nonexist,
            recursive,
        }) => {
            let src = escape(&src);
            let flags: u64 = match recursive {
                true => 16384, // MS_REC
                false => 0,
            };
            match rw {
                true => Bind(src, mp, ignore_nonexist, flags),
                false => BindReadOnly(src, mp, ignore_nonexist, flags),
            }
        }
        Mount(Priv::Normal {
            src,
            rw,
            ignore_nonexist,
            mut opts,
        }) => {
            let src = escape(&src);
            if !rw {
                opts.push("ro".into());
            }
            let opts = opts.into_iter().map(|x| ("root", x)).collect();
            Normal(src, mp, ignore_nonexist, opts)
        }
        Mount(Priv::Tmpfs { rw, mut opts }) => {
            if !rw {
                opts.push("ro".into());
            }
            Tmpfs(mp, opts.join(","))
        }
    }
}
