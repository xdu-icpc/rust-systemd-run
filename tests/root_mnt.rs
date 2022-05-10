#[cfg(feature = "systemd_233")]
use systemd_run::{Mount, RunSystem};

#[async_std::test]
#[ignore]
#[cfg(feature = "systemd_233")]
async fn test_root_mnt_bind_minimal() {
    const PATH: &'static str = concat!(env!("OUT_DIR"), "/test-aux");
    let r = RunSystem::new("/minimal")
        .mount("/", Mount::bind(PATH))
        .start()
        .await
        .expect("should be able to start test program")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(!r.is_failed());
}

#[cfg(feature = "systemd_236")]
async fn test_root_mnt_w(f: fn() -> Mount) {
    const EXE: &'static str = concat!(env!("OUT_DIR"), "/test-aux/rw");

    // Write something into the mount.
    let r = RunSystem::new(EXE)
        .arg("w")
        .arg("/tmp/rust-systemd-run-test")
        .mount("/tmp", f().writable())
        .start()
        .await
        .expect("writter should start successfully")
        .wait()
        .await
        .expect("writter should finish");
    assert!(!r.is_failed(), "writter should finish successfully");

    // Mount the filesystem ro this time, now it shouldn't be possible to modify
    // the content.
    let r = RunSystem::new("/bin/rm")
        .arg("/tmp/rust-systemd-run-test")
        .mount("/tmp", f())
        .collect_on_fail()
        .start()
        .await
        .expect("rm should start successfully")
        .wait()
        .await
        .expect("rm should finish");
    assert!(r.is_failed(), "shouldn't be able to edit ro filesystem");
}

#[cfg(feature = "systemd_236")]
async fn test_root_mnt(f: fn() -> Mount) {
    const EXE: &'static str = concat!(env!("OUT_DIR"), "/test-aux/rw");
    test_root_mnt_w(f).await;

    // Read the content back.
    let r = RunSystem::new(EXE)
        .arg("r")
        .arg("/tmp/rust-systemd-run-test")
        .mount("/tmp", f())
        .collect_on_fail()
        .start()
        .await
        .expect("reader should start successfully")
        .wait()
        .await
        .expect("reader should finish");
    assert!(!r.is_failed(), "should be able to read the content");

    // Ensure /tmp is really a mountpoint.
    assert!(
        !std::path::Path::new("/tmp/rust-systemd-run-test").exists(),
        "the test file shouldn't exist w/o image mounted",
    );
}

#[async_std::test]
#[ignore]
#[cfg(feature = "systemd_247")]
async fn test_root_mnt_image() {
    const IMG: &'static str = concat!(env!("OUT_DIR"), "/test-aux/floppy.img");

    // Create a floppy-like image first
    let r = RunSystem::new("/bin/dd")
        .arg("if=/dev/zero")
        .arg("of=".to_string() + IMG)
        .arg("bs=1024")
        .arg("count=1440")
        .start()
        .await
        .expect("this test requires dd")
        .wait()
        .await
        .expect("this test requires a runable dd");
    assert!(!r.is_failed(), "this test requires a functional dd");

    let r = RunSystem::new("/sbin/mkfs.vfat")
        .arg(IMG)
        .start()
        .await
        .expect("this test requires mkfs.vfat")
        .wait()
        .await
        .expect("this test requires a runable mkfs.vfat");
    assert!(!r.is_failed(), "this test requires a functional mkfs.vfat");

    test_root_mnt(|| Mount::normal(IMG)).await;
}

#[async_std::test]
#[ignore]
#[cfg(feature = "systemd_236")]
async fn test_root_mnt_bind() {
    const BIND: &'static str = concat!(env!("OUT_DIR"), "/test-aux");
    test_root_mnt(|| Mount::bind(BIND)).await;
}

#[async_std::test]
#[ignore]
#[cfg(feature = "systemd_238")]
async fn test_root_mnt_tmpfs() {
    test_root_mnt_w(|| Mount::tmpfs()).await;
}

#[async_std::test]
#[ignore]
#[cfg(feature = "systemd_247")]
async fn test_root_mnt_ignore_nonexist() {
    let r = RunSystem::new("/bin/true")
        .mount("/mnt", Mount::bind("/nonexist").ignore_nonexist())
        .mount("/usr", Mount::normal("/nonexist").ignore_nonexist())
        .start()
        .await
        .expect("should be able to start /bin/true")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(!r.is_failed(), "/bin/true should run successfully");
}

#[cfg(feature = "systemd_236")]
async fn test_devtmpfs(
    desc: &'static str,
    node: &'static str,
    f: fn(RunSystem) -> RunSystem,
    cmd_should_fail: bool,
) {
    let r = RunSystem::new("/bin/ls").arg(node);
    let r = if cmd_should_fail {
        r.collect_on_fail()
    } else {
        r
    };

    let r = f(r)
        .start()
        .await
        .expect("should be able to start /bin/ls")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(
        r.is_failed() == cmd_should_fail,
        "{} should{} exist with {}",
        node,
        if cmd_should_fail { " not" } else { "" },
        desc,
    );
}

#[async_std::test]
#[ignore]
#[cfg(feature = "systemd_236")]
async fn test_root_mnt_bind_norecursive() {
    let f = |x: RunSystem| x.mount("/", Mount::bind("/"));
    test_devtmpfs("/ binded non-recursively", "/sys/fs", f, true).await;
}

#[async_std::test]
#[ignore]
#[cfg(feature = "systemd_236")]
async fn test_root_mnt_apivfs() {
    let f = |x: RunSystem| x.mount("/", Mount::bind("/")).mount_api_vfs();
    test_devtmpfs("APIVFS mounted", "/sys/fs", f, false).await;
}

#[async_std::test]
#[ignore]
#[cfg(feature = "systemd_236")]
async fn test_root_mnt_private_devices() {
    let f = |x: RunSystem| x.mount("/", Mount::bind("/")).private_devices();
    test_devtmpfs("private_devices in effect", "/dev/kmsg", f, true).await;
    let f = |x: RunSystem| x.private_devices();
    test_devtmpfs("private_devices in effect", "/dev/kmsg", f, true).await;
    let f = |x: RunSystem| {
        x.mount("/", Mount::bind("/"))
            .mount_api_vfs()
            .private_devices()
    };
    test_devtmpfs("private_devices in effect", "/dev/kmsg", f, true).await;
}
