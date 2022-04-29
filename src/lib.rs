#![doc = include_str!("../README.md")]

use byte_unit::Byte;
use std::time::Duration;
use zbus::fdo::{PropertiesChangedStream, PropertiesProxy};
use zbus::zvariant::{ObjectPath, Value};
use zbus::Connection;

mod error;
mod identity;
mod sd;

pub use error::{Error, Result};
pub use identity::Identity;

/// Information of a transient service for running on the system service
/// manager.
pub struct RunSystem {
    path: String,
    args: Vec<String>,
    service_name: Option<String>,
    collect_on_fail: bool,
    identity: Identity,
    runtime_max: Option<Duration>,
    memory_max: Option<Byte>,
    memory_swap_max: Option<Byte>,
    allowed_cpus: Vec<usize>,
    cpu_quota: Option<u64>,
    private_network: bool,
    private_ipc: bool,
}

/// Information of a transient service for running on the per-user service
/// manager.
pub struct RunUser {
    inner: RunSystem,
}

/// A transient service running.
pub struct StartedRun<'a> {
    proxy: zbus::fdo::PropertiesProxy<'a>,
    stream: PropertiesChangedStream<'a>,
}

/// A transient service finished.
#[derive(Debug)]
pub struct FinishedRun {
    failed: bool,
    wall_time_usage: Duration,
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

async fn listen_unit_property_change<'a>(
    bus: &Connection,
    unit: &ObjectPath<'a>,
) -> Result<(PropertiesProxy<'a>, PropertiesChangedStream<'a>)> {
    let proxy = PropertiesProxy::builder(bus)
        .path(unit)
        .expect("should not fail with validated path")
        .destination("org.freedesktop.systemd1")
        .expect("should not fail with hardcode dest")
        .build()
        .await
        .expect("should not fail with all info provided");
    let stream = proxy
        .receive_properties_changed()
        .await
        .map_err(Error::ListenPropertyChangeFail)?;
    Ok((proxy, stream))
}

impl RunUser {
    /// Create a new [RunUser] from a path to executable.
    pub fn new<T: AsRef<str>>(path: T) -> Self {
        Self {
            inner: RunSystem::new(path).identity(Identity::session()),
        }
    }

    /// Append an argument to the command line.
    pub fn arg<T: AsRef<str>>(self, arg: T) -> Self {
        Self {
            inner: self.inner.arg(arg),
        }
    }

    /// Set a custom name for the transient service.
    ///
    /// If the name is not terminated with `.service`, it will be appended
    /// automatically.
    pub fn service_name<T: AsRef<str>>(self, name: T) -> Self {
        Self {
            inner: self.inner.service_name(name),
        }
    }

    /// Unload the transient service even if it fails.
    ///
    /// This is not available if `systemd_236` is disabled.
    ///
    /// Read `CollectMode=` in [systemd.unit(5)](man:systemd.unit(5))
    /// for details.
    #[cfg(feature = "systemd_236")]
    pub fn collect_on_fail(self) -> Self {
        Self {
            inner: self.inner.collect_on_fail(),
        }
    }

    /// Configure a maximum time for the service to run.  If this is used
    /// and the service has been active for longer than the specified time
    /// it is terminated and put into a failure state.
    ///
    /// A [Duration] exceeding [u64::MAX] microseconds is trimmed to
    /// [u64::MAX] microseconds silently.
    ///
    /// Read `RuntimeMaxSec=` in
    /// [systemd.service(5)](man:systemd.service(5)) for details.
    pub fn runtime_max(self, d: Duration) -> Self {
        Self {
            inner: self.inner.runtime_max(d),
        }
    }

    /// Specify the absolute limit on memory usage of the executed
    /// processes in this unit. If memory usage cannot be contained under
    /// the limit, out-of-memory killer is invoked inside the unit.
    ///
    /// A [Byte] exceeding [u64::MAX] bytes is trimmed to [u64::MAX] bytes
    /// silently.
    ///
    /// Read `MemoryMax=` in
    /// [systemd.resource-control(5)](man:systemd.resource-control(5))
    /// for details.
    ///
    /// If the feature `systemd_231` is disabled, `MemoryLimit=` will be
    /// used instead if `MemoryMax=` for compatibility.
    pub fn memory_max(self, d: Byte) -> Self {
        Self {
            inner: self.inner.memory_max(d),
        }
    }

    /// Specify the absolute limit on swap usage of the executed
    /// processes in this unit.
    ///
    /// This setting is supported only if the unified control group is used,
    /// so it's not available if the feature `unified_cgroup` is disabled.
    ///
    /// A [Byte] exceeding [u64::MAX] bytes is trimmed to [u64::MAX] bytes
    /// silently.
    ///
    /// Read `MemorySwapMax=` in
    /// [systemd.resource-control(5)](man:systemd.resource-control(5))
    /// for details.
    #[cfg(feature = "unified_cgroup")]
    pub fn memory_swap_max(self, d: Byte) -> Self {
        Self {
            inner: self.inner.memory_swap_max(d),
        }
    }

