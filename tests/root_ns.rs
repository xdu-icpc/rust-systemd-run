use systemd_run::{Identity, RunSystem};

#[async_std::test]
#[ignore]
#[cfg(feature = "systemd_227")]
async fn test_root_private_network_simple() {
    let r = RunSystem::new("/bin/true")
        .identity(Identity::dynamic())
        .start()
        .await
        .expect("should be able to start true")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(
        !r.is_failed(),
        "should be able to run true in private network namespace"
    );
}

#[async_std::test]
#[ignore]
#[cfg(feature = "systemd_236")]
async fn test_root_private_network_wget() {
    let r = RunSystem::new("/usr/bin/wget")
        .collect_on_fail()
        .arg("https://example.org/")
        .arg("-O")
        .arg("/dev/null")
        .private_network()
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
    const PATH: &'static str = concat!(env!("OUT_DIR"), "/test-aux/shm");
    // Run twice, if IPC namespace seperation is not in-effect the second
    // run will fail.
    for _ in 0..2 {
        let r = RunSystem::new(PATH)
            .private_ipc()
            .identity(Identity::user_group("nobody", "nogroup"))
            .start()
            .await
            .expect("should be able to start the test program")
            .wait()
            .await
            .expect("should be able to get the status of the Run");
        assert!(
            !r.is_failed(),
            "should be able to create POSIX shm in the new IPC namespace"
        );
    }
}
