use zbus::proxy;
use zbus::zvariant::Value;

#[proxy(
    interface = "org.freedesktop.systemd1.Job",
    default_service = "org.freedesktop.systemd1"
)]
pub trait SystemdJob {
    // This is a dummy.  We can't rely on systemd job objects because they
    // are finished very quickly and then removed.
}

#[proxy(
    interface = "org.freedesktop.systemd1.Manager",
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1"
)]
pub trait SystemdManager {
    #[zbus(object = "SystemdJob")]
    fn start_transient_unit(
        &self,
        name: &str,
        mode: &str,
        properties: &[(&str, &Value<'_>)],
        _unused: &[(&str, &[(&str, &Value<'_>)])],
    );

    #[zbus(object = "SystemdJob")]
    fn stop_unit(&self, name: &str, mode: &str);
}
