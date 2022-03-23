use std::time::Duration;
use systemd_run::{Identity, Run};

#[async_std::test]
#[ignore]
#[cfg(feature = "systemd_231")]
async fn test_root_dynamic_user() {
    let r = Run::new("/bin/true")
        .identity(Identity::dynamic())
        .start()
        .await
        .expect("should be able to start /bin/true")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(
        !r.is_failed(),
        "/bin/true should run successfully with Dynamic Identity"
    );
}

#[async_std::test]
#[ignore]
async fn test_root_nobody() {
    let r = Run::new("/bin/true")
        .identity(Identity::user_group("nobody", "nogroup"))
        .start()
        .await
        .expect("should be able to start /bin/true")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(
        !r.is_failed(),
        "/bin/true should run successfully with nobody Identity"
    );
}

#[async_std::test]
#[ignore]
#[cfg(feature = "systemd_236")]
async fn test_root_dynamic_user_access() {
    let f = "/run/rust_systemd_run_test_file";
    let r = Run::new("/bin/touch")
        .arg(f)
        .collect_on_fail()
        .identity(Identity::dynamic())
        .start()
        .await
        .expect("should be able to start /bin/true")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(
        r.is_failed(),
        "Dynamic identity should not be able to touch {}",
        f
    );
}

#[async_std::test]
#[ignore]
#[cfg(feature = "systemd_236")]
async fn test_root_nobody_access() {
    let f = "/run/rust_systemd_run_test_file";
    let r = Run::new("/bin/touch")
        .arg(f)
        .collect_on_fail()
        .identity(Identity::user_group("nobody", "nogroup"))
        .start()
        .await
        .expect("should be able to start /bin/true")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(
        r.is_failed(),
        "nobody Identity should not be able to touch {}",
        f
    );
}

#[async_std::test]
#[ignore]
#[cfg(feature = "systemd_244")]
#[cfg(feature = "unified_cgroup")]
async fn test_root_cpuset() {
    const PATH: &'static str = concat!(env!("OUT_DIR"), "/test-aux/cpuset");
    dbg!(PATH);
    let r = Run::new(PATH)
        .allowed_cpus(&[0])
        .identity(Identity::user_group("nobody", "nogroup"))
        .start()
        .await
        .expect("should be able to start cpuset test program")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(!r.is_failed(), "cpuset test program should exit normally");
    assert!(
        r.wall_time_usage() > Duration::from_secs(1),
        "cpuset test program should run for at least 1s on only one CPU"
    );
}
