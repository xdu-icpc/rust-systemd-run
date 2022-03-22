use std::time::Duration;
use systemd_run::Run;

#[async_std::test]
async fn test_true() {
    let r = Run::new("/bin/true")
        .start()
        .await
        .expect("should be able to start /bin/true")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(!r.is_failed(), "/bin/true should run successfully");
}

#[async_std::test]
async fn test_false() {
    let r = Run::new("/bin/false")
        .collect_on_fail()
        .start()
        .await
        .expect("should be able to start /bin/true")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(r.is_failed(), "/bin/false should fail");
}

#[async_std::test]
async fn test_wall_time_usage() {
    let r = Run::new("/bin/sleep")
        .arg("1")
        .start()
        .await
        .expect("should be able to start /bin/sleep")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(!r.is_failed(), "/bin/sleep should run successfully");
    assert!(r.wall_time_usage() > Duration::from_secs(1));
}

#[async_std::test]
async fn test_runtime_max() {
    let r = Run::new("/bin/sleep")
        .arg("2")
        .runtime_max(Duration::from_secs(1))
        .collect_on_fail()
        .start()
        .await
        .expect("should be able to start /bin/sleep")
        .wait()
        .await
        .expect("should be able to get the status of the Run");
    assert!(
        r.is_failed(),
        "/bin/sleep should have failed because of a timeout"
    );
    assert!(r.wall_time_usage() > Duration::from_secs(1));
    assert!(r.wall_time_usage() < Duration::from_secs(2));
}
