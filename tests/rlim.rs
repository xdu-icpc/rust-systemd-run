#![cfg(feature = "systemd_236")]

use byte_unit::Byte;
use byte_unit::Unit::MiB;
use systemd_run::{RunSystem, RunUser};

#[async_std::test]
async fn test_limit_fsize() {
    const F: &'static str = concat!(env!("OUT_DIR"), "/test-aux/test-fsz");
    // Attempt to copy 4M, but use limit_fsize = 1M to stop it.
    let lim = Byte::from_i64_with_unit(1, MiB).unwrap();
    let r = RunUser::new("/bin/dd")
        .arg("if=/dev/zero")
        .arg("of=".to_owned() + F)
        .arg("bs=4096")
        .arg("count=1024")
        .limit_fsize(lim)
        .collect_on_fail()
        .start()
        .await
        .expect("should be able to start dd")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(r.is_failed(), "dd shouldn't be able to finish the write");

    let f = std::fs::File::open(F).expect("output file should exist");
    use std::os::unix::fs::MetadataExt;
    let meta = f.metadata().expect("should be able to get the metadata");
    assert!(Byte::from_u64(meta.size()) <= lim);
}

#[async_std::test]
async fn test_limit_nofile() {
    const E: &'static str = concat!(env!("OUT_DIR"), "/test-aux/waste-fd");
    let r = RunUser::new(E)
        .limit_nofile(16.try_into().unwrap())
        .collect_on_fail()
        .start()
        .await
        .expect("should be able to start test waste-fd")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(r.is_failed(), "fd shouldn't be wasted with no penalty");
}

#[async_std::test]
#[ignore]
async fn test_root_limit_stack() {
    // Unfortunately, in some environments (notably, GitHub runners) the
    // hard limit of stack is set to a finite value (likely same as the soft
    // limit).  So we have to run this as root to ensure it working.
    const E: &'static str = concat!(env!("OUT_DIR"), "/test-aux/use-stack");
    let lim = Byte::from_i64_with_unit(256, MiB).unwrap();
    let r = RunSystem::new(E)
        .limit_stack(lim)
        .start()
        .await
        .expect("should be able to start test use-stack")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(!r.is_failed(), "stack can be wasted in this test");
}

#[async_std::test]
#[ignore]
async fn test_root_limit_nproc() {
    const E: &'static str = concat!(env!("OUT_DIR"), "/test-aux/waste-pid");
    // Use dynamic() here so the test will be irrelevant to any other users,
    // as RLIM_NPROC accounts all PIDs for a user.  Set runtime_max()
    // because some implementations may dead lock when PID is exhausted.
    let r = RunSystem::new(E)
        .limit_nproc(16.try_into().unwrap())
        .identity(systemd_run::Identity::dynamic())
        .runtime_max(std::time::Duration::from_secs(1))
        .collect_on_fail()
        .start()
        .await
        .expect("should be able to start test waste-pid")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(r.is_failed(), "pid shouldn't be wasted with no penalty");
}
