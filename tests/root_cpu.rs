use std::time::Duration;
use systemd_run::{Identity, Run};

#[async_std::test]
#[ignore]
#[cfg(feature = "systemd_244")]
#[cfg(feature = "unified_cgroup")]
async fn test_root_allowed_cpus() {
    const PATH: &'static str = concat!(env!("OUT_DIR"), "/test-aux/threads");
    let r = Run::new(PATH)
        .allowed_cpus(&[0])
        .identity(Identity::user_group("nobody", "nogroup"))
        .start()
        .await
        .expect("should be able to start the test program")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(!r.is_failed(), "test program should exit normally");
    assert!(
        r.wall_time_usage() >= Duration::from_secs(1),
        "test program should run for at least 1s on only one CPU"
    );
}

#[async_std::test]
#[ignore]
#[cfg(feature = "systemd_213")]
async fn test_root_cpu_quota() {
    const PATH: &'static str = concat!(env!("OUT_DIR"), "/test-aux/threads");
    let r = Run::new(PATH)
        .cpu_quota(std::num::NonZeroU64::new(100).unwrap())
        .identity(Identity::user_group("nobody", "nogroup"))
        .start()
        .await
        .expect("should be able to start the test program")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(!r.is_failed(), "test program should exit normally");
    // We use 0.9s instead of 1s, as the result can be so inaccurate because
    // the default CPUQuotaPeriodSec is 100ms.
    assert!(
        r.wall_time_usage() >= Duration::from_millis(900),
        "test program should run for at least 0.9s with 100% CPU quota"
    );
    assert!(
        r.wall_time_usage() <= Duration::from_millis(1100),
        "test program should run for about 1s with 100% CPU quota"
    );
}