    /// Start the transient service.
    pub async fn start<'a>(self) -> Result<StartedRun<'a>> {
        self.inner.start().await
    }
}

impl RunSystem {
    /// Create a new [RunSystem] from a path to executable.
    pub fn new<T: AsRef<str>>(path: T) -> Self {
        Self {
            path: path.as_ref().to_string(),
            args: vec![],
            service_name: None,
            collect_on_fail: false,
            identity: Identity::root(),
            runtime_max: None,
            memory_max: None,
            memory_swap_max: None,
            allowed_cpus: vec![],
            cpu_quota: None,
            private_network: false,
            private_ipc: false,
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
    /// [Identity::root()].
    pub fn identity(mut self, i: Identity) -> Self {
        self.identity = i;
        self
    }

    /// Unload the transient service even if it fails.
    ///
    /// This is not available if `systemd_236` is disabled.
    ///
    /// Read `CollectMode=` in [systemd.unit(5)](man:systemd.unit(5))
    /// for details.
    #[cfg(feature = "systemd_236")]
    pub fn collect_on_fail(mut self) -> Self {
        self.collect_on_fail = true;
        self
    }

    /// Configure a maximum time for the service to run.  If this is used
    /// and the service has been active for longer than the specified time
    /// it is terminated and put into a failure state.
    ///
    /// A [Duration] exceeding [u64::MAX] microseconds is trimmed to
    /// [u64::MAX] microseconds silently.
    ///
    /// Read `RuntimeMaxSec=` in
    /// [systemd.service(5)](man:systemd.service(5)) for details.
    pub fn runtime_max(mut self, d: Duration) -> Self {
        self.runtime_max = Some(d);
        self
    }

    /// Specify the absolute limit on memory usage of the executed
    /// processes in this unit. If memory usage cannot be contained under
    /// the limit, out-of-memory killer is invoked inside the unit.
    ///
    /// A [Byte] exceeding [u64::MAX] bytes is trimmed to [u64::MAX] bytes
    /// silently.
    ///
    /// Read `MemoryMax=` in
    /// [systemd.resource-control(5)](man:systemd.resource-control(5))
    /// for details.
    ///
    /// If the feature `systemd_231` is disabled, `MemoryLimit=` will be
    /// used instead if `MemoryMax=` for compatibility.
    pub fn memory_max(mut self, d: Byte) -> Self {
        self.memory_max = Some(d);
        self
    }

    /// Specify the absolute limit on swap usage of the executed
    /// processes in this unit.
    ///
    /// This setting is supported only if the unified control group is used,
    /// so it's not available if the feature `unified_cgroup` is disabled.
    ///
    /// A [Byte] exceeding [u64::MAX] bytes is trimmed to [u64::MAX] bytes
    /// silently.
    ///
    /// Read `MemorySwapMax=` in
    /// [systemd.resource-control(5)](man:systemd.resource-control(5))
    /// for details.
    #[cfg(feature = "unified_cgroup")]
    pub fn memory_swap_max(mut self, d: Byte) -> Self {
        self.memory_swap_max = Some(d);
        self
    }

    /// Assign the specified CPU time quota to the processes executed.
    /// Takes a percentage value.  The percentage specifies how much CPU
    /// time the unit shall get at maximum, relativeto the total CPU time
    /// available on one CPU. Use values > 100 for allotting CPU time on
    /// more than one CPU.
    ///
    /// The value will be trimmed to [u64::MAX] / 10000 silently if it
    /// exceeds this upper limit.
    ///
    /// Read `CPUQuota=` in
    /// [systemd.resource-control(5)](man:systemd.resource-control(5)) for
    /// details.
    #[cfg(feature = "systemd_213")]
    pub fn cpu_quota(mut self, percent: std::num::NonZeroU64) -> Self {
        self.cpu_quota = Some(percent.into());
        self
    }

    /// Restrict processes to be executed on specific CPUs.
    ///
    /// This setting doesn't guarantee that
    /// all of the CPUs will be used by the processes as it may be limited
    /// by parent units.
    ///
    /// Setting an empty list of CPUs will allow the processes of the unit
    /// to run on **all** CPUs.  This is also the default behavior if this
    /// is not used.
    ///
    /// Referring to an offline or non-existing CPU in this setting causes
    /// Systemd to **ignore this setting silently**.
    ///
    /// Read `AllowedCPUs=` in
    /// [systemd.resource-control(5)](man:systemd.resource-control(5))
    /// for details.
    ///
    /// This setting is supported only if the unified control group is used,
    /// so it's not available if the feature `unified_cgroup` is disabled.
    /// And, this setting is not available if the feature `systemd_244` is
    /// disabled.
    #[cfg(feature = "systemd_244")]
    #[cfg(feature = "unified_cgroup")]
    pub fn allowed_cpus(mut self, cpus: &[usize]) -> Self {
        self.allowed_cpus = cpus.to_owned();
        self
    }

    /// If this setting is used, sets up a new network namespace
    /// for the executed processes and configures only the loopback network
    /// device "lo" inside it. No other network devices will be available
    /// to the executed process. This is useful to turn off network access
    /// by the executed process.
    ///
    /// Read `PrivateNetwork=` in [systemd.exec(5)](man:systemd.exec(5)) for
    /// details.
    ///
    /// This setting is not available if the feature `systemd_227` is
    /// disabled.  And, it will be ignored silently if `CONFIG_NET_NS` is
    /// not enabled in the configuration of the running kernel.
    #[cfg(feature = "systemd_227")]
    pub fn private_network(mut self) -> Self {
        self.private_network = true;
        self
    }

    /// If this setting is used, sets up a new IPC namespace
    /// for the executed processes. Each IPC namespace has its own set of
    /// System V IPC identifiers and its own POSIX message queue file
    /// system. This is useful to avoid name clash of IPC identifiers.
    ///
    /// Read `PrivateIPC=` in [systemd.exec(5)](man:systemd.exec(5)) for
    /// details.
    ///
    /// This setting is not available if the feature `systemd_249` is
    /// disabled.  And, it will be ignored silently if `CONFIG_IPC_NS` is
    /// not enabled in the configuration of the running kernel.
    #[cfg(feature = "systemd_249")]
    pub fn private_ipc(mut self) -> Self {
        self.private_ipc = true;
        self
    }

    /// Start the transient service.
    pub async fn start<'a>(mut self) -> Result<StartedRun<'a>> {
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

        let identity_prop = identity::unit_properties(&self.identity);
        properties.extend(identity_prop);

        if let Some(d) = &self.runtime_max {
            let usec = u64::try_from(d.as_micros()).unwrap_or(u64::MAX);
            properties.push(("RuntimeMaxUSec", Value::from(usec)));
        }

        if !self.allowed_cpus.is_empty() {
            let mut cpu_set = vec![];
            for &cpu in &self.allowed_cpus {
                let (x, y) = (cpu / 8, cpu % 8);
                if cpu_set.len() <= x {
                    cpu_set.resize(x + 1, 0u8);
                }
                cpu_set[x] |= 1 << y;
            }
            properties.push(("AllowedCPUs", Value::from(cpu_set)));
        }

        let memory_max_name = if cfg!(feature = "systemd_231") {
            "MemoryMax"
        } else {
            "MemoryLimit"
        };

        for (k, v) in [
            (memory_max_name, &self.memory_max),
            ("MemorySwapMax", &self.memory_swap_max),
        ] {
            if let Some(v) = v {
                let b = u64::try_from(v.get_bytes()).unwrap_or(u64::MAX);
                properties.push((k, Value::from(b)))
            }
        }

        if let Some(v) = self.cpu_quota {
            let v = std::cmp::min(v, u64::MAX / 10000);
            properties.push(("CPUQuotaPerSecUSec", Value::from(v * 10000)));
        }

        for (k, v) in [
            ("PrivateNetwork", self.private_network),
            ("PrivateIPC", self.private_ipc),
        ] {
            // Don't push false values as they may break on old Systemd.
            if v {
                properties.push((k, Value::from(true)))
            }
        }

        let properties = properties.iter().map(|(x, y)| (*x, y)).collect::<Vec<_>>();

        let bus = if identity::is_session(&self.identity) {
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
        let (proxy, stream) = listen_unit_property_change(&bus, &unit_path).await?;

        sd::SystemdManagerProxy::builder(&bus)
            .build()
            .await
            .expect("should not fail with hardcoded parameters in sd.rs")
            .start_transient_unit(unit_name, "fail", &properties, &[])
            .await
            .map_err(Error::StartFail)
            .map(|_| StartedRun { stream, proxy })
    }
}

impl<'a> StartedRun<'a> {
    /// Wait until a [StartedRun] is finished.
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

        let iface = zbus_names::InterfaceName::try_from("org.freedesktop.systemd1.Unit")
            .expect("should not fail with hardcoded str");

        let t0 = self
            .proxy
            .get(iface.as_ref(), "InactiveExitTimestampMonotonic")
            .await
            .map_err(Error::QueryPropertyFail)?;

        let t1 = self
            .proxy
            .get(iface.as_ref(), "InactiveEnterTimestampMonotonic")
            .await
            .map_err(Error::QueryPropertyFail)?;

        let time_usage_us = match (t0.downcast_ref(), t1.downcast_ref()) {
            (Some(Value::U64(t0)), Some(Value::U64(t1))) => t1 - t0,
            _ => {
                let t0 = Box::new(t0);
                let t1 = Box::new(t1);
                return Err(Error::TimeUsageFail("wall", t0, t1));
            }
        };

        let failed = active_state.unwrap() == "failed";
        let wall_time_usage = Duration::from_micros(time_usage_us);
        Ok(FinishedRun {
            failed,
            wall_time_usage,
        })
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

    /// Get the usage of wall-clock time of the finished transient service.
    pub fn wall_time_usage(&self) -> Duration {
        self.wall_time_usage
    }
}
