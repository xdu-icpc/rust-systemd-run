use systemd_run::{Identity, OutputSpec, RunSystem};

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
async fn test_root_private_network_curl() {
    let r = RunSystem::new("/usr/bin/curl")
        .collect_on_fail()
        .arg("https://example.org/")
        .arg("-O")
        .arg("/dev/null")
        .private_network()
        .identity(Identity::dynamic())
        .start()
        .await
        .expect("should be able to start curl https://example.org")
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

#[async_std::test]
#[ignore]
#[cfg(feature = "systemd_232")]
async fn test_root_private_users() {
    const PATH: &'static str = concat!(env!("OUT_DIR"), "/test-aux/setuid");
    let r = RunSystem::new(PATH)
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

#[async_std::test]
#[ignore]
#[cfg(feature = "systemd_227")]
async fn test_root_private_network_join() {
    const F1: &'static str = concat!(env!("OUT_DIR"), "/test-aux/aaa.txt");
    const F2: &'static str = concat!(env!("OUT_DIR"), "/test-aux/bbb.txt");

    let name = format!("test-{}.service", std::process::id());

    let p = RunSystem::new("/usr/bin/bash")
        .service_name(&name)
        .collect_on_fail()
        .arg("-c")
        .arg("trap '' SIGTERM; readlink /proc/self/ns/net; trap - SIGTERM; sleep infinity")
        .private_network()
        .stdout(OutputSpec::file(F1))
        .start()
        .await
        .expect("should be able to start a service with a private net ns");

    RunSystem::new("/usr/bin/readlink")
        .collect_on_fail()
        .arg("/proc/self/ns/net")
        .private_network()
        .joins_namespace_of(&name)
        .stdout(OutputSpec::file(F2))
        .start()
        .await
        .expect("should be able to start a service joining the previous net ns")
        .wait()
        .await
        .expect("should be able to get the status of the second service");

    // TODO we should really implement StartedRun::stop and use it instead
    RunSystem::new("/usr/bin/systemctl")
        .collect_on_fail()
        .arg("stop")
        .arg(&name)
        .start()
        .await
        .expect("should be able to start systemctl stop")
        .wait()
        .await
        .expect("should be able to stop the service");

    p.wait()
        .await
        .expect("should be able to get the status of the first service");

    let aaa = std::fs::read(F1).expect("should be able to read aaa.txt");
    let bbb = std::fs::read(F2).expect("should be able to read aaa.txt");
    assert!(aaa.starts_with(b"net:"));
    assert!(aaa == bbb);
}
