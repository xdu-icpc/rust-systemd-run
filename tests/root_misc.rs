use systemd_run::{Identity, RunSystem};

#[async_std::test]
#[ignore]
#[cfg(feature = "systemd_227")]
async fn test_root_no_new_priv() {
    const PATH: &'static str = concat!(env!("OUT_DIR"), "/test-aux/nosgid");
    let r = RunSystem::new(PATH)
        .identity(Identity::user_group("nobody", "nogroup"))
        .no_new_privileges()
        .start()
        .await
        .expect("should be able to start test program")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(!r.is_failed(), "test program should finish successfully");
}

#[async_std::test]
#[ignore]
#[cfg(feature = "systemd_247")]
async fn test_root_protect_proc() {
    let r = RunSystem::new("/bin/test")
        .args(&["-e", "/proc/1"])
        .identity(Identity::dynamic())
        .protect_proc(systemd_run::ProtectProc::Invisible)
        .start()
        .await
        .expect("should be able to start /bin/test")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(r.is_failed(), "/proc/1 should be invisible");
}
