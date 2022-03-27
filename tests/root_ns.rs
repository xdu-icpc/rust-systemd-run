use systemd_run::{Identity, Run};

#[async_std::test]
#[ignore]
#[cfg(feature = "systemd_227")]
async fn test_root_private_network() {
    let r = Run::new("/usr/bin/wget")
        .arg("https://example.org/")
        .identity(Identity::dynamic())
        .start()
        .await
        .expect("should be able to start wget https://example.org")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(
        r.is_failed(),
        "should not be able to access Internet with private_network"
    );
}

#[async_std::test]
#[ignore]
#[cfg(feature = "systemd_249")]
async fn test_root_private_ipc() {
    const PATH: &'static str = concat!(env!("OUT_DIR"), "/test-aux/sh,");
    // Run twice, if IPC namespace seperation is not in-effect the secmond
    // run will fail.
    for _ in 0..2 {
        let r = Run::new(PATH)
            .identity(Identity::user_group("nobody", "nogroup"))
            .start()
            .await
            .expect("should be able to start the test program")
            .wait()
            .await
            .expect("should be able to get the status of the Run");
        assert!(
            r.is_failed(),
            "should be able to create POSIX shm in the new IPC namespace"
        );
    }
}
