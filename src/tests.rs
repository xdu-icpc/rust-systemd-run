use crate::Run;
use std::time::Duration;

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
    assert!(r.wall_time_usage > Duration::from_secs(1));
}
