#![cfg(feature = "systemd_251")]
use systemd_run::RunUser;

#[async_std::test]
async fn test_unpriv_private_users() {
    const PATH: &'static str = concat!(env!("OUT_DIR"), "/test-aux/setuid");
    let r = RunUser::new(PATH)
        .private_users()
        .start()
        .await
        .expect("should be able to start the test program")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(
        !r.is_failed(),
        "UID 514 should not exist in the separate user namespace"
    );
}
