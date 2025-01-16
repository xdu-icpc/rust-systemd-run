#![doc = include_str!("../README.md")]

use byte_unit::Byte;
use std::num::NonZeroU64;
use std::time::Duration;
use zbus::fdo::{PropertiesChangedStream, PropertiesProxy};
use zbus::zvariant::{ObjectPath, Value};
use zbus::Connection;

mod cpu_sched;
mod error;
mod identity;
mod ioredirect;
mod mount;
mod sd;

pub use cpu_sched::CpuScheduling;
pub use error::{Error, Result};
pub use identity::Identity;
pub use ioredirect::{InputSpec, OutputSpec};
pub use mount::Mount;

#[allow(dead_code)]
enum ProtectProcInternal {
    NoAccess,
    Invisible,
    Ptraceable,
    Default,
}

/// Controls the `hidepid=` mount option of the `procfs` instance in the
/// private namespace of the unit.
///
/// Read `ProtectProc=` in [systemd.exec(5)](man:systemd.exec(5)) for
/// details.
#[cfg(feature = "systemd_247")]
pub struct ProtectProc(ProtectProcInternal);

#[cfg(feature = "systemd_247")]
impl ProtectProc {
    /// Take away the ability to access most of other users' process
    /// metadata
    pub fn no_access() -> Self {
        Self(ProtectProcInternal::NoAccess)
    }
    /// Processes owned by other users are hidden
    pub fn invisible() -> Self {
        Self(ProtectProcInternal::Invisible)
    }
    /// Processes not traceable by the unit are hidden
    pub fn ptraceable() -> Self {
        Self(ProtectProcInternal::Ptraceable)
    }
}

#[cfg(feature = "systemd_247")]
impl Default for ProtectProc {
    /// No protection
    fn default() -> Self {
        Self(ProtectProcInternal::Default)
    }
}

/// Information of a transient service for running on the system service
/// manager.
pub struct RunSystem {
    path: String,
    args: Vec<String>,
    service_name: Option<String>,
    collect_on_fail: bool,
    identity: identity::Identity,
    runtime_max: Option<Duration>,
    memory_max: Option<Byte>,
    memory_swap_max: Option<Byte>,
    allowed_cpus: Vec<usize>,
    cpu_quota: Option<u64>,
    private_network: bool,
    private_ipc: bool,
    mount: Vec<(String, Mount)>,
    mount_api_vfs: bool,
    private_devices: bool,
    no_new_privileges: bool,
    limit_fsize: Option<Byte>,
    limit_fsize_soft: Option<Byte>,
    limit_stack: Option<Byte>,
    limit_stack_soft: Option<Byte>,
    limit_core: Option<Byte>,
    limit_core_soft: Option<Byte>,
    limit_nofile: Option<u64>,
    limit_nofile_soft: Option<u64>,
    limit_nproc: Option<u64>,
    limit_nproc_soft: Option<u64>,
    stdin: Option<InputSpec>,
    stdout: Option<OutputSpec>,
    stderr: Option<OutputSpec>,
    current_dir: Option<String>,
    protect_proc: ProtectProcInternal,
    slice: Option<String>,
    private_users: bool,
    timeout_stop: Option<Duration>,
    cpu_sched: CpuScheduling,
    joins_namespace_of: Vec<String>,
}

/// Information of a transient service for running on the per-user service
/// manager.
pub struct RunUser(RunSystem);

/// A transient service running.
pub struct StartedRun<'a> {
    proxy: zbus::fdo::PropertiesProxy<'a>,
    stream: PropertiesChangedStream,
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
                Ok(('r', uuid::Uuid::new_v4().simple().to_string()))
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

