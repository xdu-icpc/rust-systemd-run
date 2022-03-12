#![doc = include_str!("../README.md")]

use zbus::fdo::{PropertiesChangedStream, PropertiesProxy};
use zbus::zvariant::{ObjectPath, Value};
use zbus::Connection;

mod error;
mod identity;
mod sd;

pub use error::{Error, Result};
pub use identity::Identity;

/// Information of a transient service.
pub struct Run {
    path: String,
    args: Vec<String>,
    service_name: Option<String>,
    collect_on_fail: bool,
    identity: Identity,
}

/// A transient service running.
pub struct StartedRun<'a> {
    stream: PropertiesChangedStream<'a>,
}

/// A transient service finished.
#[derive(Debug)]
pub struct FinishedRun {
    failed: bool,
}

// The logic is "borrowed" from systemd/src/run.c.
fn default_unit_name(bus: &zbus::Connection) -> Result<String> {
    bus.unique_name()
        .map_or_else(
            || {
                // We couldn't get the unique name, which is a pretty
                // common case if we are connected to systemd directly.
                // In that case, just pick a random uuid as name.
                Ok(('r', uuid::Uuid::new_v4().to_simple().to_string()))
            },
            |s| {
                for p in [":1.", ":"] {
                    if let Some(s) = s.strip_prefix(p) {
                        return Ok(('u', s.to_owned()));
                    }
                }
                unreachable!("zbus should have rejected invalid name");
            },
        )
        .map(|(tp, id)| format!("run-{}{}.service", tp, id))
}

fn escape_byte_for_object_path(b: u8) -> String {
    if b.is_ascii_alphanumeric() {
        std::str::from_utf8(&[b])
            .expect("[0-9a-zA-Z] is valid UTF-8")
            .to_owned()
    } else {
        format!("_{:02x}", b)
    }
}

fn object_path_from_unit_name<'a, 'b>(s: &'a str) -> Result<ObjectPath<'b>> {
    let path_string = "/org/freedesktop/systemd1/unit/".to_owned()
        + &s.bytes()
            .map(escape_byte_for_object_path)
            .collect::<Vec<_>>()
            .join("");
    ObjectPath::try_from(path_string).map_err(Error::DBusInvalidPath)
}

async fn stream_for_unit_property_change<'a>(
    bus: &Connection,
    unit: &ObjectPath<'a>,
) -> Result<PropertiesChangedStream<'a>> {
    PropertiesProxy::builder(bus)
        .path(unit)
        .expect("should not fail with validated path")
        .destination("org.freedesktop.systemd1")
        .expect("should not fail with hardcode dest")
        .build()
        .await
        .expect("should not fail with all info provided")
        .receive_properties_changed()
        .await
        .map_err(Error::ListenPropertyChangeFail)
}

impl Run {
    /// Create a new Run from a path to executable.
    pub fn new<T: AsRef<str>>(path: T) -> Self {
        Self {
            path: path.as_ref().to_string(),
            args: vec![],
            service_name: None,
            collect_on_fail: false,
            identity: Identity::Session,
        }
    }

    /// Append an argument to the command line.
    pub fn arg<T: AsRef<str>>(mut self, arg: T) -> Self {
        self.args.push(arg.as_ref().to_string());
        self
    }

    /// Set a custom name for the transient service.
    ///
    /// If the name is not terminated with `.service`, it will be appended
    /// automatically.
    pub fn service_name<T: AsRef<str>>(mut self, name: T) -> Self {
        let mut name = name.as_ref().to_owned();
        if !name.ends_with(".service") {
            name += ".service";
        }
        self.service_name = Some(name);
        self
    }

    /// Set an identity to run the transient service.  The default is
    /// `Identity::Session`.
    pub fn identity(mut self, i: Identity) -> Self {
        self.identity = i;
        self
    }

    /// Unload the transient service even if it fails.
    ///
    /// Read `CollectMode=` in [systemd.unit(5)](man:systemd.unit(5))
    /// for details.
    pub fn collect_on_fail(mut self) -> Self {
        self.collect_on_fail = true;
        self
    }

    /// Start the transient service.
    pub async fn start<'a>(mut self) -> Result<StartedRun<'a>> {
        let bus = if matches!(self.identity, Identity::Session) {
            Connection::session().await
        } else {
            Connection::system().await
        }
        .map_err(Error::DBusConnectionFail)?;
        if self.service_name.is_none() {
            self.service_name = Some(default_unit_name(&bus)?);
        }
        let unit_name = self.service_name.as_ref().unwrap();
        let unit_path = object_path_from_unit_name(unit_name)?;

        // We must do this before really telling systemd to start the
        // service.  Or we may miss D-Bus signals, causing StartedRun::wait
        // to hang forever.  And this also prevents the start of the
        // transient service in case this fails.
        let stream = stream_for_unit_property_change(&bus, &unit_path).await?;

        let mut argv = vec![&self.path];
        argv.extend(&self.args);

        let exec_start = vec![(&self.path, &argv, false)];

        let mut properties = vec![
            ("Description", Value::from(&self.path)),
            ("ExecStart", Value::from(&exec_start)),
            ("AddRef", Value::from(true)),
        ];

        if self.collect_on_fail {
            let prop = ("CollectMode", Value::from("inactive-or-failed"));
            properties.push(prop);
        }

        match &self.identity {
            Identity::UserGroup(u, g) => {
                let prop = [
                    ("User", Value::from(u.clone())),
                    ("Group", Value::from(g.clone())),
                ];
                properties.extend(prop);
            }
            Identity::Dynamic => {
                properties.push(("DynamicUser", Value::from(true)));
            }
            Identity::Session => {}
        }

        let properties = properties.iter().map(|(x, y)| (*x, y)).collect::<Vec<_>>();

        sd::SystemdManagerProxy::builder(&bus)
            .build()
            .await
            .expect("should not fail with hardcoded parameters in sd.rs")
            .start_transient_unit(unit_name, "fail", &properties, &[])
            .await
            .map_err(Error::StartFail)
            .map(|_| StartedRun { stream })
    }
}

impl<'a> StartedRun<'a> {
    /// Wait until a `StartedRun` is finished.
    pub async fn wait(self) -> Result<FinishedRun> {
        let mut stream = self.stream;
        let mut has_job = false;
        let mut active_state = None;
        let no_job = Value::from((0u32, ObjectPath::try_from("/").unwrap()));
        use futures::stream::StreamExt;
        while let Some(ev) = stream.next().await {
            let changed = &ev
                .args()
                .map_err(Error::ParsePropertyChangeFail)?
                .changed_properties;
            if let Some(Value::Str(state)) = changed.get("ActiveState") {
                active_state = Some(state.as_str().to_owned());
            }
            if let Some(job) = changed.get("Job") {
                has_job = job != &no_job;
            }
            match (has_job, active_state.as_deref()) {
                (false, Some("inactive")) => break,
                (false, Some("failed")) => break,
                _ => {}
            }
        }

        let failed = active_state.unwrap() == "failed";
        Ok(FinishedRun { failed })
    }
}

impl FinishedRun {
    /// Check if the `FinishedRun` has failed.
    ///
    /// Read `SuccessExitStatus=` in
    /// [systemd.service(5)](man:systemd.service(5)) for details.
    pub fn is_failed(&self) -> bool {
        self.failed
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_root;
