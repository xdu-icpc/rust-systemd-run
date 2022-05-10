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