fn object_path_from_unit_name<'a>(s: &str) -> Result<ObjectPath<'a>> {
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
) -> Result<(PropertiesProxy<'a>, PropertiesChangedStream)> {
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
        Self(RunSystem {
            identity: identity::session(),
            ..RunSystem::new(path)
        })
    }

    /// Append an argument to the command line.
    pub fn arg<T: AsRef<str>>(self, arg: T) -> Self {
        Self(self.0.arg(arg))
    }

    /// Append multiple arguments to the command line.
    pub fn args<T: AsRef<str>, I: IntoIterator<Item = T>>(self, args: I) -> Self {
        Self(self.0.args(args))
    }

    /// Set a custom name for the transient service.
    ///
    /// If the name is not terminated with `.service`, it will be appended
    /// automatically.
    pub fn service_name<T: AsRef<str>>(self, name: T) -> Self {
        Self(self.0.service_name(name))
    }

    /// Unload the transient service even if it fails.
    ///
    /// This is not available if `systemd_236` is disabled.
    ///
    /// Read `CollectMode=` in [systemd.unit(5)](man:systemd.unit(5))
    /// for details.
    #[cfg(feature = "systemd_236")]
    pub fn collect_on_fail(self) -> Self {
        Self(self.0.collect_on_fail())
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
    ///
    /// This setting will be unavailable with the feature `systemd_229`
    /// disabled.
    #[cfg(feature = "systemd_229")]
    pub fn runtime_max(self, d: Duration) -> Self {
        Self(self.0.runtime_max(d))
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
        Self(self.0.memory_max(d))
    }

    /// Specify the absolute limit on swap usage of the executed
    /// processes in this unit.
    ///
    /// This setting is supported only if the unified control group is used,
    /// so it's not available if the feature `unified_cgroup` is disabled.
    /// And it will be unavailable with `systemd_232` disabled.
    ///
    /// A [Byte] exceeding [u64::MAX] bytes is trimmed to [u64::MAX] bytes
    /// silently.
    ///
    /// Read `MemorySwapMax=` in
    /// [systemd.resource-control(5)](man:systemd.resource-control(5))
    /// for details.
    #[cfg(feature = "unified_cgroup")]
    #[cfg(feature = "systemd_232")]
    pub fn memory_swap_max(self, d: Byte) -> Self {
        Self(self.0.memory_swap_max(d))
    }

    /// Set soft and hard limits of the maximum size in bytes of files that
    /// the process may create.
    ///
    /// Read `LimitFSIZE=` in [systemd.exec(5)](man:systemd.exec(5)) and
    /// `RLIMIT_FSIZE` in [prlimit(2)](man:prlimit(2)) for details.
    ///
    /// Any setting exceeding [u64::MAX] bytes will be trimmed to [u64::MAX]
    /// bytes silently.  And, if `soft` is greater than `hard`, it will be
    /// trimmed to `hard` silently.
    ///
    /// Unlike [RunSystem::limit_fsize_soft_hard], this can't be used to
    /// increase the hard limit because of insufficient privileges.
    pub fn limit_fsize_soft_hard(self, soft: Byte, hard: Byte) -> Self {
        Self(self.0.limit_fsize_soft_hard(soft, hard))
    }

    /// Shorthand for `self.limit_fsize_soft_hard(lim, lim)`.
    pub fn limit_fsize(self, lim: Byte) -> Self {
        self.limit_fsize_soft_hard(lim, lim)
    }

    /// Set soft and hard limits of the maximum size in bytes of files that
    /// the process may create.
    ///
    /// Any setting exceeding [u64::MAX] bytes will be trimmed to
    /// [u64::MAX] bytes silently.  And, if `soft` is greater than `hard`,
    /// it will be trimmed to `hard` silently.
    ///
    /// Read `LimitCORE=` in [systemd.exec(5)](man:systemd.exec(5)) and
    /// `RLIMIT_CORE` in [prlimit(2)](man:prlimit(2)) for details.
    ///
    /// Unlike [RunSystem::limit_core_soft_hard], this can't be used to
    /// increase the hard limit because of insufficient privileges.
    pub fn limit_core_soft_hard(self, soft: Byte, hard: Byte) -> Self {
        Self(self.0.limit_core_soft_hard(soft, hard))
    }

    /// Shorthand for `self.limit_fsize_soft_hard(lim, lim)`.
    pub fn limit_core(self, lim: Byte) -> Self {
        self.limit_core_soft_hard(lim, lim)
    }

    /// Set soft and hard limits of the number of threads for the real user
    /// ID of the process.
    ///
    /// If `soft` is greater than `hard`, it will be trimmed to `hard`
    /// silently.
    ///
    /// Read `LimitNPROC=` in [systemd.exec(5)](man:systemd.exec(5)) and
    /// `RLIMIT_NPROC` in [prlimit(2)](man:prlimit(2)) for details.
    ///
    /// Unlike [RunSystem::limit_nproc_soft_hard], this can't be used to
    /// increase the hard limit because of insufficient privileges.
    pub fn limit_nproc_soft_hard(self, soft: NonZeroU64, hard: NonZeroU64) -> Self {
        Self(self.0.limit_nproc_soft_hard(soft, hard))
    }

    /// Shorthand for `self.limit_nproc_soft_hard(lim, lim)`.
    pub fn limit_nproc(self, lim: NonZeroU64) -> Self {
        self.limit_nproc_soft_hard(lim, lim)
    }

    /// Set soft and hard limits of the number of threads for the real user
    /// ID of the process.
    ///
    /// If `soft` is greater than `hard`, it will be trimmed to `hard`
    /// silently.
    ///
    /// Read `LimitNOFILE=` in [systemd.exec(5)](man:systemd.exec(5)) and
    /// `RLIMIT_NOFILE` in [prlimit(2)](man:prlimit(2)) for details.
    ///
    /// Unlike [RunSystem::limit_nofile_soft_hard], this can't be used to
    /// increase the hard limit because of insufficient privileges.
    pub fn limit_nofile_soft_hard(self, soft: NonZeroU64, hard: NonZeroU64) -> Self {
        Self(self.0.limit_nofile_soft_hard(soft, hard))
    }

    /// Shorthand for `self.limit_nofile_soft_hard(lim, lim)`.
    pub fn limit_nofile(self, lim: NonZeroU64) -> Self {
        self.limit_nofile_soft_hard(lim, lim)
    }

    /// Set the soft and hard limit on the size of the process stack.
    ///
    /// If `soft` is greater than `hard`, it will be trimmed to `hard`
    /// silently.
    ///
    /// Read `LimitSTACK=` in [systemd.exec(5)](man:systemd.exec(5)) and
    /// `RLIMIT_STACK` in [prlimit(2)](man:prlimit(2)) for details.
    ///
    /// Unlike [RunSystem::limit_stack_soft_hard], this can't be used to
    /// increase the hard limit because of insufficient privileges.
    pub fn limit_stack_soft_hard(self, soft: Byte, hard: Byte) -> Self {
        Self(self.0.limit_stack_soft_hard(soft, hard))
    }

    /// Shorthand for `self.limit_stack_soft_hard(lim, lim)`.
    pub fn limit_stack(self, lim: Byte) -> Self {
        self.limit_stack_soft_hard(lim, lim)
    }

    /// Controls where file descriptor 0 (STDIN) of the executed processes
    /// is connected to.
    ///
    /// Read [InputSpec] and `StandardInput=` in
    /// [systemd.exec(5)](man:systemd.exec(5)) for details.
    ///
    /// The default is [InputSpec::null()].
    pub fn stdin(self, spec: InputSpec) -> Self {
        Self(self.0.stdin(spec))
    }

    /// Controls where file descriptor 1 (STDOUT) of the executed processes
    /// is connected to.
    ///
    /// Read [OutputSpec] and `StandardOutput=` in
    /// [systemd.exec(5)](man:systemd.exec(5)) for details.
    ///
    /// The default depends on system configuration.
    pub fn stdout(self, spec: OutputSpec) -> Self {
        Self(self.0.stdout(spec))
    }

    /// Controls where file descriptor 2 (STDERR) of the executed processes
    /// is connected to.
    ///
    /// Read [OutputSpec] and `StandardError=` in
    /// [systemd.exec(5)](man:systemd.exec(5)) for details.
    ///
    /// The default depends on system configuration.
    pub fn stderr(self, spec: OutputSpec) -> Self {
        Self(self.0.stderr(spec))
    }

    /// Sets the working directory for executed processes.
    ///
    /// Read `WorkingDirectory=` in
    /// [systemd.exec(5)](man:systemd.exec(5)) for details.
    ///
    /// This setting is unavailable with the feature `systemd_227`
    /// disabled.
    #[cfg(feature = "systemd_227")]
    pub fn current_dir<P: AsRef<str>>(self, path: P) -> Self {
        Self(self.0.current_dir(path))
    }

    /// Put the transient service into a slice.
    ///
    /// Read `Slice=` in
    /// [systemd.resource-control(5)](man:systemd.resource-control(5))
    /// for details.
    pub fn slice<S: AsRef<str>>(self, slice: S) -> Self {
        Self(self.0.slice(slice))
    }

    /// Sets up a new user namespace for the executed processes and
    /// configures a minimal user and group mapping.
    ///
    /// Read `PrivateUsers=` in [systemd.exec(5)](man:systemd.exec(5))
    /// for details.
    ///
    /// This setting is unavailable with the feature `systemd_251`
    /// disabled.
    #[cfg(feature = "systemd_251")]
    pub fn private_users(self) -> Self {
        Self(self.0.private_users())
    }

    /// Configure the time to wait for the service itself to stop.
    /// If the service doesn't terminate in the specified time, it will be
    /// forcibly terminated by SIGKILL.
    ///
    /// A [Duration] exceeding [u64::MAX] microseconds is trimmed to
    /// [u64::MAX] microseconds silently.
    ///
    /// Read `TimeoutStopSec=` in
    /// [systemd.service(5)](man:systemd.service(5)) for details.
    ///
    /// This setting will be unavailable with the feature `systemd_188`
    /// disabled.
    #[cfg(feature = "systemd_188")]
    pub fn timeout_stop(self, d: Duration) -> Self {
        Self(self.0.timeout_stop(d))
    }

    /// Start the transient service.
    pub async fn start<'a>(self) -> Result<StartedRun<'a>> {
        self.0.start().await
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
            mount: vec![],
            mount_api_vfs: false,
            private_devices: false,
            no_new_privileges: false,
            limit_fsize: None,
            limit_fsize_soft: None,
            limit_stack: None,
            limit_stack_soft: None,
            limit_nofile: None,
            limit_nofile_soft: None,
            limit_nproc: None,
            limit_nproc_soft: None,
            limit_core: None,
            limit_core_soft: None,
            stdin: None,
            stdout: None,
            stderr: None,
            current_dir: None,
            protect_proc: ProtectProcInternal::Default,
            slice: None,
            private_users: false,
            timeout_stop: None,
            cpu_sched: CpuScheduling::default(),
            joins_namespace_of: vec![],
        }
    }

    /// Append an argument to the command line.
    pub fn arg<T: AsRef<str>>(mut self, arg: T) -> Self {
        self.args.push(arg.as_ref().to_string());
        self
    }

    /// Append multiple arguments to the command line.
    pub fn args<T: AsRef<str>, I: IntoIterator<Item = T>>(mut self, args: I) -> Self {
        self.args
            .extend(args.into_iter().map(|x| x.as_ref().to_owned()));
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
    ///
    /// This setting will be unavailable with the feature `systemd_229`
    /// disabled.
    #[cfg(feature = "systemd_229")]
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
    /// And, if `systemd_232` is disabled, this setting will also be
    /// unavailable.
    ///
    /// A [Byte] exceeding [u64::MAX] bytes is trimmed to [u64::MAX] bytes
    /// silently.
    ///
    /// Read `MemorySwapMax=` in
    /// [systemd.resource-control(5)](man:systemd.resource-control(5))
    /// for details.
    #[cfg(feature = "unified_cgroup")]
    #[cfg(feature = "systemd_232")]
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
    pub fn cpu_quota(mut self, percent: NonZeroU64) -> Self {
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
    pub fn allowed_cpus<'a, I: IntoIterator<Item = &'a usize>>(mut self, cpus: I) -> Self {
        self.allowed_cpus = cpus.into_iter().copied().collect();
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
    #[cfg(feature = "systemd_248")]
    pub fn private_ipc(mut self) -> Self {
        self.private_ipc = true;
        self
    }

    /// Set up a mount point for the transient service.  See [Mount] for
    /// details.
    ///
    /// This setting is not available if the feature `systemd_233` is
    /// disabled.
    #[cfg(feature = "systemd_233")]
    pub fn mount<T: AsRef<str>>(mut self, mount_point: T, mount: Mount) -> Self {
        self.mount.push((mount_point.as_ref().to_owned(), mount));
        self
    }

    /// Mount the API file systems `/proc`, `/sys`, `/dev`, and `/run`
    /// for the private mount namespace of the transient service.
    ///
    /// Read `MountAPIVFS=` in [systemd.exec(5)](man:systemd.exec(5)) for
    /// details.
    ///
    /// Implied by [Identity::dynamic].
    ///
    /// This setting is mostly useful with [RunSystem::mount] at `/`, but
    /// you'll need to ensure the mount points for the API file systems
    /// existing first if this setting is specified or implied.
    ///
    /// This setting is not available if the feature `systemd_233` is
    /// disabled.  And, if the version of systemd is less than 248, `/run`
    /// is not affected by this setting.  You may use [RunSystem::mount] to
    /// control `/run` more precisely anyway.
    #[cfg(feature = "systemd_233")]
    pub fn mount_api_vfs(self) -> Self {
        Self {
            mount_api_vfs: true,
            ..self
        }
    }

    /// Sets up a new `/dev` mount for the executed processes and only adds
    /// API pseudo devices such as `/dev/null` to it, but no physical
    /// devices such as `/dev/sda`, system memory `/dev/mem`, system ports
    /// `/dev/port` and others.
    ///
    /// Read `PrivateDevices=` in [systemd.exec(5)](man:systemd.exec(5)) for
    /// details.
    ///
    /// This setting is not available if the feature `systemd_227` is
    /// disabled.
    #[cfg(feature = "systemd_227")]
    pub fn private_devices(self) -> Self {
        Self {
            private_devices: true,
            ..self
        }
    }

    /// Ensures that the service process and all its children can never gain
    /// new privileges through `execve()` (e.g. via setuid or setgid bits,
    /// or filesystem capabilities).
    ///
    /// Read `NoNewPrivileges=` in [systemd.exec(5)](man:systemd.exec(5))
    /// for details.
    ///
    /// Implied by [Identity::dynamic].
    ///
    /// This setting is not available if the feature `systemd_227` is
    /// disabled.
    #[cfg(feature = "systemd_227")]
    pub fn no_new_privileges(self) -> Self {
        Self {
            no_new_privileges: true,
            ..self
        }
    }

    /// Set soft and hard limits of the maximum size in bytes of files that
    /// the process may create.
    ///
    /// Any setting exceeding [u64::MAX] bytes will be trimmed to [u64::MAX]
    /// bytes silently.  And, if `soft` is greater than `hard`, it will be
    /// trimmed to `hard` silently.
    ///
    /// Read `LimitFSIZE=` in [systemd.exec(5)](man:systemd.exec(5)) and
    /// `RLIMIT_FSIZE` in [prlimit(2)](man:prlimit(2)) for details.
    pub fn limit_fsize_soft_hard(self, soft: Byte, hard: Byte) -> Self {
        let soft = std::cmp::min(soft, hard);
        Self {
            limit_fsize: Some(hard),
            limit_fsize_soft: Some(soft),
            ..self
        }
    }

    /// Shorthand for `self.limit_fsize_soft_hard(lim, lim)`.
    pub fn limit_fsize(self, lim: Byte) -> Self {
        self.limit_fsize_soft_hard(lim, lim)
    }

    /// Set soft and hard limits of the maximum size in bytes of files that
    /// the process may create.
    ///
    /// Any setting exceeding [u64::MAX] bytes will be trimmed to
    /// [u64::MAX] bytes silently.  And, if `soft` is greater than `hard`,
    /// it will be trimmed to `hard` silently.
    ///
    /// Read `LimitCORE=` in [systemd.exec(5)](man:systemd.exec(5)) and
    /// `RLIMIT_CORE` in [prlimit(2)](man:prlimit(2)) for details.
    pub fn limit_core_soft_hard(self, soft: Byte, hard: Byte) -> Self {
        let soft = std::cmp::min(soft, hard);
        Self {
            limit_core: Some(hard),
            limit_core_soft: Some(soft),
            ..self
        }
    }

    /// Shorthand for `self.limit_fsize_soft_hard(lim, lim)`.
    pub fn limit_core(self, lim: Byte) -> Self {
        self.limit_core_soft_hard(lim, lim)
    }

    /// Set soft and hard limits of the number of threads for the real user
    /// ID of the process.
    ///
    /// If `soft` is greater than `hard`, it will be trimmed to `hard`
    /// silently.
    ///
    /// Read `LimitNPROC=` in [systemd.exec(5)](man:systemd.exec(5)) and
    /// `RLIMIT_NPROC` in [prlimit(2)](man:prlimit(2)) for details.
    pub fn limit_nproc_soft_hard(self, soft: NonZeroU64, hard: NonZeroU64) -> Self {
        let soft = std::cmp::min(soft, hard);
        Self {
            limit_nproc: Some(hard.into()),
            limit_nproc_soft: Some(soft.into()),
            ..self
        }
    }

    /// Shorthand for `self.limit_nproc_soft_hard(lim, lim)`.
    pub fn limit_nproc(self, lim: NonZeroU64) -> Self {
        self.limit_nproc_soft_hard(lim, lim)
    }

    /// Set **the value one greater than** soft and hard limits of the
    /// number of file descriptors opened by the process.
    ///
    /// If `soft` is greater than `hard`, it will be trimmed to `hard`
    /// silently.
    ///
    /// Read `LimitNOFILE=` in [systemd.exec(5)](man:systemd.exec(5)) and
    /// `RLIMIT_NOFILE` in [prlimit(2)](man:prlimit(2)) for details.
    pub fn limit_nofile_soft_hard(self, soft: NonZeroU64, hard: NonZeroU64) -> Self {
        let soft = std::cmp::min(soft, hard);
        Self {
            limit_nofile: Some(hard.into()),
            limit_nofile_soft: Some(soft.into()),
            ..self
        }
    }

    /// Shorthand for `self.limit_nofile_soft_hard(lim, lim)`.
    pub fn limit_nofile(self, lim: NonZeroU64) -> Self {
        self.limit_nofile_soft_hard(lim, lim)
    }

    /// Set the soft and hard limit on the size of the process stack.
    ///
    /// If `soft` is greater than `hard`, it will be trimmed to `hard`
    /// silently.
    ///
    /// Read `LimitSTACK=` in [systemd.exec(5)](man:systemd.exec(5)) and
    /// `RLIMIT_STACK` in [prlimit(2)](man:prlimit(2)) for details.
    pub fn limit_stack_soft_hard(self, soft: Byte, hard: Byte) -> Self {
        let soft = std::cmp::min(soft, hard);
        Self {
            limit_stack: Some(hard),
            limit_stack_soft: Some(soft),
            ..self
        }
    }

    /// Shorthand for `self.limit_stack_soft_hard(lim, lim)`.
    pub fn limit_stack(self, lim: Byte) -> Self {
        self.limit_stack_soft_hard(lim, lim)
    }

    /// Controls where file descriptor 0 (STDIN) of the executed processes
    /// is connected to.
    ///
    /// Read [InputSpec] and `StandardInput=` in
    /// [systemd.exec(5)](man:systemd.exec(5)) for details.
    ///
    /// The default is [InputSpec::null()].
    pub fn stdin(self, spec: InputSpec) -> Self {
        Self {
            stdin: Some(spec),
            ..self
        }
    }

    /// Controls where file descriptor 1 (STDOUT) of the executed processes
    /// is connected to.
    ///
    /// Read [OutputSpec] and `StandardOutput=` in
    /// [systemd.exec(5)](man:systemd.exec(5)) for details.
    ///
    /// The default depends on system configuration.
    pub fn stdout(self, spec: OutputSpec) -> Self {
        Self {
            stdout: Some(spec),
            ..self
        }
    }

    /// Controls where file descriptor 2 (STDERR) of the executed processes
    /// is connected to.
    ///
    /// Read [OutputSpec] and `StandardError=` in
    /// [systemd.exec(5)](man:systemd.exec(5)) for details.
    ///
    /// The default depends on system configuration.
    pub fn stderr(self, spec: OutputSpec) -> Self {
        Self {
            stderr: Some(spec),
            ..self
        }
    }

    /// Sets the working directory for executed processes.
    ///
    /// Read `WorkingDirectory=` in
    /// [systemd.exec(5)](man:systemd.exec(5)) for details.
    ///
    /// This setting is unavailable with the feature `systemd_227`
    /// disabled.
    #[cfg(feature = "systemd_227")]
    pub fn current_dir<P: AsRef<str>>(self, path: P) -> Self {
        Self {
            current_dir: Some(path.as_ref().to_owned()),
            ..self
        }
    }

    /// Read [ProtectProc] for details.
    ///
    /// This setting will be unavailable if the feature `systemd_247` is
    /// disabled.
    #[cfg(feature = "systemd_247")]
    pub fn protect_proc(self, x: ProtectProc) -> Self {
        Self {
            protect_proc: x.0,
            ..self
        }
    }

    /// Put the transient service into a slice.
    ///
    /// Read `Slice=` in
    /// [systemd.resource-control(5)](man:systemd.resource-control(5))
    /// for details.
    pub fn slice<S: AsRef<str>>(self, slice: S) -> Self {
        Self {
            slice: Some(slice.as_ref().to_owned()),
            ..self
        }
    }

    /// Sets up a new user namespace for the executed processes and
    /// configures a minimal user and group mapping.
    ///
    /// Read `PrivateUsers=` in [systemd.exec(5)](man:systemd.exec(5))
    /// for details.
    ///
    /// This setting is unavailable with the feature `systemd_232`
    /// disabled.
    #[cfg(feature = "systemd_232")]
    pub fn private_users(self) -> Self {
        Self {
            private_users: true,
            ..self
        }
    }

    /// Configure the time to wait for the service itself to stop.
    /// If the service doesn't terminate in the specified time, it will be
    /// forcibly terminated by SIGKILL.
    ///
    /// A [Duration] exceeding [u64::MAX] microseconds is trimmed to
    /// [u64::MAX] microseconds silently.
    ///
    /// Read `TimeoutStopSec=` in
    /// [systemd.service(5)](man:systemd.service(5)) for details.
    ///
    /// This setting will be unavailable with the feature `systemd_188`
    /// disabled.
    #[cfg(feature = "systemd_188")]
    pub fn timeout_stop(self, d: Duration) -> Self {
        Self {
            timeout_stop: Some(d),
            ..self
        }
    }

    /// Specify CPU scheduling policy and real-time priority.
    /// See [CpuScheduling] for details.
    pub fn cpu_schedule(self, cpu_sched: CpuScheduling) -> Self {
        Self { cpu_sched, ..self }
    }

    /// See the same `/tmp/`, `/var/tmp/`, IPC namespace, and network
    /// namespace as one unit that is already started and specified with
    /// this setting.  If this setting is used multiple times and the
    /// specified units are started but not sharing their namespace, then
    /// it is not defined which namespace is joined.  Note that this
    /// setting only has an effect if [Self::private_network],
    /// [Self::private_ipc], and/or [Identity::dynamic] is in effect for
    /// both this unit and the unit whose namespace is joined.
    ///
    /// Read `JoinsNamespaceOf=` in [systemd.unit(5)](man:systemd.unit(5))
    /// for details.
    ///
    /// This setting is unavailable with the feature `systemd_227`
    /// disabled.
    #[cfg(feature = "systemd_227")]
    pub fn joins_namespace_of<S: AsRef<str>>(mut self, unit: S) -> Self {
        self.joins_namespace_of.push(unit.as_ref().into());
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

        for (k, v) in [
            ("WorkingDirectory", self.current_dir),
            ("Slice", self.slice),
        ] {
            if let Some(v) = v {
                properties.push((k, Value::from(v)));
            }
        }

        let join_ns = self.joins_namespace_of;
        if !join_ns.is_empty() {
            properties.push(("JoinsNamespaceOf", Value::from(join_ns)));
        }

        let proc = match self.protect_proc {
            ProtectProcInternal::NoAccess => Some("noaccess"),
            ProtectProcInternal::Invisible => Some("invisible"),
            ProtectProcInternal::Ptraceable => Some("ptraceable"),
            ProtectProcInternal::Default => None,
        };

        if let Some(v) = proc {
            properties.push(("ProtectProc", Value::from(v)));
        }

        let identity_prop = identity::unit_properties(&self.identity);
        properties.extend(identity_prop);

        for (k, v) in [
            ("RuntimeMaxUSec", &self.runtime_max),
            ("TimeoutStopUSec", &self.timeout_stop),
        ] {
            if let Some(d) = v {
                let usec = u64::try_from(d.as_micros()).unwrap_or(u64::MAX);
                properties.push((k, Value::from(usec)));
            }
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

        for (k, v) in [
            ("LimitNPROC", &self.limit_nproc),
            ("LimitNPROCSoft", &self.limit_nproc_soft),
            ("LimitNOFILE", &self.limit_nofile),
            ("LimitNOFILESoft", &self.limit_nofile_soft),
        ] {
            if let Some(v) = v {
                properties.push((k, Value::from(v)))
            }
        }

        let memory_max_name = if cfg!(feature = "systemd_231") {
            "MemoryMax"
        } else {
            "MemoryLimit"
        };

        for (k, v) in [
            (memory_max_name, &self.memory_max),
            ("MemorySwapMax", &self.memory_swap_max),
            ("LimitFSIZE", &self.limit_fsize),
            ("LimitFSIZESoft", &self.limit_fsize_soft),
            ("LimitSTACK", &self.limit_stack),
            ("LimitSTACKSoft", &self.limit_stack_soft),
            ("LimitCORE", &self.limit_core),
            ("LimitCORESoft", &self.limit_core_soft),
        ] {
            if let Some(v) = v {
                properties.push((k, Value::from(v.as_u64())))
            }
        }

        if let Some(v) = self.cpu_quota {
            let v = std::cmp::min(v, u64::MAX / 10000);
            properties.push(("CPUQuotaPerSecUSec", Value::from(v * 10000)));
        }

        for (k, v) in [
            ("PrivateNetwork", self.private_network),
            ("PrivateIPC", self.private_ipc),
            ("MountAPIVFS", self.mount_api_vfs),
            ("PrivateDevices", self.private_devices),
            ("NoNewPrivileges", self.no_new_privileges),
            ("PrivateUsers", self.private_users),
        ] {
            // Don't push false values as they may break on old Systemd.
            if v {
                properties.push((k, Value::from(true)))
            }
        }

        let mut p_bind = vec![];
        let mut p_bind_ro = vec![];
        let mut p_image = vec![];
        let mut p_tmpfs = vec![];
        for mnt in self.mount.into_iter().map(|(x, y)| mount::marshal(x, y)) {
            use mount::MarshaledMount::*;
            match mnt {
                Bind(a, b, c, d) => p_bind.push((a, b, c, d)),
                BindReadOnly(a, b, c, d) => p_bind_ro.push((a, b, c, d)),
                Normal(a, b, c, d) => p_image.push((a, b, c, d)),
                Tmpfs(a, b) => p_tmpfs.push((a, b)),
            }
        }

        if !p_bind.is_empty() {
            properties.push(("BindPaths", Value::from(p_bind)));
        }

        if !p_bind_ro.is_empty() {
            properties.push(("BindReadOnlyPaths", Value::from(p_bind_ro)));
        }

        if !p_image.is_empty() {
            properties.push(("MountImages", Value::from(p_image)));
        }

        if !p_tmpfs.is_empty() {
            properties.push(("TemporaryFileSystem", Value::from(p_tmpfs)));
        }

        let mut io_prop = vec![];

        for (pfx, (sfx, val)) in [
            ("StandardInput", self.stdin.map(ioredirect::marshal_input)),
            (
                "StandardOutput",
                self.stdout.map(ioredirect::marshal_output),
            ),
            ("StandardError", self.stderr.map(ioredirect::marshal_output)),
        ]
        .into_iter()
        .filter_map(|(a, b)| Some(a).zip(b))
        {
            let key = pfx.to_owned() + sfx;
            io_prop.push((key, val))
        }

        for (k, v) in io_prop.iter() {
            properties.push((k, Value::from(v)))
        }

        let (policy, priority, reset_on_fork) = cpu_sched::marshal(self.cpu_sched);

        for (k, v) in [
            ("CPUSchedulingPolicy", Value::from(policy)),
            ("CPUSchedulingResetOnFork", Value::from(reset_on_fork)),
        ] {
            properties.push((k, v));
        }

        if let Some(v) = priority {
            properties.push(("CPUSchedulingPriority", Value::from(v)));
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

impl StartedRun<'_> {
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
            (Ok(Value::U64(t0)), Ok(Value::U64(t1))) => t1 - t0,
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
