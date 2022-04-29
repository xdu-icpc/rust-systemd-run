use systemd_run::{Identity, RunSystem};

#[async_std::test]
#[ignore]
#[cfg(feature = "systemd_231")]
async fn test_root_dynamic_user() {
    let r = RunSystem::new("/bin/true")
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
    let r = RunSystem::new("/bin/true")
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
    let r = RunSystem::new("/bin/touch")
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
    let r = RunSystem::new("/bin/touch")
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
